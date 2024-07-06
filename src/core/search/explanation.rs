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

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Explanation {
    is_match: bool,
    value: f32,
    description: String,
    details: Vec<Explanation>,
}

impl Explanation {
    pub fn new(
        is_match: bool,
        value: f32,
        description: String,
        details: Vec<Explanation>,
    ) -> Explanation {
        let value = if !is_match { 0.0f32 } else { value };

        Explanation {
            is_match,
            value,
            description,
            details,
        }
    }

    pub fn is_match(&self) -> bool {
        self.is_match
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn description(&self) -> String {
        self.description.clone()
    }

    pub fn summary(&self) -> String {
        format!("{} = {}", self.value, self.description)
    }

    pub fn details(&self) -> &[Explanation] {
        self.details.as_ref()
    }

    pub fn to_string(&self, depth: i32) -> String {
        let mut buffer = "  ".repeat(depth as usize);

        buffer.push_str(&self.summary());
        buffer.push('\n');

        for detail in &self.details {
            buffer.push_str(&detail.to_string(depth + 1))
        }

        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explaination_serde() {
        let e = Explanation::new(
            true,
            1.23,
            "a good match".into(),
            vec![Explanation::new(true, 1.23, "a fair match".into(), vec![])],
        );
        let s = serde_json::to_string(&e).expect("failed to serialize explaination");
        let o: Explanation = serde_json::from_str(&s).expect("failed to deserialize explaination");
        assert_eq!(e.is_match, o.is_match);
        assert_eq!(e.value, o.value);
        assert_eq!(e.description, o.description);
        assert_eq!(e.details.len(), o.details.len());
    }

    #[test]
    fn explaination_debug() {
        let e = Explanation::new(
            true,
            1.23,
            "a good match".into(),
            vec![Explanation::new(true, 1.23, "a fair match".into(), vec![])],
        );
        let s = r#"Explanation { is_match: true, value: 1.23, description: "a good match", details: [Explanation { is_match: true, value: 1.23, description: "a fair match", details: [] }] }"#;
        let o = format!("{e:?}");
        assert_eq!(s, o);
    }
}
