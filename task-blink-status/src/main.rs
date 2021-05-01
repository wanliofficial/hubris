#![no_std]
#![no_main]

use userlib::*;

#[export_name = "main"]
pub fn main() -> ! {
    const TIMER_NOTIFICATION: u32 = 1;
    const INTERVAL: u64 = 500;

    let user_leds = get_user_leds();
    let mut dl = INTERVAL;
    let mut msg = [0; 16];

    sys_set_timer(Some(dl), TIMER_NOTIFICATION);

    loop {
        let msginfo = sys_recv_open(&mut msg, TIMER_NOTIFICATION);

        match msginfo.sender {
            TaskId::KERNEL => {
                dl += INTERVAL;
                sys_set_timer(Some(dl), TIMER_NOTIFICATION);
                user_leds.led_toggle(3).unwrap();
            }
            _ => {
                panic!("unexpected sender");
            }
        }
    }
}

fn get_user_leds() -> drv_user_leds_api::UserLeds {
    #[cfg(not(feature = "standalone"))]
    const USER_LEDS: Task = Task::user_leds;

    #[cfg(feature = "standalone")]
    const USER_LEDS: Task = Task::anonymous;

    drv_user_leds_api::UserLeds::from(TaskId::for_index_and_gen(
        USER_LEDS as usize,
        Generation::default(),
    ))
}
