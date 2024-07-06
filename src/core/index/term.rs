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

/// A Term represents a word from text.  This is the unit of search.  It is
/// composed of two elements, the text of the word, as a string, and the name of
/// the field that the text occurred in.
///
/// Note that terms may represent more than words from text fields, but also
/// things like dates, email addresses, urls, etc.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct Term {
    /// The field indicates the part of a document which this term came from.
    pub field: String,
    pub(crate) bytes: Vec<u8>,
}

impl Term {
    /// Constructs a Term with the given field and bytes.
    /// <p>Note that a null field or null bytes value results in undefined
    /// behavior for most Lucene APIs that accept a Term parameter.
    ///
    /// <p>The provided BytesRef is copied when it is non null.
    pub fn new(field: String, bytes: Vec<u8>) -> Term {
        Term { field, bytes }
    }

    pub fn from_str(field: String, text: &str) -> Self {
        Self {
            field: field,
            bytes: text.bytes().collect(),
        }
    }

    /// Returns the field of this term.   
    pub fn field(&self) -> &str {
        &self.field
    }

    pub fn bytes(&self) -> &[u8] {
        return &self.bytes;
    }

    /// Returns the text of this term.  In the case of words, this is simply the
    /// text of the word.  In the case of dates and other types, this is an
    /// encoding of the object as a string.
    pub fn text(&self) -> String {
        String::from_utf8(self.bytes.clone()).unwrap_or(format!("{:02X?}", self.bytes))
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.field, self.text())
    }

    pub fn is_empty(&self) -> bool {
        self.field.is_empty() && self.bytes.is_empty()
    }

    pub fn copy_bytes(&mut self, bytes: &[u8]) {
        if self.bytes.len() != bytes.len() {
            self.bytes.resize(bytes.len(), 0);
        }
        self.bytes.copy_from_slice(bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn term_ctor_utf8() {
        let f = "title";
        let s = "search";
        let term = Term::new(f.into(), s.bytes().collect());
        assert_eq!(term.field(), f);
        assert_eq!(&term.text(), s);
        assert_eq!(&term.to_string(), "title:search");
    }

    #[test]
    fn term_to_string() {
        let f = "title";
        let v = vec![0, 1, 2, 3, 254, 255];
        let term = Term::new(f.into(), v);
        assert_eq!(term.field(), f);
        assert_eq!(&term.text(), "[00, 01, 02, 03, FE, FF]");
        assert_eq!(&term.to_string(), "title:[00, 01, 02, 03, FE, FF]");
    }

    #[test]
    fn term_cmp() {
        let term_1 = Term::from_str("title".into(), "hello");
        let term_2 = Term::from_str("title".into(), "world");
        let term_3 = Term::from_str("body".into(), "world");
        assert_eq!(term_1, term_1.clone());
        assert_eq!(term_2, term_2.clone());
        assert!(term_1 < term_2);
        assert!(term_1 != term_2);
        assert!(term_2 > term_1);
        assert!(term_3 < term_1);
        assert!(term_1 > term_3);
        assert!(term_1 != term_3);
    }
}
