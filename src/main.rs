// Copyright 2024 Kevin Ludwig
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![no_std]
#![no_main]

extern crate alloc;

pub mod mm;
pub mod sm;
pub mod util;

#[global_allocator]
static ALLOCATOR: mm::VirtualMemoryScope = mm::VirtualMemoryScope;

core::arch::global_asm!(include_str!("start.S"), options(att_syntax));

#[no_mangle]
unsafe fn main(multiboot_magic: u32, multiboot_info: &mut multiboot::multiboot_info) -> ! {
    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    loop {}
}

mod multiboot {
    include!(concat!(env!("OUT_DIR"), "/multiboot.rs"));
}
