/// Common descriptor format.
///
/// This is the right shape for both transmit and receive descriptors containing
/// both data and context. It provides a place to centralize the volatile
/// accesses and barriers we need to correctly share descriptors with DMA.
#[repr(C)]
struct RawDescriptor {
    des: [vcell::VolatileCell<u32>; 4],
}

impl Default for RawDescriptor {
    fn default() -> Self {
        Self {
            des: [
                vcell::VolatileCell::new(0),
                vcell::VolatileCell::new(0),
                vcell::VolatileCell::new(0),
                vcell::VolatileCell::new(0),
            ],
        }
    }
}

/// A transmit data descriptor.
#[repr(transparent)]
#[derive(Default)]
pub struct TxDescriptor {
    inner: RawDescriptor,
}

impl TxDescriptor {
    pub fn write_buf1_address(&self, ptr: *const ()) {
        self.inner.des[0].set(ptr as u32)
    }

    pub fn write_buf2_address(&self, ptr: *const ()) {
        self.inner.des[1].set(ptr as u32)
    }

    pub fn write_tdes2(&self, val: Tdes2) {
        self.inner.des[2].set(val.0)
    }

    pub fn read_tdes3(&self) -> Tdes3 {
        Tdes3(self.inner.des[3].get())
    }

    pub fn write_tdes3(&mut self, val: Tdes3) {
        self.inner.des[3].set(val.0)
    }
}

bitfield::bitfield! {
    /// Contents of the TDES2 word in descriptors published to the DMA
    /// controller.
    pub struct Tdes2(u32);
    /// Interrupt on Completion; when this descriptor is published, the TI bit
    /// in the DMACSR will be set.
    pub ioc, set_ioc: 31;
    /// Transmit Timestamp Enable: requests recording of the transmission
    /// timestamp.
    pub ttse, set_ttse: 30;
    /// Buffer 2 length; number of valid bytes at the buffer 2 address.
    pub buf2_len, set_buf2_len: 29, 16;
    /// VLAN Tag Insertion/Replacement.
    pub vtir, set_vtir: 15, 14;
    /// Buffer 1 length; number of valid bytes at the buffer 1 address.
    pub buf1_len, set_buf1_len: 13, 0;
}
bitfield::bitfield! {
    /// Contents of the TDES3 word in descriptors published to the DMA
    /// controller.
    pub struct Tdes3(u32);
    /// The "owned" bit indicates that the descriptor is owned by the DMA and is
    /// pending processing.
    pub own, set_own: 31;
    /// Context type (should be 0/false).
    pub ctxt, set_ctxt: 30;
    /// First Descriptor flag; contains the buffers that begin a new packet.
    pub fd, set_fd: 29;
    /// Last Descriptor flag; contains the final buffer of a packet, will be
    /// updated with transmission status.
    pub ld, set_ld: 28;
    /// CRC Pad Control. You basically always want this set to 0b00.
    pub cpc, set_cpc: 27, 26;
    /// Source Address Insertion Control.
    pub saic, set_saic: 25, 23;
    /// TCP Header Length; length of TCP header when TSE is on.
    pub thl, set_thl: 22, 19;
    /// TCP Segmentation Enable.
    pub tse, set_tse: 18;

    /// Checksum Insertion Control (valid only when TSE is off).
    pub cic, set_cic: 17, 16;
    /// Frame Length (valid only when TSE is off).
    pub fl, set_fl: 14, 0;

    /// TCP Payload Length (valid only when TSE is on).
    pub tpl, set_tpl: 17, 0;
}

bitfield::bitfield! {
    /// Contents of the TDES3 word in descriptors that have been processed and
    /// updated by the DMA controller.
    pub struct Tdes3Status(u32);
    /// The "owned" bit indicates that the descriptor is owned by the DMA and is
    /// pending processing.
    pub own, set_own: 31;
    /// Context type (should be 0/false).
    pub ctxt, set_ctxt: 30;
    /// First Descriptor flag; contains the buffers that begin a new packet.
    pub fd, set_fd: 29;
    /// Last Descriptor flag; contains the final buffer of a packet, will be
    /// updated with transmission status.
    pub ld, set_ld: 28;

    /// Transmit Timestamp Status. When set, timestamp was captured in TDES0/1.
    /// Valid only when LD is set (i.e. on the last descriptor of a packet).
    pub ttss, set_ttss: 17;

    /// Error Summary. Set when any of the error flags below are set.
    pub es, set_es: 15;
    /// Jabber Timeout.
    pub jt, set_jt: 14;
    /// Packet Flushed.
    pub ff, set_ff: 13;
    /// Payload Checksum Error.
    pub pce, set_pce: 12;
    /// Loss Of Carrier.
    pub loc, set_loc: 11;
    /// No Carrier.
    pub nc, set_nc: 10;
    /// Late Collision.
    pub lc, set_lc: 9;
    /// Excessive Collision.
    pub ec, set_ec: 8;
    /// Collision Count.
    pub cc, set_cc: 7, 4;
    /// Excessive Deferral.
    pub ed, set_ed: 3;
    /// UnderFlow.
    pub uf, set_uf: 2;
    /// Deferred Bit.
    pub db, set_db: 1;
    /// IP Header Error.
    pub ihe, set_ihe: 0;
}

/// A receive data descriptor.
#[repr(transparent)]
#[derive(Default)]
pub struct RxDescriptor {
    inner: RawDescriptor,
}

impl RxDescriptor {
    pub fn write_buf1_address(&mut self, addr: *mut u8) {
        self.inner.des[0].set(addr as u32)
    }

    pub fn read_rdes3(&self) -> Rdes3 {
        Rdes3(self.inner.des[3].get())
    }
    pub fn write_rdes3(&mut self, val: Rdes3) {
        self.inner.des[3].set(val.0)
    }

    pub fn read_rdes3_status(&self) -> Rdes3Status {
        Rdes3Status(self.inner.des[3].get())
    }
}

bitfield::bitfield! {
    /// Contents of the RDES3 word in descriptors published to the DMA
    /// controller.
    pub struct Rdes3(u32);
    /// The "owned" bit indicates that the descriptor is owned by the DMA and is
    /// pending processing.
    pub own, set_own: 31;
    /// Interrupt On Completion
    pub ioc, set_ioc: 30;
    /// Buffer 2 Address Valid
    pub buf2v, set_buf2v: 25;
    /// Buffer 1 Address Valid
    pub buf1v, set_buf1v: 24;
}

bitfield::bitfield! {
    /// Contents of the RDES3 word in descriptors that have been processed and
    /// updated by the DMA controller.
    pub struct Rdes3Status(u32);
    impl Debug;
    /// The "owned" bit indicates that the descriptor is owned by the DMA and is
    /// pending processing.
    pub own, set_own: 31;
    /// Context type (should be 0/false).
    pub ctxt, set_ctxt: 30;
    /// First Descriptor flag; contains the buffers that begin a new packet.
    pub fd, set_fd: 29;
    /// Last Descriptor flag; contains the final buffer of a packet, will be
    /// updated with transmission status.
    pub ld, set_ld: 28;
    /// Receive Status RDES2 Valid.
    pub rs2v, set_rs2v: 27;
    /// Receive Status RDES1 Valid.
    pub rs1v, set_rs1v: 26;
    /// Receive Status RDES0 Valid.
    pub rs0v, set_rs0v: 25;
    /// CRC Error.
    pub ce, set_ce: 24;
    /// Giant Packet.
    pub gp, set_gp: 23;
    /// Receive Watchdog Timeout.
    pub rwt, set_rwt: 22;
    /// Overflow Error.
    pub oe, set_oe: 21;
    /// Receive Error.
    pub re, set_re: 20;
    /// Dribble Bit Error.
    pub de, set_de: 19;
    /// Length/Type Field.
    pub lt, set_lt: 18, 16;

    /// Error Summary. Set when any of the error flags below are set.
    pub es, set_es: 15;
    /// Packet Length.
    pub pl, set_pl: 14, 0;
}
