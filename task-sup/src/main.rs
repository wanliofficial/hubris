#![no_std]
#![no_main]

use userlib::*;

use cortex_m_semihosting::hprintln;
use core::convert::TryInto;

const SUP2: Task = Task::sup2;

#[export_name = "main"]
fn main() -> ! {
    hprintln!("sup1: starting!").ok();

    let target = TaskId::for_index_and_gen(SUP2 as usize, Generation::default());

    let mut response = [0; 32];

    let mut i: u32 = 2;

    loop {
        /*
        hprintln!("sup1: asking sup2 what's up").ok();

        let (_code, len) =
            sys_send(sup2, op, b"what's up", &mut response, &[]);

        hprintln!("sup1: got response: '{}'", core::str::from_utf8(&response[..len]).unwrap()).ok();
        */

        hprintln!("sup1: asking sup2 what the square of {} is", i).ok();

        let mut answer = [0; 4];

        let operation = 2;
        let outgoing = &i.to_le_bytes();
        let incoming = &mut response;
        let leases = &[Lease::from(&mut answer[..])];

        let (_code, len) = sys_send(target, operation, outgoing, incoming, leases);

        let num = &incoming[..len];
        let num = u32::from_le_bytes(num.try_into().unwrap());

        hprintln!("sup1: sup2 says the square is: {}", num).ok();

        i += 1;
    }
}
