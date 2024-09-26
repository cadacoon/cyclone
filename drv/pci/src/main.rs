#![no_std]
#![no_main]

use core::{hint, panic};

/*
Enumerate PCI devices
*/
fn main() {}

#[panic_handler]
fn panic(_info: &panic::PanicInfo) -> ! {
    loop {
        hint::spin_loop();
    }
}
