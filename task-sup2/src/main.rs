#![no_std]
#![no_main]

use userlib::*;

use cortex_m_semihosting::hprintln;

#[export_name = "main"]
pub fn main() -> ! {
    hprintln!("sup2: starting!").ok();

    const TIMER_NOTIFICATION: u32 = 1;
    const SUCCESS_RESPONSE: u32 = 0;

    let mut msg = [0; 16];

    loop {
        let msginfo = sys_recv_open(&mut msg, TIMER_NOTIFICATION);

        let msg = &msg[..msginfo.message_len];

        hprintln!("sup2: got message: '{}'", core::str::from_utf8(msg).unwrap()).ok();

        if msginfo.sender != TaskId::KERNEL {
            hprintln!("sup2: responding").ok();
            sys_reply(msginfo.sender, SUCCESS_RESPONSE, b"nm whats up with you");
        } else {
            sys_reply(msginfo.sender, SUCCESS_RESPONSE, &[]);
        }
    }
}
