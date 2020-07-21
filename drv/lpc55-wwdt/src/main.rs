#![no_std]
#![no_main]

use lpc55_pac as device;

use userlib::*;
use cortex_m_semihosting::hprintln;

use drv_lpc55_syscon_api::{Peripheral, Syscon};

#[cfg(feature = "standalone")]
const SYSCON: Task = SELF;

#[cfg(not(feature = "standalone"))]
const SYSCON: Task = Task::syscon_driver;


#[export_name = "main"]
fn main() -> ! {
    let syscon = TaskId::for_index_and_gen(SYSCON as usize, Generation::default());
    let syscon = Syscon::from(syscon);

    hprintln!("Starting wwdt!").ok();

    let wwdt = unsafe { &*device::WWDT::ptr() };


    syscon.configure_wwdt(Peripheral::Wwdt);

    //syscon.enter_reset(Peripheral::Wwdt);
    syscon.leave_reset(Peripheral::Wwdt);

    let wdtof = wwdt.mod_.read().wdtof().bits();
    hprintln!("wdtof: {}", wdtof).ok();

    // reset wdtof to zero so that if it's one on next boot, we know it's the wwdt that did it
    wwdt.mod_.write(|w| w.wdtof().bit(false));

    // tc is the "timer constant," aka, where we start counting down from. It's 24-bit.
    wwdt.tc.write(|w| unsafe { w.bits(0x00_3FFF) });

    // enable, allow it to reset the board, 
    wwdt.mod_.write(|w| 
        w.wden().run()
            .wdreset().reset()
            .wdint().set_bit()
    );

    /* reset value is already 0xFF_FFFF
    // set windowing to max, since we don't intend to use it
    wwdt.window.write(|w| unsafe {
        w.window().bits(0xFF_FFFF)
    });
    */

    /* reset value is already 0
    // set the interrupt warning value to zero, since we don't intend to use it
    wwdt.warnint.write(|w| unsafe {
        w.warnint().bits(0x0)
    });
    */

    // Feed the watchdog
    wwdt.feed.write(|w|
        unsafe {
            w.feed()
                .bits(0xAA)
        }
    );

    wwdt.feed.write(|w|
        unsafe {
            w.feed()
                .bits(0x55)
        }
    );

    // we only need to sleep for three cycles, but we don't have the ability to
    // do that right now. One tick is more than enough, and not a big deal that
    // it's not accurate.
    hl::sleep_for(1);

    // after waiting, protect the timer by setting the right bit
    //wwdt.mod_.write(|w| w.wdprotect().set_bit());

    let mut counter = 0;

    loop {
        hprintln!("wwdt loop start!").ok();
        let tv = wwdt.tv.read().count().bits();
        hprintln!("tv: {:?}", tv).ok();
        hprintln!("window: {:?}", wwdt.window.read().window().bits()).ok();

        hl::sleep_for(1);
        if counter > 4 {
            hprintln!("no feeding!").ok();
        } else {
            hprintln!("feeding!").ok();
            counter += 1;

            // Feed the watchdog
            wwdt.feed.write(|w|
                unsafe {
                    w.feed()
                        .bits(0xAA)
                }
            );

            wwdt.feed.write(|w|
                unsafe {
                    w.feed()
                        .bits(0x55)
                }
            );
            hprintln!("sleeping more: {}", counter).ok();
        }
    }
}
