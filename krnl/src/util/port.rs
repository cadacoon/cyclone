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

use core::{arch, marker};

use volatile::access::{Readable, Writable};

#[derive(Clone, Copy)]
pub struct Port<T: PortType, A = volatile::access::ReadWrite> {
    port: u16,
    r#type: marker::PhantomData<T>,
    access: marker::PhantomData<A>,
}

impl<T, A> !Sync for Port<T, A> {}

impl<T: PortType> Port<T> {
    pub const unsafe fn new<A>(port: u16) -> Port<T, A> {
        Port {
            port,
            r#type: marker::PhantomData,
            access: marker::PhantomData,
        }
    }
}

impl<T: PortType, A> Port<T, A> {
    pub fn read(&mut self) -> T
    where
        A: Readable,
    {
        unsafe { T::read_port(self.port) }
    }

    pub fn write(&mut self, value: T)
    where
        A: Writable,
    {
        unsafe { T::write_port(self.port, value) }
    }
}

pub trait PortType {
    unsafe fn read_port(port: u16) -> Self;

    unsafe fn write_port(port: u16, value: Self);
}

impl PortType for u8 {
    unsafe fn read_port(port: u16) -> Self {
        let value;
        arch::asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags));
        value
    }

    unsafe fn write_port(port: u16, value: Self) {
        arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }
}

impl PortType for u16 {
    unsafe fn read_port(port: u16) -> Self {
        let value;
        arch::asm!("in ax, dx", out("ax") value, in("dx") port, options(nomem, nostack, preserves_flags));
        value
    }

    unsafe fn write_port(port: u16, value: Self) {
        arch::asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
    }
}

impl PortType for u32 {
    unsafe fn read_port(port: u16) -> Self {
        let value;
        arch::asm!("in eax, dx", out("eax") value, in("dx") port, options(nomem, nostack, preserves_flags));
        value
    }

    unsafe fn write_port(port: u16, value: Self) {
        arch::asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack, preserves_flags));
    }
}
