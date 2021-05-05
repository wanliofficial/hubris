//! STM32H7 Ethernet Server.

#![no_std]
#![no_main]
#![feature(min_const_generics)]

mod desc;
mod ring;
mod rx_ring;
mod tx_ring;

use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering};

use self::desc::*;
use userlib::*;

#[cfg(feature = "h743")]
use stm32h7::stm32h743 as device;

#[cfg(not(feature = "standalone"))]
const RCC: Task = Task::rcc_driver;

// For standalone mode -- this won't work, but then, neither will a task without
// a kernel.
#[cfg(feature = "standalone")]
const RCC: Task = Task::anonymous;

#[cfg(not(feature = "standalone"))]
const GPIO: Task = Task::gpio_driver;
#[cfg(feature = "standalone")]
const GPIO: Task = Task::anonymous;

#[export_name = "main"]
fn main() -> ! {
    sys_log!("it would be fun to have ethernet");
    turn_on_ethernet_controller();
    configure_ethernet_pins();

    let ethdma = unsafe { &*device::ETHERNET_DMA::ptr() };
    let ethmac = unsafe { &*device::ETHERNET_MAC::ptr() };

    // Soft-reset logic. This is controlled through the DMA block but affects
    // the MAC etc. Gotta do this before interacting with the peripheral.
    ethdma.dmamr.write(|w| w.swr().set_bit());
    // The soft reset completes quickly, but not instantaneously; don't proceed
    // until we've seen it.
    while ethdma.dmamr.read().swr().bit() {
        // spin
    }

    // Clock value hardcoded for 200MHz AHB frequency
    ethmac.macmdioar.write(|w| unsafe { w.cr().bits(0b0100) });

    // Program the DMA's bus interface. We'd love to set this to generate
    // fixed-length bursts, but it sure looks like ST has removed the ability to
    // configure burst length since the hardware was released, soooo
    ethdma.dmasbmr.reset();

    // Get a hold of the TX ring.
    let mut tx_ring = get_tx_ring();
    // Record the length MINUS ONE - the docs are wrong
    ethdma
        .dmactx_rlr
        .write(|w| unsafe { w.tdrl().bits(tx_ring.len() as u16 - 1) });
    // Record the ring base address.
    ethdma
        .dmactx_dlar
        .write(|w| unsafe { w.tdesla().bits(tx_ring.base_ptr() as u32 >> 2) });
    // Poke the tail pointer to make it clear that none of the descriptors are
    // worth reading yet. Note that at this point the two pointers are the same.
    ethdma.dmactx_dtpr.write(|w| unsafe {
        w.tdt().bits(tx_ring.next_ptr() as u32 >> 2)
    });
    // Bursts are overrated, set length to 1
    ethdma.dmactx_cr.write(|w| unsafe { w.txpbl().bits(1) });

    // Do the same for RX.
    let mut rx_ring = get_rx_ring();
    ethdma
        .dmacrx_rlr
        .write(|w| unsafe { w.rdrl().bits(rx_ring.len() as u16 - 1) });
    ethdma
        .dmacrx_dlar
        .write(|w| unsafe { w.rdesla().bits(rx_ring.base_ptr() as u32 >> 2) });
    ethdma.dmacrx_dtpr.write(|w| unsafe { w.rdt().bits(0) });
    ethdma
        .dmacrx_cr
        .write(|w| unsafe { w.rxpbl().bits(1).rbsz().bits(rx_ring::RxRing::MTU as u16) });

    // We're not appending any additional words to our descriptors.
    ethdma.dmaccr.write(|w| unsafe { w.dsl().bits(0) });

    // We'd like to hear about successful frame reception.
    ethdma.dmacier.write(|w| w.nie().set_bit().rie().set_bit());

    // Start transmit and receive DMA.
    ethdma.dmactx_cr.modify(|_, w| w.st().set_bit());
    ethdma.dmacrx_cr.modify(|_, w| w.sr().set_bit());

    sys_log!("ethernet DMA init complete");

    // MTL block init
    let ethmtl = unsafe { &*device::ETHERNET_MTL::ptr() };
    ethmtl
        .mtltx_qomr
        .write(|w| unsafe { w.tqs().bits(0b111).tsf().set_bit() });
    ethmtl.mtlrx_qomr.write(|w| unsafe { w.rsf().set_bit() });

    sys_log!("ethernet MTL init complete");

    // MAC block init

    ethmac.macpfr.write(|w| w.pr().set_bit());

    ethmac.maccr.write(|w| w.te().set_bit().re().set_bit());

    sys_log!("ethernet MAC online");

    sys_log!("attempting to bring ethernet link up");
    let (dup, spd) = smi_auto_negotiate(ethmac, 0);
    sys_log!("negotiated: {:?} {:?}", dup, spd);

    const TIMER: u32 = 1 << 1;
    const IRQ: u32 = 1 << 0;
    let mut next_tx = 0;
    sys_set_timer(Some(next_tx), TIMER);
    sys_irq_control(IRQ, true);

    let mut rx_count = 0;

    loop {
        let rm = sys_recv_open(&mut [], TIMER | IRQ);
        if rm.sender == TaskId::KERNEL {
            if rm.operation & TIMER != 0 {
                let ocom = tx_ring.transmit(PACKET.len(), |buf| {
                    buf.copy_from_slice(PACKET);
                    ring::Commit::Yes
                });

                if ocom.is_none() {
                    // No buffer was available, we'll try again in the next
                    // iteration.
                } else {
                    // We have enqueued a transmit descriptor, but the hardware
                    // itself has not been informed. Poke it.
                    let tail = tx_ring.next_ptr();
                    ethdma
                        .dmactx_dtpr
                        .write(|w| unsafe { w.tdt().bits(tail as u32 >> 2) });
                }

                sys_log!("rx: {}", rx_count);

                next_tx += 1000;
                sys_set_timer(Some(next_tx), TIMER);
            }

            if rm.operation & IRQ != 0 {
                // Diagnosing an ETH IRQ is kind of involved. Start at the high
                // level summary register.
                let dmaisr = ethdma.dmaisr.read();
                if dmaisr.dc0is().bit() {
                    // DMA interrupt.
                    if ethdma.dmacsr.read().ri().bit() {
                        // Received a packet.
                        // Clear interrupt summary and bit.
                        ethdma
                            .dmacsr
                            .write(|w| w.nis().set_bit().ri().set_bit());
                        loop {
                            let rr = rx_ring.receive(|_buf| {
                                rx_count += 1;
                                ring::Commit::Yes
                            });
                            if rr.is_some() {
                                ethdma
                                    .dmacrx_dtpr
                                    .write(|w| unsafe { w.rdt().bits(0) });
                            } else {
                                break;
                            }
                        }
                    }
                }
                if dmaisr.macis().bit() {
                    // MAC interrupt.
                }
                if dmaisr.mtlis().bit() {
                    // MTL interrupt.
                }
                sys_irq_control(IRQ, true);
            }
        } else {
            // huh.
            sys_log!("unexpected incoming message to ETH driver");
        }
    }
}

static PACKET: &[u8] = &[
    // Destination
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // Source
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Ethertype
    0x81, 0x37, // lol IPX
    // Payload
    0xDE, 0xAD, 0xBE, 0xEF,
];

fn turn_on_ethernet_controller() {
    let rcc = drv_stm32h7_rcc_api::Rcc::from(TaskId::for_index_and_gen(
        RCC as usize,
        Generation::default(),
    ));
    const PNUM_MAC: usize = 15; // see AHB1ENR bits.
    const PNUM_TX: usize = 16; // see AHB1ENR bits.
    const PNUM_RX: usize = 17; // see AHB1ENR bits.

    rcc.enable_clock_raw(PNUM_MAC).unwrap();
    rcc.enable_clock_raw(PNUM_TX).unwrap();
    rcc.enable_clock_raw(PNUM_RX).unwrap();

    rcc.enter_reset_raw(PNUM_MAC).unwrap();
    rcc.leave_reset_raw(PNUM_MAC).unwrap();
}

#[cfg(any(feature = "standalone", target_board = "nucleo-h743zi2"))]
fn configure_ethernet_pins() {
    // This board's mapping:
    //
    // RMII REF CLK     PA1
    // MDIO             PA2
    // RMII RX DV       PA7
    //
    // RMII TXD1        PB13
    //
    // MDC              PC1
    // RMII RXD0        PC4
    // RMII RXD1        PC5
    //
    // RMII TX EN       PG11
    // RMII TXD0        PG13
    use drv_stm32h7_gpio_api::*;

    let gpio = Gpio::from(TaskId::for_index_and_gen(
        GPIO as usize,
        Generation::default(),
    ));

    gpio.configure(
        Port::A,
        (1 << 1) | (1 << 2) | (1 << 7),
        Mode::Alternate,
        OutputType::PushPull,
        Speed::VeryHigh,
        Pull::None,
        Alternate::AF11,
    )
    .unwrap();
    gpio.configure(
        Port::B,
        1 << 13,
        Mode::Alternate,
        OutputType::PushPull,
        Speed::VeryHigh,
        Pull::None,
        Alternate::AF11,
    )
    .unwrap();
    gpio.configure(
        Port::C,
        (1 << 1) | (1 << 4) | (1 << 5),
        Mode::Alternate,
        OutputType::PushPull,
        Speed::VeryHigh,
        Pull::None,
        Alternate::AF11,
    )
    .unwrap();
    gpio.configure(
        Port::G,
        (1 << 11) | (1 << 13),
        Mode::Alternate,
        OutputType::PushPull,
        Speed::VeryHigh,
        Pull::None,
        Alternate::AF11,
    )
    .unwrap();
}

fn smi_write(
    ethmac: &device::ethernet_mac::RegisterBlock,
    phy: u8,
    register: u8,
    value: u16,
) {
    ethmac.macmdiodr.write(|w| unsafe { w.md().bits(value) });
    ethmac.macmdioar.modify(|_, w| unsafe {
        w.pa()
            .bits(phy)
            .rda()
            .bits(register)
            .goc()
            .bits(0b01) // ??
            .mb()
            .set_bit()
    });
    while { ethmac.macmdioar.read().mb().bit() } {
        hl::sleep_for(1);
    }
}

fn smi_read(
    ethmac: &device::ethernet_mac::RegisterBlock,
    phy: u8,
    register: u8,
) -> u16 {
    ethmac.macmdioar.modify(|_, w| unsafe {
        w.pa()
            .bits(phy)
            .rda()
            .bits(register)
            .goc()
            .bits(0b11)
            .mb()
            .set_bit()
    });
    while { ethmac.macmdioar.read().mb().bit() } {
        hl::sleep_for(1);
    }
    ethmac.macmdiodr.read().md().bits()
}

const NTX: usize = 4;

fn get_tx_ring() -> tx_ring::TxRing {
    static TAKEN: AtomicBool = AtomicBool::new(false);

    if TAKEN.swap(true, Ordering::SeqCst) {
        panic!();
    }

    // Note: these are MaybeUninit<ARRAY> rather than [MaybeUninit<T>; N]
    // because MaybeUninit ain't Copy.
    #[link_section = ".eth_bulk"]
    static mut DTABLE: MaybeUninit<[TxDescriptor; NTX]> = MaybeUninit::uninit();
    #[link_section = ".eth_bulk"]
    static mut BTABLE: MaybeUninit<[[u8; tx_ring::TxRing::MTU]; NTX]> =
        MaybeUninit::uninit();

    let dtable: &mut [MaybeUninit<TxDescriptor>; NTX] =
        unsafe { core::mem::transmute(&mut DTABLE) };
    let btable: &mut [MaybeUninit<[u8; tx_ring::TxRing::MTU]>; NTX] =
        unsafe { core::mem::transmute(&mut BTABLE) };
    tx_ring::TxRing::new(dtable, btable)
}

const NRX: usize = 4;

fn get_rx_ring() -> rx_ring::RxRing {
    static TAKEN: AtomicBool = AtomicBool::new(false);

    if TAKEN.swap(true, Ordering::SeqCst) {
        panic!();
    }
    // Note: these are MaybeUninit<ARRAY> rather than [MaybeUninit<T>; N]
    // because MaybeUninit ain't Copy.
    #[link_section = ".eth_bulk"]
    static mut DTABLE: MaybeUninit<[RxDescriptor; NRX]> = MaybeUninit::uninit();
    #[link_section = ".eth_bulk"]
    static mut BTABLE: MaybeUninit<[[u8; rx_ring::RxRing::MTU]; NRX]> =
        MaybeUninit::uninit();

    let dtable: &mut [MaybeUninit<RxDescriptor>; NRX] =
        unsafe { core::mem::transmute(&mut DTABLE) };
    let btable: &mut [MaybeUninit<[u8; rx_ring::RxRing::MTU]>; NRX] =
        unsafe { core::mem::transmute(&mut BTABLE) };
    rx_ring::RxRing::new(dtable, btable)
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Speed {
    _10,
    _100,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Duplex {
    Half,
    Full,
}

fn smi_auto_negotiate(
    ethmac: &device::ethernet_mac::RegisterBlock,
    phy: u8,
) -> (Duplex, Speed) {
    smi_reset_phy(ethmac, phy);
    // Set bits 12 (Auto-Neg) and 9 (Restart Auto-Neg) in BMCR
    sys_log!("starting autoneg");
    smi_write(
        ethmac,
        phy,
        0,
        smi_read(ethmac, phy, 0) | (1 << 12) | (1 << 9),
    );
    // Wait for link. Sample duplex and speed when it comes up.
    let (dup, spd) = loop {
        let physts = smi_read(ethmac, phy, 0x1F);
        // Check if autonegotiation is done.
        // TODO: this is specific to the LAN8742 part. Apparently different PHYs
        // do this different ways? Frustrating.
        if physts & (1 << 12) == 0 {
            continue;
        }
        match (physts >> 2) & 0b111 {
            0b001 => break (Duplex::Half, Speed::_10),
            0b101 => break (Duplex::Full, Speed::_10),
            0b010 => break (Duplex::Half, Speed::_100),
            0b110 => break (Duplex::Full, Speed::_100),
            x => panic!("unexpected phy status: {:b}", x),
        }
    };

    ethmac.maccr.modify(|_, w| w.fes().set_bit().dm().set_bit());

    (dup, spd)
}

fn smi_reset_phy(ethmac: &device::ethernet_mac::RegisterBlock, phy: u8) {
    sys_log!("resetting PHY");
    smi_write(ethmac, phy, 0, 1 << 15);
    while smi_read(ethmac, phy, 0) & (1 << 15) != 0 {
        // spin; smi_read sleeps for us
    }
}
