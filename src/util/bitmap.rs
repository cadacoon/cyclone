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

pub type BitmapType = usize;

pub struct Bitmap([BitmapType; (1024 * 1024) / 32]);

unsafe impl Send for Bitmap {}

impl Bitmap {
    pub const fn new() -> Self {
        Self([0; (1024 * 1024) / 32])
    }

    pub fn consecutive_zeros(&mut self, fits: usize) -> ConsecutiveZeros {
        assert!(fits > 0);
        ConsecutiveZeros {
            block: self.0[0],
            bitmap: self,
            block_index: 0,
            index: 0,
            fits,
        }
    }

    pub fn set_ones<R: ops::RangeBounds<usize>>(&mut self, range: R) {
        for (block, mask) in Masks::new(range, BitmapType::BITS as usize * self.0.len()) {
            self.0[block] |= mask;
        }
    }

    pub fn set_zeros<R: ops::RangeBounds<usize>>(&mut self, range: R) {
        for (block, mask) in Masks::new(range, BitmapType::BITS as usize * self.0.len()) {
            self.0[block] &= !mask;
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

struct Masks {
    first_block: usize,
    first_mask: BitmapType,
    last_block: usize,
    last_mask: BitmapType,
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

        let first_block = start / BitmapType::BITS as usize;
        let first_mask = BitmapType::MAX << (start % BitmapType::BITS as usize);
        let last_block = end / BitmapType::BITS as usize;
        let last_mask = (BitmapType::MAX >> 1)
            >> (BitmapType::BITS - (end % BitmapType::BITS as usize) as u32 - 1);

        Self {
            first_block,
            first_mask,
            last_mask,
            last_block,
        }
    }
}

impl Iterator for Masks {
    type Item = (usize, BitmapType);

    fn next(&mut self) -> Option<Self::Item> {
        match self.first_block.cmp(&self.last_block) {
            cmp::Ordering::Less => {
                let block = self.first_block;
                let mask = self.first_mask;
                self.first_block += 1;
                self.first_mask = !0;
                Some((block, mask))
            }
            cmp::Ordering::Equal => {
                let block = self.first_block;
                let mask = self.first_mask & self.last_mask;
                self.first_block += 1;
                if mask == 0 {
                    None
                } else {
                    Some((block, mask))
                }
            }
            cmp::Ordering::Greater => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.first_block..=self.last_block).size_hint()
    }
}

pub struct ConsecutiveZeros<'a> {
    bitmap: &'a mut Bitmap,
    block_index: usize,
    block: usize,
    index: usize,
    fits: usize,
}

impl<'a> ConsecutiveZeros<'a> {
    pub fn set_ones<R: ops::RangeBounds<usize>>(&mut self, range: R) {
        self.bitmap.set_ones(range);
    }
}

impl<'a> Iterator for ConsecutiveZeros<'a> {
    type Item = ops::Range<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.block_index < self.bitmap.0.len() {
            if self.block == 0 {
                let index = self.index;
                let next_index = (self.block_index + 1) * BitmapType::BITS as usize;
                if next_index - index >= self.fits {
                    self.index = next_index;
                    self.block_index += 1;
                    self.block = *self.bitmap.0.get(self.block_index).unwrap_or(&0);
                    return Some(index..next_index);
                }
            }
            while self.block != 0 {
                let index = self.index;
                let next_index = self.block_index * (BitmapType::BITS as usize)
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
