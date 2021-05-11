use super::mdio;
use userlib::*;

pub struct Port<'a> {
    mdio: &'a mut dyn mdio::Controller,
    address: u8,
}

impl<'a> Port<'a> {
    pub fn new(mdio: &'a mut dyn mdio::Controller, address: u8) -> Self {
        Port {
            mdio: mdio,
            address: address,
        }
    }

    pub fn read(&mut self, page_address: u8) -> u16 {
        self.mdio.read(self.address, page_address)
    }

    pub fn write(&mut self, page_address: u8, value: u16) {
        self.mdio.write(self.address, page_address, value)
    }

    pub fn write_masked(&mut self, page_address: u8, value: u16, mask: u16) {
        self.mdio
            .write_masked(self.address, page_address, value, mask)
    }

    pub fn page(&mut self) -> Pages {
        Pages::from_u16(self.read(31)).unwrap()
    }

    pub fn set_page(&mut self, page: Pages) {
        self.write(31, page as u16)
    }
}

#[repr(u16)]
#[derive(Copy, Clone, Debug, FromPrimitive, PartialEq)]
pub enum Pages {
    Main = 0,
    E1 = 1,
    E2 = 2,
    E3 = 3,
    G = 16,
    Test = 0x2a30,
    TokenRing = 0x52b5,
}

#[repr(u8)]
pub enum Main {
    ModeControl = 0,
    ModeStatus = 1,
    PHYId1 = 2,
    PHYId2 = 3,
    AutoNegAdvertisement = 4,
    AutoNegLinkPartner = 5,
    AugoNegExpansion = 6,
    AugoNegNextPageTransmit = 7,
    AutoNegLinkParterNextPageReceive = 8,
    _1000baseTcontrol = 9,
    _1000baseTStatus = 10,
    Clause45Access1 = 13,
    Caluse45Access2 = 14,
    _1000baseTStatusExtension1 = 15,
    _1000baseTStatusExtension2 = 17,
    _100baseTxStatusExtension = 16,
    BypassControl = 18,
    ErrorCount1 = 19,
    ErrorCount2 = 20,
    ErrorCount3 = 21,
    ExtendedControlAndStatus = 22,
    ExtendedPHYControl1 = 23,
    ExtendedPHYControl2 = 24,
    InterruptMask = 25,
    InterruptStatus = 26,
    AuxControlAndStatus = 28,
    LEDMode = 29,
    LEDBehavior = 30,
    ExtendedPageAccess = 31,
}

#[repr(u8)]
pub enum E1 {
    SerDesMediaControl = 16,
    CopperMediaCRCGoodCount = 18,
    ExtendedModeSIGDETControl = 19,
    ExtendedPHYControl3 = 20,
    ExtendedPHYControl4 = 23,
    EPGControl1 = 29,
    EPGCOntrol2 = 30,
    ExtendedPageAccess = 31,
}

#[repr(u8)]
pub enum E2 {
    CuPMDTransmitControl = 16,
    ExtendedPageAccess = 31,
}

#[repr(u8)]
pub enum E3 {
    ExtendedPageAccess = 31,
}

#[repr(u8)]
pub enum G {
    Mcu0 = 0,
    Mcu1 = 1,
    ProcessorCommand = 18,
    MACConfigAndFastLink = 19,
    ExtendedPageAccess = 31,
}

//
// Main Registers
//

bitfield::bitfield! {
    /// Contents of the Mode Control register.
    pub struct ModeControl(u16);
    /// Software reset; Self-clearing. restores SMI to default state except for
    /// sticky and super-sticky bits.
    pub soft_resetting, set_soft_reset: 15;
    /// Loopback; Enable/disable loopback (on the MII).
    pub loopback, set_loopback: 14;
    /// Forced speed select.
    ///     00: 10Mbps
    ///     01: 100Mbps
    ///     10: 1000Mbps
    ///
    /// This may not work if link partner is 100BASE-FX?
    pub speed_msb, set_speed_msb: 6;
    pub speed_lsb, set_speed_lsb: 13;
    /// Autonegotiation; Enable/disable autonegotiation.
    pub auto_negotiation, set_auto_negotiation: 12;
    /// Power-down.
    pub powered_down, set_power_down: 11;
    /// Isolate. Disconnect the MII from the datapath. Traffic entering the PHY
    /// from the MAC-side or media-side terminates inside the PHY.
    pub isolated, set_isolated: 10;
    /// Restart autonegotiation. Self-clearing.
    pub autonegotiation_restarted, restart_negotiation: 9;
    /// Full duplex.
    pub full_duplex, set_full_duplex: 8;
    /// Collision test enable.
    pub collision_test_enabled, set_collision_test_enable: 7;
    /// Allow transmit from the MII regardless of whether the PHY has
    /// determined that a valid link has been established.
    ///     false: enable transmit from MII only if PHY link established
    ///     true: enable transmit from MII regardless if PHY link established
    ///
    /// Note: this only applies in 100BASE-FX and 1000BASE-X fiber media modes
    pub unidirectional_enabled, unidirectional_enable: 5;
}

bitfield::bitfield! {
    /// Contents of the Extended PHY Control 1 register.
    pub struct ExtendedPHYControl1(u16);
    /// MAC interface mode; Super-sticky bit.
    ///     0: RGMII/SGMII
    ///     1: 1000BASE-X
    ///
    /// Note: Register MACConfigAndFastLink.mac_source must be 0 for this
    /// selection to be valid.
    pub mac_mode_1000base_x, set_mac_mode_1000base_x: 12;
    /// AMS preference; Super-sticky bit.
    ///     0: Copper preferred
    ///     1: SerDes fiber/SFP preferred
    pub ams_preference, set_ams_preference: 11;
    /// Media operating mode; Super-sticky bits.
    ///     0: Copper only
    ///     1: SerDes fiber/SFP protocol transfer mode only
    ///     2: 1000BASE-X fiber/SFP media only with autonegotiation
    ///     3: 100BASE-FX fiber/SFP on fiber media pins only
    ///     4: Automatic Media Sense (AMS)
    ///     5: AMS with copper media or SerDes/SFP protocol transfer mode
    ///     6: AMS with copper media or 1000BASE-X fiber/SFP media with
    ///         autonegotiation
    ///     7: AMS with copper media or 100BASE-FX fiber/media
    pub media_mode, set_media_mode: 10, 8;
    /// Force override of AMS mode; Sticky bits.
    ///     0: Normal
    ///     1: SerDes media only
    ///     2: Copper media only
    pub force_ams_override, set_force_ams_override: 7, 6;
    /// Enable/disable far-end loopback. This enables/disables the receive pins
    /// on the twisted pair/fiber media interface.
    pub far_end_loopback, set_far_end_loopback: 3;
}

//
// Extended Page 1 Registers
//

bitfield::bitfield! {
    /// Contents of the Ethernet Packet Generator Control 1 register.
    pub struct EPGControl1(u16);
    /// Enable EPG: disables all MAC interface transmit pins and selects EPG
    /// as the source for all data transmitted to the copper/SerDes interfaces.
    pub enabled, enable: 15;
    /// EPG start/stop; start or stop the packet generator.
    pub running, set_running: 14;
    /// Transmission duration;
    ///     0: Send 30,000,000 packets and stop
    ///     1: Send continuous, in 10,000 packet increments
    pub duration, set_duration: 13;
    /// Packet length; select one of four packet sizes.
    ///     0: 125 bytes
    ///     1: 64 bytes
    ///     2: 1518 bytes
    ///     3: 10,000 bytes (jumbo packet)
    pub packet_len, set_packet_len: 12, 11;
    /// Interpacket gab; set the interpacket gap
    ///     0: 96 ns
    ///     1: 8,192 ns
    pub interpacket_gap, set_interpacket_gap: 10;
    /// Destination address; set the lowest nibble of the 6-byte destination
    /// MAC address.
    pub dest_address, set_dest_address: 9, 6;
    /// Source address; set the lowest nibble of the 6-byte source MAC address.
    pub src_address, set_src_address: 5, 2;
    /// Payload type; set the payload type.
    ///     0: Fixed payload pattern, set pattern using EPG Control 2 register
    ///     1: Random generated payload pattern
    pub payload_type, set_payload_type: 1;
    /// Bad FCS generation; generate packets with bad frame check sequence
    pub bad_fcs, set_bad_fcs: 0;
}

//
// General Purpose Registers
//

#[repr(u16)]
pub enum ProcessorCommands {
    Nop = 0x800f,
    EnableDualPortMACAsSGMII = 0x80f0,
    EnableDualPortMACAsQSGMII = 0x80e0,
    EnableDualPortMedia1000BaseX = 0x8fc1,
    EnableDualPortMedia100BaseFX = 0x83d1,
}

bitfield::bitfield! {
    /// Contents of the MAC Configuration And Fast Link register.
    pub struct MACConfigAndFastLink(u16);
    /// MAC source; select active the MAC input.
    ///     0: SGMII
    ///     1: QSGMII
    ///     2: RGMII
    pub mac_source, set_mac_source: 15, 14;
    /// Fast link failure port; select fast link failure PHY source.
    ///     0: Port0
    ///     1: Port1
    ///     2-3: Reserved
    ///     _: Output disabled
    pub fast_link_failure_source, set_fast_link_failure_source: 3, 0;
}

pub fn pre_init<'a>(port: &'a mut Port) {
    port.set_page(Pages::Main);
    // Enable SMI broadcast
    port.write_masked(Main::ExtendedControlAndStatus as u8, 0x0001, 0x0001);
    port.write(Main::ExtendedPHYControl2 as u8, 0x0040); // Reserved

    port.set_page(Pages::E2);
    port.write(E2::CuPMDTransmitControl as u8, 0x02be);

    // No idea what this is about. It's entirely undocumented.
    port.set_page(Pages::Test);
    port.write(20, 0x4320);
    port.write(24, 0x0c00);
    port.write(9, 0x18ca);
    port.write(5, 0x1b20);
    port.write_masked(8, 0x8000, 0x8000);

    port.set_page(Pages::TokenRing);
    port.write(18, 0x0004);
    port.write(17, 0x01bd);
    port.write(16, 0x8fae);
    port.write(18, 0x000f);
    port.write(17, 0x000f);
    port.write(16, 0x8fac);
    port.write(18, 0x00a0);
    port.write(17, 0xf147);
    port.write(16, 0x97a0);
    port.write(18, 0x0005);
    port.write(17, 0x2f54);
    port.write(16, 0x8fe4);
    port.write(18, 0x0027);
    port.write(17, 0x303d);
    port.write(16, 0x9792);
    port.write(18, 0x0000);
    port.write(17, 0x0704);
    port.write(16, 0x87fe);
    port.write(18, 0x0006);
    port.write(17, 0x0150);
    port.write(16, 0x8fe0);
    port.write(18, 0x0012);
    port.write(17, 0xb00a);
    port.write(16, 0x8f82);
    port.write(18, 0x0000);
    port.write(17, 0x0d74);
    port.write(16, 0x8f80);
    port.write(18, 0x0000);
    port.write(17, 0x0012);
    port.write(16, 0x82e0);
    port.write(18, 0x0005);
    port.write(17, 0x0208);
    port.write(16, 0x83a2);
    port.write(18, 0x0000);
    port.write(17, 0x9186);
    port.write(16, 0x83b2);
    port.write(18, 0x000e);
    port.write(17, 0x3700);
    port.write(16, 0x8fb0);
    port.write(18, 0x0004);
    port.write(17, 0x9f81);
    port.write(16, 0x9688);
    port.write(18, 0x0000);
    port.write(17, 0xffff);
    port.write(16, 0x8fd2);
    port.write(18, 0x0003);
    port.write(17, 0x9fa2);
    port.write(16, 0x968a);
    port.write(18, 0x0020);
    port.write(17, 0x640b);
    port.write(16, 0x9690);
    port.write(18, 0x0000);
    port.write(17, 0x2220);
    port.write(16, 0x8258);
    port.write(18, 0x0000);
    port.write(17, 0x2a20);
    port.write(16, 0x825a);
    port.write(18, 0x0000);
    port.write(17, 0x3060);
    port.write(16, 0x825c);
    port.write(18, 0x0000);
    port.write(17, 0x3fa0);
    port.write(16, 0x825e);
    port.write(18, 0x0000);
    port.write(17, 0xe0f0);
    port.write(16, 0x83a6);
    port.write(18, 0x0000);
    port.write(17, 0x1489);
    port.write(16, 0x8f92);
    port.write(18, 0x0000);
    port.write(17, 0x7000);
    port.write(16, 0x96a2);
    port.write(18, 0x0007);
    port.write(17, 0x1448);
    port.write(16, 0x96a6);
    port.write(18, 0x00ee);
    port.write(17, 0xffdd);
    port.write(16, 0x96a0);
    port.write(18, 0x0091);
    port.write(17, 0xb06c);
    port.write(16, 0x8fe8);
    port.write(18, 0x0004);
    port.write(17, 0x1600);
    port.write(16, 0x8fea);
    port.write(18, 0x00ee);
    port.write(17, 0xff00);
    port.write(16, 0x96b0);
    port.write(18, 0x0000);
    port.write(17, 0x7000);
    port.write(16, 0x96b2);
    port.write(18, 0x0000);
    port.write(17, 0x0814);
    port.write(16, 0x96b4);
    port.write(18, 0x0068);
    port.write(17, 0x8980);
    port.write(16, 0x8f90);
    port.write(18, 0x0000);
    port.write(17, 0xd8f0);
    port.write(16, 0x83a4);
    port.write(18, 0x0000);
    port.write(17, 0x0400);
    port.write(16, 0x8fc0);
    port.write(18, 0x0050);
    port.write(17, 0x100f);
    port.write(16, 0x87fa);
    port.write(18, 0x0000);
    port.write(17, 0x0003);
    port.write(16, 0x8796);
    port.write(18, 0x00c3);
    port.write(17, 0xff98);
    port.write(16, 0x87f8);
    port.write(18, 0x0018);
    port.write(17, 0x292a);
    port.write(16, 0x8fa4);
    port.write(18, 0x00d2);
    port.write(17, 0xc46f);
    port.write(16, 0x968c);
    port.write(18, 0x0000);
    port.write(17, 0x0620);
    port.write(16, 0x97a2);
    port.write(18, 0x0013);
    port.write(17, 0x132f);
    port.write(16, 0x96a4);
    port.write(18, 0x0000);
    port.write(17, 0x0000);
    port.write(16, 0x96a8);
    port.write(18, 0x00c0);
    port.write(17, 0xa028);
    port.write(16, 0x8ffc);
    port.write(18, 0x0090);
    port.write(17, 0x1c09);
    port.write(16, 0x8fec);
    port.write(18, 0x0004);
    port.write(17, 0xa6a1);
    port.write(16, 0x8fee);
    port.write(18, 0x00b0);
    port.write(17, 0x1807);
    port.write(16, 0x8ffe);
    // ifndef VTSS_10BASE_TE
    port.set_page(Pages::E2);
    port.write(16, 0x028e);

    port.set_page(Pages::TokenRing);
    port.write(18, 0x0008);
    port.write(17, 0xa518);
    port.write(16, 0x8486);
    port.write(18, 0x006d);
    port.write(17, 0xc696);
    port.write(16, 0x8488);
    port.write(18, 0x0000);
    port.write(17, 0x0912);
    port.write(16, 0x848a);
    port.write(18, 0x0000);
    port.write(17, 0x0db6);
    port.write(16, 0x848e);
    port.write(18, 0x0059);
    port.write(17, 0x6596);
    port.write(16, 0x849c);
    port.write(18, 0x0000);
    port.write(17, 0x0514);
    port.write(16, 0x849e);
    port.write(18, 0x0041);
    port.write(17, 0x0280);
    port.write(16, 0x84a2);
    port.write(18, 0x0000);
    port.write(17, 0x0000);
    port.write(16, 0x84a4);
    port.write(18, 0x0000);
    port.write(17, 0x0000);
    port.write(16, 0x84a6);
    port.write(18, 0x0000);
    port.write(17, 0x0000);
    port.write(16, 0x84a8);
    port.write(18, 0x0000);
    port.write(17, 0x0000);
    port.write(16, 0x84aa);
    port.write(18, 0x007d);
    port.write(17, 0xf7dd);
    port.write(16, 0x84ae);
    port.write(18, 0x006d);
    port.write(17, 0x95d4);
    port.write(16, 0x84b0);
    port.write(18, 0x0049);
    port.write(17, 0x2410);
    port.write(16, 0x84b2);
    // endif

    port.set_page(Pages::Test);
    port.write_masked(8, 0x0000, 0x8000);

    port.set_page(Pages::Main);
    // Disable SMI broadcast
    port.write_masked(Main::ExtendedControlAndStatus as u8, 0x0000, 0x0001);

    /*
    #if defined (MICRO_PATCH_REV_TS_FIFO_2)
        VTSS_RC(tesla_revD_8051_patch(vtss_state, port_no)); // Rev D. This is for RevD Only
    #else
        VTSS_RC(tesla_revB_8051_patch(vtss_state, port_no)); // This is the OLD-Patch (Non-Middle-Man), where rev B, C, & D have the same patch.
    #endif

        VTSS_RC(vtss_phy_pre_init_tesla_revB_1588(vtss_state, port_no)); //Init 1588 register using Tesla RevB micro patch
    */

    //return VTSS_RC_OK;
}

pub fn assert_mcu_reset<'a>(port: &'a mut Port) {
    port.set_page(Pages::G);
    // Pass the NOP cmd to Micro to insure that any consumptive patch exits There is no issue with
    // doing this on any revision since it is just a NOP on any Vitesse PHY.
    execute_mcu_command(port, ProcessorCommands::Nop);

    // Force MCU into a loop, preventing any SMI access. These writes are all in undocumented
    // registers. Comments are taken from the PHY API.

    // Disable patch vector 3 (just in case)
    port.write_masked(12, 0x0000, 0x0800);
    // Setup patch vector 3 to trap MicroWake interrupt
    port.write(9, 0x005b);
    // Loop forever on MicroWake interrupts
    port.write(10, 0x005b);
    // Enable patch vector 3
    port.write_masked(12, 0x0800, 0x0800);
    // Trigger MicroWake interrupt to allow safe reset
    port.write(G::ProcessorCommand as u8, ProcessorCommands::Nop as u16);

    // Assert reset after MCU is trapped in a loop (averts MCU-SMI access deadlock on reset)
    port.write_masked(0, 0x0000, 0x8000);
    // Make sure no MicroWake persists after reset
    port.write(G::ProcessorCommand as u8, 0x0000);
    // Disable patch vector 3
    port.write_masked(12, 0x0000, 0x0800);
}

pub fn start_mcu<'a>(port: &'a mut Port) {
    port.set_page(Pages::G);

    // Trap ROM at _MicroSmiRead+0x1d to spoof patch-presence
    port.write(3, 0x3eb7);
    // Branch to starting address of SpoofPatchPresence
    port.write(4, 0x4012);
    // Enable patch from trap described in register 3-4
    port.write(12, 0x0100);
    // Enable 8051 clock; Clear patch present; Disable PRAM clock override and addr. auto-incr;
    // Operate at 125 MHz
    port.write(0, 0x4018);
    // Release 8051 SW Reset
    port.write(0, 0xc018);
}

pub fn execute_mcu_command<'a>(port: &'a mut Port, cmd: ProcessorCommands) {
    port.write(G::ProcessorCommand as u8, cmd as u16);

    while (port.read(G::ProcessorCommand as u8) & 0x8000) != 0 {
        hl::sleep_for(10);
    }
}

pub fn init<'a>(port: &'a mut Port) {
    port.set_page(Pages::G);

    let mut g_19 = MACConfigAndFastLink(0);
    g_19.set_mac_source(0); // SGMII
    g_19.set_fast_link_failure_source(0); // Port 0

    port.write(G::MACConfigAndFastLink as u8, g_19.0);

    execute_mcu_command(port, ProcessorCommands::EnableDualPortMACAsSGMII);
    execute_mcu_command(port, ProcessorCommands::EnableDualPortMedia100BaseFX);

    port.set_page(Pages::Main);

    // Update Extended PHY Control 1 to match the above and set 100BASE-FX
    // link partner.
    let mut main_23 = ExtendedPHYControl1(0);
    main_23.set_mac_mode_1000base_x(false); // RGMII/SGMII mode
    main_23.set_media_mode(3); // 100BASE-FX fiber only

    port.write(Main::ExtendedPHYControl1 as u8, main_23.0);
}

pub fn soft_reset<'a>(port: &'a mut Port) {
    port.set_page(Pages::Main);
    port.write_masked(Main::ModeControl as u8, 0x8000, 0x8000);

    while (port.read(Main::ModeControl as u8) & 0x8000) != 0 {
        hl::sleep_for(10);
    }
}
