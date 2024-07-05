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

pub type DocId = i32;

pub mod bkd;
pub mod external;
pub mod fst;
pub mod packed;

mod numeric;

pub use numeric::{
    double2sortable_long, float2sortable_int, int2sortable_bytes, long2sortable_bytes,
    sortable_bytes2int, sortable_bytes2long, sortable_double_bits, sortable_float_bits,
    sortable_int2float, sortable_long2double, to_base36, Numeric,
};

mod variant_value;

pub use variant_value::VariantValue;

mod bits;

pub use bits::{Bits, BitsMut, BitsRef, LiveBits, MatchAllBits, MatchNoBits, SparseBits};

mod version;

pub use version::{Version, VERSION_LATEST};

mod paged_bytes;

pub use paged_bytes::{PagedBytes, PagedBytesDataInput, PagedBytesReader};

mod doc_id_set_builder;

pub use doc_id_set_builder::DocIdSetBuilder;

mod context;

pub use context::{IndexedContext, KeyedContext};

mod bytes_ref;
mod counter;

pub use bytes_ref::{BytesRef, BytesRefBuilder};

mod bit_set;

pub use bit_set::{bits2words, BitSet, BitSetIterator, FixedBitSet, ImmutableBitSet};

mod bit_util;

pub use bit_util::{BitsRequired, UnsignedShift, ZigZagEncoding};

mod byte_block_pool;

pub use byte_block_pool::{ByteBlockAllocator, ByteBlockPool, DirectTrackingAllocator};

mod byte_slice_reader;

pub(crate) use byte_slice_reader::ByteSliceReader;

mod bytes_ref_hash;

pub use bytes_ref_hash::{BytesRefHash, BytesStartArray, DirectByteStartArray, DEFAULT_CAPACITY};

mod doc_id_set;

pub use doc_id_set::{
    BitDocIdSet, BitSetDocIterator, DocIdSetDocIterEnum, DocIdSetEnum, NotDocIdSet,
    ShortArrayDocIdSet,
};

mod int_block_pool;

pub use int_block_pool::{
    IntAllocator, IntBlockPool, INT_BLOCK_MASK, INT_BLOCK_SHIFT, INT_BLOCK_SIZE,
};

mod ints_ref;

pub use ints_ref::{to_ints_ref, IntsRefBuilder};

mod math;

pub use math::{gcd, log, long_to_int_exact};

mod selector;

mod small_float;

pub use small_float::SmallFloat;

mod sorter;

pub use sorter::{Sorter, BINARY_SORT_THRESHOLD};

mod string_util;

pub use string_util::{bytes_difference, id2str, random_id, sort_key_length, ID_LENGTH};

mod compression;

pub use compression::{Compress, CompressionMode, Compressor, Decompress, Decompressor};

mod disi;

pub use disi::DisiPriorityQueue;

use std::ops::Deref;

use crate::core::codec::doc_values::NumericDocValues;

use crate::Result;

// a iterator that can be used over and over by call reset
pub trait ReusableIterator: Iterator {
    fn reset(&mut self);
}

pub fn fill_slice<T: Copy>(array: &mut [T], value: T) {
    for i in array {
        *i = value;
    }
}

pub fn over_size(size: usize) -> usize {
    let mut size = size;
    let mut extra = size >> 3;
    if extra < 3 {
        // for very small arrays, where constant overhead of
        // realloc is presumably relatively high, we grow
        // faster
        extra = 3;
    }
    size += extra;
    size
}

pub const BM25_SIMILARITY_IDF: &str = "idf";

pub struct DerefWrapper<T>(pub T);

impl<T> Deref for DerefWrapper<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Abstraction over an array of longs.
///
/// This class extends `NumericDocValues` so that we don't need to add another
/// level of abstraction every time we want eg. to use the `PackedInts`
/// utility classes to represent a `NumericDocValues` instance.
pub trait LongValues: NumericDocValues {
    fn get64(&self, index: i64) -> Result<i64>;

    fn get64_mut(&mut self, index: i64) -> Result<i64> {
        self.get64(index)
    }
}

pub trait CloneableLongValues: LongValues {
    fn cloned(&self) -> Box<dyn CloneableLongValues>;

    fn cloned_lv(&self) -> Box<dyn LongValues>;
}

impl<T: LongValues + Clone + 'static> CloneableLongValues for T {
    fn cloned(&self) -> Box<dyn CloneableLongValues> {
        Box::new(self.clone())
    }

    fn cloned_lv(&self) -> Box<dyn LongValues> {
        Box::new(self.clone())
    }
}
