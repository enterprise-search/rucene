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

use crate::core::search::scorer::Scorer;
use crate::core::search::{DocIterator, NO_MORE_DOCS};
use crate::core::util::{DisiPriorityQueue, DocId};

use crate::Result;
use std::f32;

pub const DEFAULT_MIN_SHOULD_MATCH: i32 = 1;

/// A Scorer for OR like queries, counterpart of `ConjunctionScorer`.
pub struct DisjunctionSumScorer<T: Scorer> {
    sub_scorers: SubScorers<T>,
    needs_scores: bool,
    cost: usize,
    min_should_match: i32,
}

impl<T: Scorer> DisjunctionSumScorer<T> {
    pub fn new(
        children: Vec<T>,
        needs_scores: bool,
        min_should_match: i32,
    ) -> DisjunctionSumScorer<T> {
        debug_assert!(children.len() > 0);

        let cost = children.iter().map(|w| w.cost()).sum();

        let sub_scorers = if children.len() < 10 || min_should_match > DEFAULT_MIN_SHOULD_MATCH {
            SubScorers::SQ(SimpleQueue::new(children))
        } else {
            SubScorers::DPQ(DisiPriorityQueue::new(children))
        };

        DisjunctionSumScorer {
            sub_scorers,
            needs_scores,
            cost,
            min_should_match,
        }
    }
}

impl<T: Scorer> Scorer for DisjunctionSumScorer<T> {
    fn score(&mut self) -> Result<f32> {
        if !self.needs_scores {
            return Ok(0.0f32);
        }

        self.sub_scorers.score_sum()
    }
}

impl<T: Scorer> DocIterator for DisjunctionSumScorer<T> {
    fn doc_id(&self) -> DocId {
        self.sub_scorers.doc_id()
    }

    fn next(&mut self) -> Result<DocId> {
        self.approximate_next()
    }

    fn advance(&mut self, target: DocId) -> Result<DocId> {
        self.approximate_advance(target)
    }

    fn cost(&self) -> usize {
        self.cost
    }

    fn matches(&mut self) -> Result<bool> {
        Ok(true)
    }

    fn match_cost(&self) -> f32 {
        0f32
    }

    fn approximate_next(&mut self) -> Result<DocId> {
        let min_should_match = if self.min_should_match > DEFAULT_MIN_SHOULD_MATCH {
            Some(self.min_should_match)
        } else {
            None
        };

        self.sub_scorers.approximate_next(min_should_match)
    }

    fn approximate_advance(&mut self, target: DocId) -> Result<DocId> {
        self.sub_scorers.approximate_advance(target)
    }
}

/// The Scorer for DisjunctionMaxQuery.  The union of all documents generated by the the subquery
/// scorers is generated in document number order.  The score for each document is the maximum of
/// the scores computed by the subquery scorers that generate that document, plus
/// tieBreakerMultiplier times the sum of the scores for the other subqueries that generate the
/// document.
pub struct DisjunctionMaxScorer<T: Scorer> {
    sub_scorers: SubScorers<T>,
    needs_scores: bool,
    cost: usize,
    tie_breaker_multiplier: f32,
}

impl<T: Scorer> DisjunctionMaxScorer<T> {
    pub fn new(
        children: Vec<T>,
        tie_breaker_multiplier: f32,
        needs_scores: bool,
    ) -> DisjunctionMaxScorer<T> {
        debug_assert!(children.len() > 0);

        let cost = children.iter().map(|w| w.cost()).sum();

        let sub_scorers = if children.len() < 10 {
            SubScorers::SQ(SimpleQueue::new(children))
        } else {
            SubScorers::DPQ(DisiPriorityQueue::new(children))
        };

        DisjunctionMaxScorer {
            sub_scorers,
            needs_scores,
            cost,
            tie_breaker_multiplier,
        }
    }
}

impl<T: Scorer> Scorer for DisjunctionMaxScorer<T> {
    fn score(&mut self) -> Result<f32> {
        if !self.needs_scores {
            return Ok(0.0f32);
        }

        self.sub_scorers.score_max(self.tie_breaker_multiplier)
    }
}

impl<T: Scorer> DocIterator for DisjunctionMaxScorer<T> {
    fn doc_id(&self) -> DocId {
        self.sub_scorers.doc_id()
    }

    fn next(&mut self) -> Result<DocId> {
        self.approximate_next()
    }

    fn advance(&mut self, target: DocId) -> Result<DocId> {
        self.approximate_advance(target)
    }

    fn cost(&self) -> usize {
        self.cost
    }

    fn matches(&mut self) -> Result<bool> {
        Ok(true)
    }

    fn match_cost(&self) -> f32 {
        0f32
    }

    fn approximate_next(&mut self) -> Result<DocId> {
        self.sub_scorers.approximate_next(None)
    }

    fn approximate_advance(&mut self, target: DocId) -> Result<DocId> {
        self.sub_scorers.approximate_advance(target)
    }
}

pub struct SimpleQueue<T: Scorer> {
    scorers: Vec<T>,
    curr_doc: DocId,
}

impl<T: Scorer> SimpleQueue<T> {
    pub fn new(children: Vec<T>) -> SimpleQueue<T> {
        let mut curr_doc = NO_MORE_DOCS;
        for s in children.iter() {
            curr_doc = curr_doc.min(s.doc_id());
        }
        SimpleQueue {
            scorers: children,
            curr_doc,
        }
    }
}

pub enum SubScorers<T: Scorer> {
    SQ(SimpleQueue<T>),
    DPQ(DisiPriorityQueue<T>),
}

impl<T: Scorer> SubScorers<T> {
    fn score_sum(&mut self) -> Result<f32> {
        match self {
            SubScorers::SQ(sq) => {
                let mut score: f32 = 0.0f32;

                let doc_id = sq.curr_doc;
                for s in sq.scorers.iter_mut() {
                    if s.doc_id() == doc_id {
                        let sub_score = s.score()?;
                        score += sub_score;
                    }
                }

                Ok(score)
            }
            SubScorers::DPQ(dpq) => {
                let mut score: f32 = 0.0f32;
                let mut disi = dpq.top_list();

                loop {
                    let sub_score = disi.inner_mut().score()?;
                    score += sub_score;

                    if disi.next.is_null() {
                        break;
                    } else {
                        unsafe { disi = &mut *disi.next };
                    }
                }

                Ok(score)
            }
        }
    }

    fn score_max(&mut self, tie_breaker_multiplier: f32) -> Result<f32> {
        match self {
            SubScorers::SQ(sq) => {
                let mut score_sum = 0.0f32;
                let mut score_max = f32::NEG_INFINITY;

                let doc_id = sq.curr_doc;
                for s in sq.scorers.iter_mut() {
                    if s.doc_id() == doc_id {
                        let sub_score = s.score()?;

                        score_sum += sub_score;
                        score_max = score_max.max(sub_score);
                    }
                }

                Ok(score_max + (score_sum - score_max) * tie_breaker_multiplier)
            }
            SubScorers::DPQ(dbq) => {
                let mut score_sum = 0.0f32;
                let mut score_max = f32::NEG_INFINITY;
                let mut disi = dbq.top_list();

                loop {
                    let sub_score = disi.inner_mut().score()?;
                    score_sum += sub_score;
                    if sub_score > score_max {
                        score_max = sub_score;
                    }

                    if disi.next.is_null() {
                        break;
                    } else {
                        unsafe { disi = &mut *disi.next };
                    }
                }

                Ok(score_max + (score_sum - score_max) * tie_breaker_multiplier)
            }
        }
    }

    fn doc_id(&self) -> DocId {
        match self {
            SubScorers::SQ(sq) => sq.curr_doc,
            SubScorers::DPQ(dbq) => dbq.peek().doc(),
        }
    }

    fn approximate_next(&mut self, min_should_match: Option<i32>) -> Result<DocId> {
        match self {
            SubScorers::SQ(sq) => {
                let min_should_match = min_should_match.unwrap_or(DEFAULT_MIN_SHOULD_MATCH);

                loop {
                    if sq.curr_doc == NO_MORE_DOCS {
                        return Ok(sq.curr_doc);
                    }

                    // curr_doc begin with -1
                    let curr_doc = sq.curr_doc;
                    let mut min_doc = NO_MORE_DOCS;
                    for s in sq.scorers.iter_mut() {
                        if s.doc_id() == curr_doc {
                            s.approximate_next()?;
                        }

                        min_doc = min_doc.min(s.doc_id());
                    }
                    sq.curr_doc = min_doc;

                    if min_should_match > DEFAULT_MIN_SHOULD_MATCH {
                        let mut should_count = 0;

                        for s in sq.scorers.iter_mut() {
                            if s.doc_id() == min_doc {
                                should_count += 1;
                            }
                        }

                        if should_count < min_should_match {
                            continue;
                        }
                    }

                    return Ok(sq.curr_doc);
                }
            }
            SubScorers::DPQ(dbq) => {
                // reset with -1, @posting_reader.rs#1208
                let doc = dbq.peek().doc();

                loop {
                    dbq.peek_mut().approximate_next()?;
                    if dbq.peek().doc() != doc {
                        break;
                    }
                }

                Ok(dbq.peek().doc())
            }
        }
    }

    fn approximate_advance(&mut self, target: DocId) -> Result<DocId> {
        match self {
            SubScorers::SQ(sq) => {
                let mut min_doc = NO_MORE_DOCS;
                for s in sq.scorers.iter_mut() {
                    if s.doc_id() < target {
                        s.approximate_advance(target)?;
                    }

                    min_doc = min_doc.min(s.doc_id());
                }

                sq.curr_doc = min_doc;
                Ok(sq.curr_doc)
            }
            SubScorers::DPQ(dbq) => {
                loop {
                    dbq.peek_mut().approximate_advance(target)?;
                    if dbq.peek().doc() >= target {
                        break;
                    }
                }

                Ok(dbq.peek().doc())
            }
        }
    }
}
