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

use core::ops::Range;

pub struct Device {
    pub class: [u8; 4],
    pub class_vendor: [u16; 2],
    pub resource: [Resource; 6],
}

pub enum Resource {
    None,
    Pio(Range<u16>),
    Mem16(Range<u16>),
    Mem32(Range<u32>),
    Mem64(Range<u64>),
}
