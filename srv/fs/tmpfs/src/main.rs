#![no_std]
#![no_main]

use core::{hint, panic};

/*
Go through FileSystem<TMP> IPC queue
*/
fn main() {}

#[panic_handler]
fn panic(_info: &panic::PanicInfo) -> ! {
    loop {
        hint::spin_loop();
    }
}
