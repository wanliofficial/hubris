//! STM32H7 Ethernet Server.

#![no_std]
#![no_main]
#![allow(dead_code)]

pub mod mdio;
pub mod vsc8552_regs;

use ringbuf::*;
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

const PHY0: u8 = 0x1c | 0x01;
const PHY1: u8 = 0x1c | 0x02;

#[derive(Copy, Clone, PartialEq)]
enum RegisterAccess {
    None,
    Read(u8, u8, u16),
    Write(u8, u8, u16),
}

ringbuf!(RegisterAccess, 16, RegisterAccess::None);

#[export_name = "main"]
fn main() -> ! {
    sys_log!("it would be fun to have ethernet");
    turn_on_ethernet_controller();
    configure_ethernet_pins();
    configure_vsc8552_reset_pin();

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

    let mut mdio = MdioController::new();
    let mut port = vsc8552_regs::Port::new(&mut mdio, PHY0);

    // Configure the PHY in "QSGMII/SGMII MAC-to-100BASE-FX Link Partner" mode as
    // per 3.1.2 and 3.20.

    vsc8552_assert_reset(true);
    hl::sleep_for(100);
    vsc8552_assert_reset(false);

    // Make sure the PHY is ready, 120ms minimum.
    hl::sleep_for(200);

    port.read(vsc8552_regs::Main::PHYId1 as u8);
    port.read(vsc8552_regs::Main::PHYId2 as u8);

    vsc8552_regs::pre_init(&mut port);
    //vsc8552_regs::init(&mut port);
    //vsc8552_regs::soft_reset(&mut port);

    //port.set_page(vsc8552_regs::Pages::Main);

    loop {
        //port.read(vsc8552_regs::Main::ModeStatus as u8);
        //port.read(vsc8552_regs::Main::_100baseTxStatusExtension as u8);
        hl::sleep_for(1000);
    }
}

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

#[cfg(any(feature = "standalone", target_board = "gemini-bu-1"))]
fn configure_ethernet_pins() {
    // This board's mapping:
    //
    // RMII REF CLK     PA1
    // MDIO             PA2
    // RMII RX DV       PA7
    //
    // MDC              PC1
    // RMII RXD0        PC4
    // RMII RXD1        PC5
    //
    // RMII TX EN       PG11
    // RMII TXD1        PG12
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
        (1 << 11) | (1 << 12) | (1 << 13),
        Mode::Alternate,
        OutputType::PushPull,
        Speed::VeryHigh,
        Pull::None,
        Alternate::AF11,
    )
    .unwrap();
}

fn configure_vsc8552_reset_pin() {
    use drv_stm32h7_gpio_api::*;

    let gpio = Gpio::from(TaskId::for_index_and_gen(
        GPIO as usize,
        Generation::default(),
    ));

    let pin = 1 << 9;
    gpio.configure(
        Port::A,
        pin,
        Mode::Output,
        OutputType::OpenDrain,
        Speed::Medium,
        Pull::None,
        Alternate::AF0,
    )
    .unwrap();
    sys_log!("config called");
}

fn vsc8552_assert_reset(v: bool) {
    use drv_stm32h7_gpio_api::*;

    let gpio = Gpio::from(TaskId::for_index_and_gen(
        GPIO as usize,
        Generation::default(),
    ));

    let pin = 1 << 9;
    if v {
        gpio.set_reset(Port::A, 0, pin).unwrap()
    } else {
        gpio.set_reset(Port::A, pin, 0).unwrap()
    }
}

struct MdioController<'a> {
    mac: &'a device::ethernet_mac::RegisterBlock,
}

impl MdioController<'_> {
    fn new() -> Self {
        MdioController {
            mac: unsafe { &*device::ETHERNET_MAC::ptr() },
        }
    }
}

impl mdio::Controller for MdioController<'_> {
    fn read(&mut self, phy: u8, page_address: u8) -> u16 {
        self.mac.macmdioar.modify(|_, w| unsafe {
            w.pa()
                .bits(phy)
                .rda()
                .bits(page_address)
                .goc()
                .bits(0b11)
                .mb()
                .set_bit()
        });
        while self.mac.macmdioar.read().mb().bit() {
            hl::sleep_for(1);
        }

        let v = self.mac.macmdiodr.read().md().bits();
        ringbuf_entry!(RegisterAccess::Read(phy, page_address, v));
        v
    }

    fn write(&mut self, phy: u8, page_address: u8, value: u16) {
        self.mac.macmdiodr.write(|w| unsafe { w.md().bits(value) });
        self.mac.macmdioar.modify(|_, w| unsafe {
            w.pa()
                .bits(phy)
                .rda()
                .bits(page_address)
                .goc()
                .bits(0b01) // ??
                .mb()
                .set_bit()
        });
        while self.mac.macmdioar.read().mb().bit() {
            hl::sleep_for(1);
        }

        ringbuf_entry!(RegisterAccess::Write(phy, page_address, value));
    }

    fn write_masked(
        &mut self,
        phy: u8,
        page_address: u8,
        value: u16,
        mask: u16,
    ) {
        let v = (self.read(phy, page_address) & !mask) | (value & mask);
        self.write(phy, page_address, v)
    }
}

/*
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
    mdio: &MdioController,
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
    let v = smi_read(ethmac, phy, 0);
    smi_write(ethmac, phy, 0, v | 1 << 15);
    while smi_read(ethmac, phy, 0) & (1 << 15) != 0 {
        // spin; smi_read sleeps for us
    }
}
*/

//
// VSC PHY specific bits. These need to go somewhere else at some point.
//

/*
fn current_register_page(
    ethmac: &device::ethernet_mac::RegisterBlock,
    phy: u8,
) -> Option<vsc8552_regs::Pages> {
    vsc8552_regs::Pages::from_u16(smi_read(ethmac, phy, 31))
}

fn activate_register_page(
    ethmac: &device::ethernet_mac::RegisterBlock,
    phy: u8,
    page: vsc8552_regs::Pages,
) {
    smi_write(ethmac, phy, 31, page as u16);
}

fn exectute_processor_command(
    ethmac: &device::ethernet_mac::RegisterBlock,
    phy: u8,
    cmd: vsc8552_regs::ProcessorCommands,
) {
    smi_write(
        ethmac,
        phy,
        vsc8552_regs::G::ProcessorCommand as u8,
        cmd as u16,
    );
    while (smi_read(ethmac, phy, vsc8552_regs::G::ProcessorCommand as u8)
        & 0x8000)
        != 0
    {
        hl::sleep_for(25);
    }
}

/// Assert the reset on the internal 8051 MCU.
fn mcu_assert_reset(ethmac: &device::ethernet_mac::RegisterBlock, phy: u8) {
    if current_register_page(ethmac, phy).unwrap() != vsc8552_regs::Pages::G {
        activate_register_page(ethmac, phy, vsc8552_regs::Pages::G);
    }

    exectute_processor_command(
        ethmac,
        phy,
        vsc8552_regs::ProcessorCommands::Nop,
    );
}
*/
