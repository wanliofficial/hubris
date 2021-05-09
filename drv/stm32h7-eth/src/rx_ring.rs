use core::mem::MaybeUninit;

use crate::desc::RxDescriptor;
use crate::ring::*;

/// Implementation of the Ring-related operations for the `RxDescriptor`.
unsafe impl Descriptor for RxDescriptor {
    const INITIALLY_OWNED_BY_HW: bool = true;

    fn initial_state(buffer: *mut u8, len: usize) -> Self {
        // The length is not stored in each descriptor, but is loaded centrally
        // into a register of the DMA controller, which then assumes that all
        // buffers are the same size. Don't accept anything smaller. (We can
        // tolerate buffers that are larger, though that would be a little
        // strange.)
        assert!(len >= RxRing::MTU);

        // Because RX descriptors begin life bound to a buffer and waiting for
        // receive, we need to go ahead and fill this out.
        let mut d = Self::default();
        d.write_buf1_address(buffer as *mut u8);
        d.write_rdes3({
            let mut rdes3 = crate::desc::Rdes3(0);
            // N.B. we deliberately DO NOT set the OWN bit here. See the docs on
            // the Descriptor trait.
            rdes3.set_ioc(true);
            rdes3.set_buf1v(true);
            rdes3
        });
        d
    }

    fn is_owned_by_hw(&self) -> bool {
        self.read_rdes3().own()
    }

    fn set_owned_by_hw(&'static mut self) {
        let mut t = self.read_rdes3();
        t.set_own(true);
        self.write_rdes3(t);
        // Any use of `self` past this point is potential UB. Notice that there
        // aren't any. Please keep it that way.
    }
}

/// Specialization of `Ring` for the receive case, fixing both the descriptor
/// type and buffer size.
pub struct RxRing(Ring<RxDescriptor, { RxRing::MTU }>);

impl RxRing {
    /// Maximum size of packet that can be received.
    ///
    /// If you raise this, you will need to adjust MAC settings to enable jumbo
    /// frames, which does not happen in this module.
    pub const MTU: usize = 1536;

    pub fn new(
        descriptors: &'static mut [MaybeUninit<RxDescriptor>],
        buffers: &'static mut [MaybeUninit<[u8; Self::MTU]>],
    ) -> Self {
        Self(Ring::new(descriptors, buffers))
    }

    /// Attempts to dequeue an incoming packet from the ring buffer.
    ///
    /// If an incoming packet is available, this will call `body` with its
    /// contents. The result of `body` determines whether the packet is treated
    /// as completely processed and recycled (`Commit::Yes`), or left in the
    /// queue, effectively undoing the receive (`Commit::No`).
    ///
    /// TODO: do we ever actually want to leave packets in the queue though?
    pub fn receive(
        &mut self,
        body: impl FnOnce(&[u8]) -> Commit,
    ) -> Option<Commit> {
        unsafe {
            self.0.with_next_buffer(|desc, buffer| {
                // The hardware has released this descriptor. It always writes the
                // descriptor when it does so -- that is, after all, how the
                // descriptor gets released -- but it only writes the _packet
                // buffer_ if it has actually received a packet. As opposed to using
                // the descriptor to report an error, say.

                let status = desc.read_rdes3_status();
                // Automatically commit error or context cases without invoking
                // `body`.
                let com = if status.ctxt() {
                    // We're not prepared to handle context descriptors, skip this.
                    userlib::sys_log!("unexpected context descriptor!");
                    Commit::Yes
                } else if !status.fd() || !status.ld() {
                    // We don't do multi-buffer receives. This should not happen.
                    userlib::sys_log!("partial packet descriptor?");
                    Commit::Yes
                } else if status.es() || status.oe() {
                    // Okay, this is substantially more reasonable: the MAC reported
                    // an error receiving this frame.
                    userlib::sys_log!("ETH RX error: {:?}", status);
                    Commit::Yes
                } else {
                    // The MAC is claiming to have written this many bytes of the
                    // receive buffer:
                    let len = status.pl() as usize;
                    // We must assume it is correct.
                    let buffer_window = unsafe {
                        core::slice::from_raw_parts(
                            buffer.as_ptr() as *const u8,
                            len,
                        )
                    };
                    //userlib::sys_log!("ETH rx packet of {} bytes", len);

                    body(buffer_window)
                };

                if com == Commit::Yes {
                    // We're done with this descriptor/packet, make it ready for
                    // reuse. Note that this does _not_ set OWN. Ring does that.
                    let first_elt = buffer.as_mut_ptr() as *mut u8;
                    *desc = RxDescriptor::initial_state(first_elt, Self::MTU);
                    // Unsafe to touch desc now; make that clear to the compiler.
                    drop(desc);
                }

                com
            })
        }
    }

    pub fn base_ptr(&self) -> *const RxDescriptor {
        self.0.base_ptr()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
