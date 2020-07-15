#![no_std]
#![no_main]

use userlib::*;

use cortex_m_semihosting::hprintln;

const SUP2: Task = Task::sup2;

#[export_name = "main"]
fn main() -> ! {
    hprintln!("sup1: starting!").ok();

    let sup2 = TaskId::for_index_and_gen(SUP2 as usize, Generation::default());

    const PING_OP: u16 = 1;

    let mut response = [0; 32];
    loop {

        hprintln!("sup1: asking sup2 what's up").ok();
        let (_code, len) =
            sys_send(sup2, PING_OP, b"what's up", &mut response, &[]);
        
        hprintln!("sup1: got response: '{}'", core::str::from_utf8(&response[..len]).unwrap()).ok();
    }
}
