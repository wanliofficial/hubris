#![no_std]
#![no_main]

use userlib::*;

use core::convert::TryInto;

use cortex_m_semihosting::hprintln;

#[export_name = "main"]
pub fn main() -> ! {
    hprintln!("sup2: starting!").ok();

    const TIMER_NOTIFICATION: u32 = 1;

    let mut msg = [0; 16];

    loop {
        let msg_info = sys_recv_open(&mut msg, TIMER_NOTIFICATION);

        let message = &msg[..msg_info.message_len];

        if msg_info.sender != TaskId::KERNEL {
            match msg_info.operation {
                2 => square(msg_info, message),
                _ => panic!("Unknown operation!"),
            };
        } else {
            sys_reply(msg_info.sender, 0, &[]);
        }
    }
}


fn square(msg_info: RecvMessage, message: &[u8]) {
    hprintln!("got square message").ok();

    let i = u32::from_le_bytes(message.try_into().unwrap());

    // overflow? what's overflow?
    let result = i * i;

    sys_reply(msg_info.sender, 0, &result.to_le_bytes());
}
