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

pub use numeric::{Numeric, to_base36, int2sortable_bytes, float2sortable_int, sortable_int2float, sortable_bytes2int, long2sortable_bytes, double2sortable_long, sortable_long2double, sortable_bytes2long, sortable_float_bits, sortable_double_bits};

mod variant_value;

pub use variant_value::VariantValue;

mod bits;

pub use bits::{BitsMut, MatchNoBits, LiveBits, Bits, MatchAllBits, SparseBits, BitsRef};

mod version;

pub use version::{Version, VERSION_LATEST};

mod paged_bytes;

pub use paged_bytes::{PagedBytes, PagedBytesDataInput, PagedBytesReader};

mod doc_id_set_builder;

pub use doc_id_set_builder::DocIdSetBuilder;

mod context;

pub use context::{IndexedContext, KeyedContext};

mod counter;
mod bytes_ref;

pub use bytes_ref::{BytesRef, BytesRefBuilder};

mod bit_set;

pub use bit_set::{BitSet, BitSetIterator, FixedBitSet, bits2words, ImmutableBitSet};

mod bit_util;

pub use bit_util::{BitsRequired, UnsignedShift, ZigZagEncoding};

mod byte_block_pool;

pub use byte_block_pool::{ByteBlockPool, DirectTrackingAllocator, ByteBlockAllocator};

mod byte_slice_reader;

pub (crate) use byte_slice_reader::ByteSliceReader;

mod bytes_ref_hash;

pub use bytes_ref_hash::{DirectByteStartArray, DEFAULT_CAPACITY, BytesRefHash, BytesStartArray};

mod doc_id_set;

pub use doc_id_set::{BitDocIdSet, BitSetDocIterator, DocIdSetDocIterEnum, DocIdSetEnum, NotDocIdSet, ShortArrayDocIdSet};

mod int_block_pool;

pub use int_block_pool::{IntBlockPool, INT_BLOCK_MASK, INT_BLOCK_SHIFT, INT_BLOCK_SIZE, IntAllocator};

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

pub use string_util::{id2str, ID_LENGTH, sort_key_length, bytes_difference, random_id};

mod compression;

pub use compression::{CompressionMode, Decompress, Decompressor, Compress, Compressor};

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
