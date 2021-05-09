/// A trait to allow abstraction of reading from and writing to an MDIO device.
pub trait Controller {
    fn read(&mut self, phy_address: u8, register_address: u8) -> u16;
    fn write(&mut self, phy_address: u8, register_address: u8, value: u16);
    fn write_masked(
        &mut self,
        phy_address: u8,
        register_address: u8,
        value: u16,
        mask: u16,
    );
}
