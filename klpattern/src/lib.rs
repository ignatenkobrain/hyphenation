//! Data structures and methods for parsing and applying Knuth-Liang
//! hyphenation patterns and exceptions.

#![cfg_attr(feature = "serde_derive", feature(proc_macro))]

#[cfg(feature = "serde_derive")]
#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate fnv;

use std::borrow::Cow;
use std::cmp::{max};
use std::collections::hash_map::{HashMap, Entry};
use std::hash::BuildHasherDefault;
use std::iter::{once};
use std::mem;

use fnv::FnvHasher;

/// A pair representing a Knuth-Liang hyphenation pattern. It comprises
/// alphabetical characters for subword matching and the score of each
/// hyphenation point.
pub type KLPair = (String, Vec<u8>);


#[cfg(feature = "serde_derive")]
include!("serde_types.in.rs");

#[cfg(feature = "serde_codegen")]
include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));


pub trait KLPTrie<'a> {
    type Score;

    fn new() -> Self;

    fn insert(&mut self, KLPair) -> Option<Vec<u8>>;

    fn score(&'a self, &str) -> Self::Score;

    fn is_empty(&self) -> bool;

}


impl<'a> KLPTrie<'a> for Patterns {
    type Score = Vec<u8>;

    /// Creates an empty `Patterns` trie.
    fn new() -> Patterns {
        let fnv = BuildHasherDefault::<FnvHasher>::default();

        Patterns {
            tally: None,
            descendants: HashMap::with_hasher(fnv)
        }
    }

    /// Inserts a Knuth-Liang hyphenation pair into the trie.
    ///
    /// If the pattern already exists, the old tally is returned; if not, `None` is.
    fn insert(&mut self, (p, tally): KLPair) -> Option<Vec<u8>> {
        let node = p.chars().fold(self, |t, c| {
            match t.descendants.entry(c) {
                Entry::Vacant(e) => e.insert(Patterns::new()),
                Entry::Occupied(e) => e.into_mut()
            }
        });

        let mut retv = None;
        match node.tally {
            Some(ref mut old) => retv = Some(mem::replace(old, tally)),
            None => node.tally = Some(tally)
        }

        retv
    }

    /// Assigns a score to each potential hyphenation point.
    ///
    /// All patterns matching a substring of `word` are compounded, and for
    /// each hyphenation point, the highest competing value is selected.
    fn score(&self, word: &str) -> Self::Score {
        let w = match word.chars().any(|c| c.is_uppercase()) {
            true => Cow::Owned(word.to_lowercase()),
            false => Cow::Borrowed(word)
        };
        let cs = once('.').chain(w.chars()).chain(once('.'));
        let match_length = cs.clone().count();

        if match_length <= 3 {
            return vec![];
        }

        let hyphenable_length = match_length - 2;
        let mut points: Vec<u8> = vec![0; hyphenable_length - 1];

        for i in 0..match_length {
            let mut m = &self.descendants;
            for c in cs.clone().skip(i) {
                match m.get(&c) {
                    Some(&Patterns { tally: Some(ref t), descendants: ref m1 }) => {
                        m = m1;
                        for (j, &p) in t.iter().enumerate() {
                            let k = i + j;
                            if k > 1 && k <= hyphenable_length {
                                let p1 = points[k - 2];
                                points[k - 2] = max(p, p1)
                            }
                        }
                    },
                    Some(patterns) => m = &patterns.descendants,
                    _ => break
                }
            }
        }

        points
    }

    fn is_empty(&self) -> bool {
        self.descendants.is_empty()
    }
}


impl<'a> KLPTrie<'a> for Exceptions {
    type Score = Option<&'a Vec<u8>>;

    /// Creates an empty `Exceptions` map.
    fn new() -> Exceptions {
        Exceptions(HashMap::new())
    }

    /// Inserts a Knuth-Liang exception pair into the map.
    ///
    /// If the pattern already exists, the old score is returned; if not, `None` is.
    fn insert(&mut self, (pattern, score): KLPair) -> Option<Vec<u8>> {
        let Exceptions(ref mut m) = *self;

        m.insert(pattern, score)
    }

    /// Retrieves the score for each hyphenation point of `word`.
    fn score(&'a self, word: &str) -> Self::Score {
        let Exceptions(ref m) = *self;
        let w = match word.chars().any(|c| c.is_uppercase()) {
            true => Cow::Owned(word.to_lowercase()),
            false => Cow::Borrowed(word)
        };

        m.get(w.as_ref())
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
