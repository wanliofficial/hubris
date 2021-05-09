use userlib::*;

#[repr(u16)]
#[derive(Copy, Clone, Debug, FromPrimitive, PartialEq)]
pub enum Pages {
    Main = 0,
    E1 = 1,
    E2 = 2,
    E3 = 3,
    G = 8,
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
