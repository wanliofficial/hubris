#![no_std]
#![no_main]

use userlib::*;

pub mod ksz8463;

#[cfg(feature = "standalone")]
const SPI: Task = Task::anonymous;
#[cfg(not(feature = "standalone"))]
const SPI: Task = Task::spi_driver;

#[export_name = "main"]
fn main() -> ! {
    let spi = TaskId::for_index_and_gen(SPI as usize, Generation::default());

    ksz8463::enable(spi).unwrap();
    // Set 100BASE-FX
    ksz8463::write_masked(spi, ksz8463::Register::CFGR, 0x0, 0xc0);
    ksz8463::write_masked(spi, ksz8463::Register::DSP_CNTRL_6, 0, 0x2000);

    ksz8463::read(spi, ksz8463::Register::P1MBCR).unwrap();

    loop {
        ksz8463::read(spi, ksz8463::Register::P1MBSR).unwrap();
        hl::sleep_for(1000);
    }
}
