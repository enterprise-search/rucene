// Copyright 2019 Zhizhesihai (Beijing) Technology Limited.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::core::store::io::{IndexInput, RandomAccessInput};
use crate::core::util::{long_to_int_exact, BitSet, LongValues};

use crate::error::Error::{IllegalArgument, IllegalState};
use crate::Result;

use std::sync::Arc;

pub type BitsContext = Option<[u8; 64]>;

/// Interface for Bitset-like structures.
pub trait Bits: Send + Sync {
    /// Returns the value of the bit with the specified <code>index</code>.
    ///
    /// @param index index, should be non-negative and &lt; {@link #length()}. The result of passing
    ///     negative or out of bounds values is undefined by this interface, <b>just don't do
    /// it!</b> @return <code>true</code> if the bit is set, <code>false</code> otherwise.
    fn get(&self, index: usize) -> bool;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // these two method are currently only implemented for FixedBitSet used
    // in live docs
    fn as_bit_set(&self) -> &dyn BitSet {
        unreachable!()
    }
    fn as_bit_set_mut(&mut self) -> &mut dyn BitSet {
        unreachable!()
    }
    fn clone_box(&self) -> BitsRef {
        unreachable!()
    }
}

pub trait BitsMut: Send + Sync {
    fn get(&mut self, index: usize) -> bool;

    fn len(&self) -> usize;
}

pub type BitsRef = Arc<dyn Bits>;

#[derive(Clone)]
pub struct MatchAllBits {
    len: usize,
}

impl MatchAllBits {
    pub fn new(len: usize) -> Self {
        MatchAllBits { len }
    }
}

impl Bits for MatchAllBits {
    fn get(&self, _index: usize) -> bool {
        true
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        true
    }
}

impl BitsMut for MatchAllBits {
    fn get(&mut self, _index: usize) -> bool {
        true
    }

    fn len(&self) -> usize {
        self.len
    }
}

#[derive(Clone)]
pub struct MatchNoBits {
    len: usize,
}

impl MatchNoBits {
    pub fn new(len: usize) -> Self {
        MatchNoBits { len }
    }
}

impl Bits for MatchNoBits {
    fn get(&self, _index: usize) -> bool {
        false
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        true
    }
}

impl BitsMut for MatchNoBits {
    fn get(&mut self, _index: usize) -> bool {
        false
    }

    fn len(&self) -> usize {
        self.len
    }
}

#[derive(Clone)]
pub struct LiveBits {
    input: Arc<dyn RandomAccessInput>,
    count: usize,
}

impl LiveBits {
    pub fn new(data: &dyn IndexInput, offset: i64, count: usize) -> Result<LiveBits> {
        let length = (count + 7) >> 3;
        let input = data.random_access_slice(offset, length as i64)?;
        Ok(LiveBits {
            input: Arc::from(input),
            count,
        })
    }
}

impl Bits for LiveBits {
    fn get(&self, index: usize) -> bool {
        let bitset = self.input.read_byte((index >> 3) as u64).unwrap();
        (bitset & (1u8 << (index & 0x7))) != 0
    }

    fn len(&self) -> usize {
        self.count
    }
}

impl BitsMut for LiveBits {
    fn get(&mut self, index: usize) -> bool {
        let bitset = self.input.read_byte((index >> 3) as u64).unwrap();
        (bitset & (1u8 << (index & 0x7))) != 0
    }

    fn len(&self) -> usize {
        self.count
    }
}

pub struct FixedBits {
    len: usize,
    bits: Arc<Vec<i64>>,
}

impl FixedBits {
    pub fn new(bits: Arc<Vec<i64>>, len: usize) -> FixedBits {
        FixedBits {
            len,
            bits,
        }
    }

    pub fn cardinality(&self) -> usize {
        let mut n = 0_usize;
        self.bits.iter().for_each(|x| {
            n += x.count_ones() as usize
        });
        n
    }
}

impl Bits for FixedBits {
    fn get(&self, index: usize) -> bool {
        debug_assert!(index < self.len, "index out of bounds (index: {index}, num_bits: {}", self.len);
        let i = index >> 6; // div 64

        let bit_mask = 1i64 << (index % 64) as i64;
        self.bits[i] & bit_mask != 0
    }

    fn len(&self) -> usize {
        self.len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_bits() {
        let fixed_bits = FixedBits::new(Arc::new(vec![1, 2, 0xDeadBeef]), 191);
        assert_eq!(fixed_bits.len(), 191);
        assert_eq!(fixed_bits.get(0), true);
        assert_eq!(fixed_bits.get(1), false);
        assert_eq!(fixed_bits.get(64), false);
        assert_eq!(fixed_bits.get(65), true);
        assert_eq!(fixed_bits.get(66), false);
        assert_eq!(fixed_bits.get(190), false);
    }
}

#[derive(Clone)]
pub struct SparseBitsContext {
    // index of doc_id in doc_ids
    pub index: i64,
    // mutable
    // doc_id at index
    doc_id: i64,
    // mutable
    // doc_id at (index + 1)
    next_doc_id: i64, // mutable
}

impl SparseBitsContext {
    fn new(first_doc_id: i64) -> Self {
        SparseBitsContext {
            index: -1,
            doc_id: -1,
            next_doc_id: first_doc_id,
        }
    }

    fn reset(&mut self, first_doc_id: i64) {
        self.index = -1;
        self.doc_id = -1;
        self.next_doc_id = first_doc_id;
    }
}

#[derive(Clone)]
pub struct SparseBits<T: LongValues> {
    max_doc: i64,
    doc_ids_length: i64,
    first_doc_id: i64,
    doc_ids: T,
    pub ctx: SparseBitsContext,
}

impl<T: LongValues> SparseBits<T> {
    pub fn new(max_doc: i64, doc_ids_length: i64, doc_ids: T) -> Result<Self> {
        if doc_ids_length > 0 && max_doc <= doc_ids.get64(doc_ids_length - 1)? {
            return Err(IllegalArgument(
                "max_doc must be > the last element of doc_ids".to_owned(),
            ));
        };
        let first_doc_id = if doc_ids_length == 0 {
            max_doc
        } else {
            doc_ids.get64(0)?
        };
        Ok(SparseBits {
            max_doc,
            doc_ids_length,
            first_doc_id,
            doc_ids,
            ctx: SparseBitsContext::new(first_doc_id),
        })
    }

    /// Gallop forward and stop as soon as an index is found that is greater than
    ///  the given docId. *index* will store an index that stores a value
    /// that is <= *docId* while the return value will give an index
    /// that stores a value that is > *doc_id*. These indices can then be
    /// used to binary search.
    fn gallop(&self, ctx: &mut SparseBitsContext, doc_id: i64) -> Result<i64> {
        ctx.index += 1;
        ctx.doc_id = ctx.next_doc_id;
        let mut hi_index = ctx.index + 1;
        loop {
            if hi_index >= self.doc_ids_length {
                hi_index = self.doc_ids_length;
                ctx.next_doc_id = self.max_doc;
                return Ok(hi_index);
            }

            let hi_doc_id = self.doc_ids.get64(hi_index)?;
            if hi_doc_id > doc_id {
                ctx.next_doc_id = hi_doc_id;
                return Ok(hi_index);
            }

            let delta = hi_index - ctx.index;
            ctx.index = hi_index;
            ctx.doc_id = hi_doc_id;
            hi_index += delta << 1; // double the step each time
        }
    }

    fn binary_search(
        &self,
        ctx: &mut SparseBitsContext,
        mut hi_index: i64,
        doc_id: i64,
    ) -> Result<()> {
        while ctx.index + 1 < hi_index {
            let mid_index = ctx.index + (hi_index - ctx.index) / 2;
            let mid_doc_id = self.doc_ids.get64(mid_index)?;
            if mid_doc_id > doc_id {
                hi_index = mid_index;
                ctx.next_doc_id = mid_doc_id;
            } else {
                ctx.index = mid_index;
                ctx.doc_id = mid_doc_id;
            }
        }
        Ok(())
    }

    fn check_invariants(
        &self,
        ctx: &SparseBitsContext,
        next_index: i64,
        doc_id: i64,
    ) -> Result<()> {
        if ctx.doc_id > doc_id || ctx.next_doc_id <= doc_id {
            return Err(IllegalState(format!(
                "internal error a {} {} {}",
                doc_id, ctx.doc_id, ctx.next_doc_id
            )));
        }
        if !((ctx.index == -1 && ctx.doc_id == -1)
            || ctx.doc_id == self.doc_ids.get64(ctx.index)?)
        {
            return Err(IllegalState(format!(
                "internal error b {} {} {}",
                ctx.index,
                ctx.doc_id,
                self.doc_ids.get64(ctx.index)?
            )));
        }
        if !((next_index == self.doc_ids_length && ctx.next_doc_id == self.max_doc)
            || ctx.next_doc_id == self.doc_ids.get64(next_index)?)
        {
            return Err(IllegalState(format!(
                "internal error c {} {} {} {} {}",
                next_index,
                self.doc_ids_length,
                ctx.next_doc_id,
                self.max_doc,
                self.doc_ids.get64(next_index)?
            )));
        }
        Ok(())
    }

    fn exponential_search(&self, ctx: &mut SparseBitsContext, doc_id: i64) -> Result<()> {
        // seek forward by doubling the interval on each iteration
        let hi_index = self.gallop(ctx, doc_id)?;
        self.check_invariants(ctx, hi_index, doc_id)?;
        // now perform the actual binary search
        self.binary_search(ctx, hi_index, doc_id)
    }

    pub fn get64(&self, ctx: &mut SparseBitsContext, doc_id: i64) -> Result<bool> {
        if doc_id < ctx.doc_id {
            // reading doc ids backward, go back to the start
            ctx.reset(self.first_doc_id)
        }

        if doc_id >= ctx.next_doc_id {
            self.exponential_search(ctx, doc_id)?;
        }
        let next_index = ctx.index + 1;
        self.check_invariants(ctx, next_index, doc_id)?;
        Ok(doc_id == ctx.doc_id)
    }

    fn len(&self) -> usize {
        if let Err(ref e) = long_to_int_exact(self.max_doc) {
            panic!("max_doc too big{}", e);
        }

        self.max_doc as usize
    }

    pub fn context(&self) -> SparseBitsContext {
        SparseBitsContext::new(self.first_doc_id)
    }
}

impl<T: LongValues> Bits for SparseBits<T> {
    fn get(&self, index: usize) -> bool {
        self.get64(&mut self.context(), index as i64).unwrap()
    }

    fn len(&self) -> usize {
        SparseBits::len(self)
    }
}

impl<T: LongValues> BitsMut for SparseBits<T> {
    fn get(&mut self, index: usize) -> bool {
        unsafe {
            let ctx: *mut SparseBitsContext = &self.ctx as *const _ as *mut _;
            let b = ctx.as_mut_unchecked();
            self.get64(b, index as i64).unwrap()
        }
    }

    fn len(&self) -> usize {
        SparseBits::len(self)
    }
}
