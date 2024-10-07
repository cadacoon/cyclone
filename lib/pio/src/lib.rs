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

use core::{arch, marker};

use volatile::access::{Readable, Writable};

pub use volatile::access::{ReadOnly, ReadWrite, WriteOnly};

/// Allows port reads and writes.
///
/// Since not all ports are both readable and writable, this type supports
/// limiting the allowed access types through an optional second generic
/// parameter `A` that can be one of `ReadWrite`, `ReadOnly`, or `WriteOnly`. It
/// defaults to `ReadWrite`, which allows all operations.
#[must_use]
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Port<T: PortType, A = ReadWrite> {
    port: u16,
    r#type: marker::PhantomData<T>,
    access: marker::PhantomData<A>,
}

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
    /// Performs a read on the contained port.
    pub fn read(self) -> T
    where
        A: Readable,
    {
        unsafe { T::read(self.port) }
    }

    /// Performs as many reads on the contained port as needed to fill the
    /// slice.
    pub fn read_slice(self, slice: &mut [T])
    where
        A: Readable,
    {
        unsafe { T::read_slice(self.port, slice) }
    }

    /// Performs a write on the contained port.
    pub fn write(self, value: T)
    where
        A: Writable,
    {
        unsafe { T::write(self.port, value) }
    }

    /// Performs as many writes on the contained port as needed to empty the
    /// slice.
    pub fn write_slice(self, slice: &[T])
    where
        A: Writable,
    {
        unsafe { T::write_slice(self.port, slice) }
    }
}

pub trait PortType: Sized {
    unsafe fn read(port: u16) -> Self;

    unsafe fn read_slice(port: u16, slice: &mut [Self]);

    unsafe fn write(port: u16, value: Self);

    unsafe fn write_slice(port: u16, slice: &[Self]);
}

impl PortType for u8 {
    unsafe fn read(port: u16) -> Self {
        let value;
        arch::asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags));
        value
    }

    unsafe fn read_slice(port: u16, slice: &mut [Self]) {
        #[cfg(target_arch = "x86")]
        {
            arch::asm!(
                "rep insb",
                in("edi") slice.as_mut_ptr(),
                in("ecx") slice.len() * size_of::<Self>(),
                in("dx") port,
                options(nostack, preserves_flags)
            );
        }
        #[cfg(target_arch = "x86_64")]
        {
            arch::asm!(
                "rep insb",
                in("rdi") slice.as_mut_ptr(),
                in("rcx") slice.len() * size_of::<Self>(),
                in("dx") port,
                options(nostack, preserves_flags)
            );
        }
    }

    unsafe fn write(port: u16, value: Self) {
        arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }

    unsafe fn write_slice(port: u16, slice: &[Self]) {
        #[cfg(target_arch = "x86")]
        {
            arch::asm!(
                "xchg esi, edi",
                "rep outsb",
                "mov esi, edi",
                in("edi") slice.as_ptr(),
                in("ecx") slice.len() * size_of::<Self>(),
                in("dx") port,
                options(readonly, nostack, preserves_flags)
            );
        }
        #[cfg(target_arch = "x86_64")]
        {
            arch::asm!(
                "rep outsb",
                in("rsi") slice.as_ptr(),
                in("rcx") slice.len() * size_of::<Self>(),
                in("dx") port,
                options(readonly, nostack, preserves_flags)
            );
        }
    }
}

impl PortType for u16 {
    unsafe fn read(port: u16) -> Self {
        let value;
        arch::asm!("in ax, dx", out("ax") value, in("dx") port, options(nomem, nostack, preserves_flags));
        value
    }

    unsafe fn read_slice(port: u16, slice: &mut [Self]) {
        #[cfg(target_arch = "x86")]
        {
            arch::asm!(
                "rep insw",
                in("edi") slice.as_mut_ptr(),
                in("ecx") slice.len() * size_of::<Self>(),
                in("dx") port,
                options(nostack, preserves_flags)
            );
        }
        #[cfg(target_arch = "x86_64")]
        {
            arch::asm!(
                "rep insw",
                in("rdi") slice.as_mut_ptr(),
                in("rcx") slice.len() * size_of::<Self>(),
                in("dx") port,
                options(nostack, preserves_flags)
            );
        }
    }

    unsafe fn write(port: u16, value: Self) {
        arch::asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
    }

    unsafe fn write_slice(port: u16, slice: &[Self]) {
        #[cfg(target_arch = "x86")]
        {
            arch::asm!(
                "xchg esi, edi",
                "rep outsw",
                "mov esi, edi",
                in("edi") slice.as_ptr(),
                in("ecx") slice.len() * size_of::<Self>(),
                in("dx") port,
                options(readonly, nostack, preserves_flags)
            );
        }
        #[cfg(target_arch = "x86_64")]
        {
            arch::asm!(
                "rep outsw",
                in("rsi") slice.as_ptr(),
                in("rcx") slice.len() * size_of::<Self>(),
                in("dx") port,
                options(readonly, nostack, preserves_flags)
            );
        }
    }
}

impl PortType for u32 {
    unsafe fn read(port: u16) -> Self {
        let value;
        arch::asm!("in eax, dx", out("eax") value, in("dx") port, options(nomem, nostack, preserves_flags));
        value
    }

    unsafe fn read_slice(port: u16, slice: &mut [Self]) {
        #[cfg(target_arch = "x86")]
        {
            arch::asm!(
                "rep insd",
                in("edi") slice.as_mut_ptr(),
                in("ecx") slice.len() * size_of::<Self>(),
                in("dx") port, options(nostack, preserves_flags)
            );
        }
        #[cfg(target_arch = "x86_64")]
        {
            arch::asm!(
                "rep insd",
                in("rdi") slice.as_mut_ptr(),
                in("rcx") slice.len() * size_of::<Self>(),
                in("dx") port, options(nostack, preserves_flags)
            );
        }
    }

    unsafe fn write(port: u16, value: Self) {
        arch::asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack, preserves_flags));
    }

    unsafe fn write_slice(port: u16, slice: &[Self]) {
        #[cfg(target_arch = "x86")]
        {
            arch::asm!(
                "xchg esi, edi",
                "rep outsd",
                "mov esi, edi",
                in("edi") slice.as_ptr(),
                in("ecx") slice.len() * size_of::<Self>(),
                in("dx") port,
                options(readonly, nostack, preserves_flags)
            );
        }
        #[cfg(target_arch = "x86_64")]
        {
            arch::asm!(
                "rep outsd",
                in("rsi") slice.as_ptr(),
                in("rcx") slice.len() * size_of::<Self>(),
                in("dx") port,
                options(readonly, nostack, preserves_flags)
            );
        }
    }
}
