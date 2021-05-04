#![no_std]
#![no_main]

use ringbuf::*;
use userlib::*;

pub mod ksz8463;

#[cfg(feature = "standalone")]
const SPI: Task = Task::anonymous;
#[cfg(not(feature = "standalone"))]
const SPI: Task = Task::spi_driver;

#[export_name = "main"]
fn main() -> ! {
    let spi = TaskId::for_index_and_gen(SPI as usize, Generation::default());

    loop {
        ksz8463::read(spi, ksz8463::Register::CIDER).unwrap();
        ksz8463::write(spi, ksz8463::Register::CIDER, 1).unwrap();
        ksz8463::read(spi, ksz8463::Register::CIDER).unwrap();

        hl::sleep_for(1000);
    }
}
