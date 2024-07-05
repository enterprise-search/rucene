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

extern crate error_chain;
extern crate serde_json;

use thiserror::Error;

use crate::core::index;
use crate::core::search;
use crate::core::search::collector;

use std::sync::PoisonError;

pub use super::Result;

#[derive(Debug, Error)]
pub enum Error {
    #[error("a thread holding the locked panicked and poisoned the lock")]
    Poisoned,
    #[error("Illegal state: {0}")]
    IllegalState(String),
    #[error("IllegalArgument: {0}")]
    IllegalArgument(String),
    #[error("Unexpected EOF: {0}")]
    UnexpectedEOF(String),
    #[error("Corrupt Index: {0}")]
    CorruptIndex(String),
    #[error("Unsupported Operation: {0}")]
    UnsupportedOperation(String),
    #[error("Already Closed: {0}")]
    AlreadyClosed(String),
    #[error("Runtime Error: {0}")]
    RuntimeError(String),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Format Error: {0}")]
    FmtError(#[from] std::fmt::Error),
    #[error("FromUtf8 Error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("Utf8 Error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("ParseIntError: {0}")]
    NumError(#[from] std::num::ParseIntError),
    #[error("ParseFloatError: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("SerdeJsonError: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("ffi Null Error: {0}")]
    NulError(#[from] std::ffi::NulError),
    #[error("System Time Error: {0}")]
    TimeError(#[from] std::time::SystemTimeError),
    #[error("Collector Error: {0}")]
    CollectorError(#[from] collector::Error),
    #[error("Search Error: {0}")]
    SearchError(#[from] search::Error),
    #[error("Index Error: {0}")]
    IndexError(#[from] index::Error),
}

impl<Guard> From<PoisonError<Guard>> for Error {
    fn from(_: PoisonError<Guard>) -> Error {
        Error::Poisoned.into()
    }
}
