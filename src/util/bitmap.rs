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

type BitmapType = usize;

pub struct Bitmap(Box<[BitmapType]>);

unsafe impl Send for Bitmap {}

impl Bitmap {
    pub const fn empty() -> Self {
        Self(unsafe {
            std::mem::transmute(std::ptr::slice_from_raw_parts(
                std::ptr::NonNull::<BitmapType>::dangling().as_ptr() as *const BitmapType,
                0,
            ))
        })
    }

    pub fn new(value: Box<[BitmapType]>) -> Self {
        Self(value)
    }

    pub fn find_zeros(&self, length: usize) -> Option<std::ops::Range<usize>> {
        let mut last_one = 0;
        for (block_index, block) in self.0.iter().enumerate() {
            let mut block_tmp = *block;
            if block_tmp == 0 {
                let next_one = (block_index + 1) * BitmapType::BITS as usize;
                if next_one - last_one >= length {
                    return Some(last_one..next_one);
                }
            }
            while block_tmp != 0 {
                let next_one =
                    block_index * (BitmapType::BITS as usize) + block_tmp.trailing_zeros() as usize;
                if next_one - last_one >= length {
                    return Some(last_one..next_one);
                }
                last_one = next_one + 1;
                block_tmp ^= block_tmp & block_tmp.wrapping_neg();
            }
            let next_one = last_one + block.leading_zeros() as usize;
            if next_one - last_one >= length {
                return Some(last_one..next_one);
            }
        }

        None
    }

    pub fn set_ones<R: std::ops::RangeBounds<usize>>(&mut self, range: R) {
        for (block, mask) in BitmapMasks::new(range, self.0.len()) {
            self.0[block] |= mask;
        }
    }

    pub fn set_zeros<R: std::ops::RangeBounds<usize>>(&mut self, range: R) {
        for (block, mask) in BitmapMasks::new(range, self.0.len()) {
            self.0[block] &= !mask;
        }
    }
}

impl std::fmt::Display for Bitmap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

struct BitmapMasks {
    first_block: usize,
    first_mask: BitmapType,
    last_block: usize,
    last_mask: BitmapType,
}

impl BitmapMasks {
    fn new<T: std::ops::RangeBounds<usize>>(range: T, length: usize) -> Self {
        let start = match range.start_bound() {
            std::ops::Bound::Included(value) => *value,
            std::ops::Bound::Excluded(value) => *value + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let first_block = start / BitmapType::BITS as usize;
        let first_mask = BitmapType::MAX << (start % BitmapType::BITS as usize);

        let end = match range.end_bound() {
            std::ops::Bound::Included(value) => *value + 1,
            std::ops::Bound::Excluded(value) => *value,
            std::ops::Bound::Unbounded => length,
        };
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

impl Iterator for BitmapMasks {
    type Item = (usize, BitmapType);

    fn next(&mut self) -> Option<Self::Item> {
        match self.first_block.cmp(&self.last_block) {
            std::cmp::Ordering::Less => {
                let block = self.first_block;
                let mask = self.first_mask;
                self.first_block += 1;
                self.first_mask = !0;
                Some((block, mask))
            }
            std::cmp::Ordering::Equal => {
                let block = self.first_block;
                let mask = self.first_mask & self.last_mask;
                self.first_block += 1;
                if mask == 0 {
                    None
                } else {
                    Some((block, mask))
                }
            }
            std::cmp::Ordering::Greater => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.first_block..=self.last_block).size_hint()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_zeros() {
        let block = usize::BITS as usize;

        // Full block
        let mut bitmap = Bitmap::new(Box::new([usize::MAX, usize::MIN, usize::MAX]));
        println!("{bitmap}");
        assert_eq!(bitmap.find_zeros(block), Some(block..block * 2));
        assert_eq!(bitmap.find_zeros(block + 1), None); // doesn't fit

        // Full block and one extra
        bitmap.set_zeros(block - 1..block * 2 + 1);
        println!("{bitmap}");
        assert_eq!(bitmap.find_zeros(block), Some(block - 1..block * 2)); // stop as soon as enough bits are found
        assert_eq!(bitmap.find_zeros(block + 2), Some(block - 1..block * 2 + 1));
        assert_eq!(bitmap.find_zeros(block + 3), None); // doesn't fit
    }
}
