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

use core::{cmp, fmt, ops};

use alloc::boxed::Box;

pub type Block = usize;
pub struct Bitmap(Box<[Block]>);

impl Bitmap {
    /// Creates a new bitmap
    pub const fn new(value: Box<[Block]>) -> Self {
        Self(value)
    }

    /// Creates an iterator which returns sequences of consecutive zeros
    pub fn consecutive_zeros(&self, fits: usize) -> ConsecutiveZeros {
        assert!(fits > 0);

        ConsecutiveZeros {
            bitmap: self,
            block_index: 0,
            block: self.0[0],
            index: 0,
            fits,
        }
    }

    /// Sets the given range to zero
    pub fn set_zeros<R: ops::RangeBounds<usize>>(&mut self, range: R) {
        for (block, mask) in Masks::new(range, Block::BITS as usize * self.0.len()) {
            self.0[block] &= !mask;
        }
    }

    /// Sets the given range to one
    pub fn set_ones<R: ops::RangeBounds<usize>>(&mut self, range: R) {
        for (block, mask) in Masks::new(range, Block::BITS as usize * self.0.len()) {
            self.0[block] |= mask;
        }
    }
}

impl fmt::Debug for Bitmap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for block in &self.0 {
            let bytes = block.to_le_bytes();
            for byte in bytes {
                let byte = byte.reverse_bits();
                write!(f, "{:02X}", byte)?;
            }
            write!(f, " ")?;
        }
        Ok(())
    }
}

pub struct ConsecutiveZeros<'owner> {
    bitmap: &'owner Bitmap,
    block_index: usize,
    block: Block,
    index: usize,
    fits: usize,
}

impl<'a> Iterator for ConsecutiveZeros<'a> {
    type Item = ops::Range<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.block_index < self.bitmap.0.len() {
            if self.block == 0 {
                let index = self.index;
                let next_index = (self.block_index + 1) * Block::BITS as usize;
                if next_index - index >= self.fits {
                    self.index = next_index;
                    self.block_index += 1;
                    self.block = *self.bitmap.0.get(self.block_index).unwrap_or(&0);
                    return Some(index..next_index);
                }
            }
            while self.block != 0 {
                let index = self.index;
                let next_index = self.block_index * (Block::BITS as usize)
                    + self.block.trailing_zeros() as usize;
                self.index = next_index + 1;
                self.block ^= self.block & self.block.wrapping_neg();
                if next_index - index >= self.fits {
                    return Some(index..next_index);
                }
            }
            let index = self.index;
            let next_index = self.index + self.bitmap.0[self.block_index].leading_zeros() as usize;
            if next_index - index >= self.fits {
                self.index = next_index;
                self.block_index += 1;
                self.block = *self.bitmap.0.get(self.block_index).unwrap_or(&0);
                return Some(index..next_index);
            }
            self.block_index += 1;
            self.block = *self.bitmap.0.get(self.block_index).unwrap_or(&0);
        }

        None
    }
}

struct Masks {
    first_index: usize,
    first_mask: Block,
    last_index: usize,
    last_mask: Block,
}

impl Masks {
    fn new<T: ops::RangeBounds<usize>>(range: T, length: usize) -> Self {
        let start = match range.start_bound() {
            ops::Bound::Included(value) => *value,
            ops::Bound::Excluded(value) => *value + 1,
            ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            ops::Bound::Included(value) => *value + 1,
            ops::Bound::Excluded(value) => *value,
            ops::Bound::Unbounded => length,
        };
        assert!(end > start);
        assert!(end <= length);
        Self {
            first_index: start / Block::BITS as usize,
            first_mask: Block::MAX << (start as u32 % Block::BITS),
            last_index: end / Block::BITS as usize,
            last_mask: (Block::MAX >> 1) >> (Block::BITS - end as u32 % Block::BITS - 1),
        }
    }
}

impl Iterator for Masks {
    type Item = (usize, Block);

    fn next(&mut self) -> Option<Self::Item> {
        match self.first_index.cmp(&self.last_index) {
            cmp::Ordering::Less => {
                let index = self.first_index;
                let mask = self.first_mask;
                self.first_index += 1;
                self.first_mask = !0;
                Some((index, mask))
            }
            cmp::Ordering::Equal => {
                let index = self.first_index;
                let mask = self.first_mask & self.last_mask;
                self.first_index += 1;
                if mask == 0 {
                    None
                } else {
                    Some((index, mask))
                }
            }
            cmp::Ordering::Greater => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.first_index..=self.last_index).size_hint()
    }
}
