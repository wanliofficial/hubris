use core::mem::MaybeUninit;

use crate::desc::TxDescriptor;
use crate::ring::*;

/// Implementation of Ring operations for the `TxDescriptor`.
unsafe impl Descriptor for TxDescriptor {
    // Not until we transmit something, thanks.
    const INITIALLY_OWNED_BY_HW: bool = false;

    fn initial_state(_: *mut u8, _: usize) -> Self {
        // Transmit descriptors don't need to start out paired with a buffer,
        // because we'll have an opportunity to fill them in before use.
        Self::default()
    }

    fn is_owned_by_hw(&self) -> bool {
        self.read_tdes3().own()
    }

    fn set_owned_by_hw(&'static mut self) {
        let mut t = self.read_tdes3();
        t.set_own(true);
        self.write_tdes3(t);
        // Any use of `self` past this point is UB.
    }
}

/// A transmit-specialized descriptor and buffer ring.
///
/// This type adapts the more general `Ring` type to transmit applications,
/// fixing the descriptor type and buffer size.
pub struct TxRing(Ring<TxDescriptor, { TxRing::MTU }>);

impl TxRing {
    /// Maximum size of packet that can be transmitted.
    ///
    /// If you raise this, you will need to adjust MAC settings to enable jumbo
    /// frames, which does not happen in this module.
    pub const MTU: usize = 1536;

    pub fn new(
        descriptors: &'static mut [MaybeUninit<TxDescriptor>],
        buffers: &'static mut [MaybeUninit<[u8; Self::MTU]>],
    ) -> Self {
        Self(Ring::new(descriptors, buffers))
    }

    /// Prepares to transmit a packet `size` bytes in length if a descriptor is
    /// available.
    ///
    /// If one is available, `transmit` calls `body` with an exclusive reference
    /// to one of the packet buffers (or, more specifically, to a `size`-byte
    /// slice of a packet buffer). `body` must then do one of two things:
    ///
    /// 1. Fill in all `size` bytes of the buffer with a packet to be
    ///    transmitted, and then return `Commit::Yes`, or
    /// 2. Return `Commit::No`. In this case, changes to the buffer (if any)
    ///    will be ignored.
    ///
    /// # Specifying the size up front
    ///
    /// The API currently requires that you know the exact length of the packet
    /// in advance -- `body` does not get an opportunity to return it after the
    /// fact. This might prove annoying and could be changed. The motivation is
    /// that this lets us defensively fill and truncate the packet buffer we
    /// give to `body`, reducing the risk of accidentally sending uninitialized
    /// data, while not requiring us to defensively scribble the full MTU every
    /// time.
    pub fn transmit(
        &mut self,
        size: usize,
        body: impl FnOnce(&mut [u8]) -> Commit,
    ) -> Option<Commit> {
        assert!(size < Self::MTU);

        // Here's the thing we'll do with the buffer if it becomes available.
        // This is here instead of being written inline as an argument to
        // `with_next_buffer` because doing so causes its entire contents to be
        // treated as an `unsafe` block, and I am not amused by this.
        let body_wrap =
            |desc: &mut TxDescriptor,
             buffer: &mut MaybeUninit<[u8; Self::MTU]>| {
                // Defensively scribble the section of buffer the caller has
                // requested, to detect errors where they fail to fill it in. We're
                // being rather paranoid and treating the buffer as poisonous
                // uninitialized memory, in part to avoid packet-to-packet data
                // leaks. If this proves to be a performance bottleneck there are
                // some other options we could consider (though simply passing
                // references to uninitialized memory is _not_ one of them).
                let buffer_window = unsafe {
                    core::ptr::write_bytes(
                        buffer.as_mut_ptr() as *mut u8,
                        0xAA,
                        size,
                    );
                    // Now that we have initialized it we can safely make a slice
                    // reference.
                    core::slice::from_raw_parts_mut(
                        buffer.as_mut_ptr() as *mut u8,
                        size,
                    )
                };
                // Let the caller fill in the section of the buffer they wanted.
                let com = body(buffer_window);
                // Ensure that they are interested in proceeding.
                if com == Commit::Yes {
                    // TODO this is where we'd flush the buffer to main memory.

                    // Fill out the transmit descriptor with the actual base address
                    // of our buffer, and the valid length.
                    desc.write_buf1_address(buffer_window.as_ptr() as *const ());
                    desc.write_tdes2({
                        let mut t = crate::desc::Tdes2(0);
                        t.set_buf1_len(buffer_window.len() as u32);
                        t
                    });

                    desc.write_tdes3({
                        let mut t = crate::desc::Tdes3(0);
                        // This is not a context descriptor, it's a data descriptor.
                        t.set_ctxt(false);
                        // It is this long:
                        t.set_fl(size as u32);
                        // This is both the first and last part of this packet.
                        t.set_fd(true);
                        t.set_ld(true);
                        // We would like both FCS insertion and padding plz.
                        t.set_cpc(0b00);
                        // Do not insert the source address. (TODO this might be
                        // useful).
                        t.set_saic(0b00);
                        // Do not use TCP segmentation offload.
                        t.set_tse(false);
                        // Don't do any IP checksum insertion (TODO)
                        t.set_cic(0b00);
                        t
                    });
                    // Unsafe to touch desc now. Communicate that to the compiler.
                    drop(desc);
                }
                com
            };

        // Safety: `body` does not set the descriptor OWN bit and thus fulfills
        // the conditions for `with_next_buffer`.
        unsafe { self.0.with_next_buffer(body_wrap) }
    }

    pub fn base_ptr(&self) -> *const TxDescriptor {
        self.0.base_ptr()
    }

    pub fn next_ptr(&self) -> *const TxDescriptor {
        self.0.next_ptr()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
