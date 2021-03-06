/// A basic trie, used to associate patterns to their hyphenation scores.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Patterns {
    pub tally: Option<Vec<(u8, u8)>>,
    pub descendants: HashMap<u8, Patterns, BuildHasherDefault<FnvHasher>>
}

/// A specialized hash map of pattern-score pairs.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Exceptions(pub HashMap<String, Vec<usize>>);
