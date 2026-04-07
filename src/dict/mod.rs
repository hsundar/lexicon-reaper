mod trie;
pub mod definitions;

use std::collections::HashSet;
use trie::Trie;

const DICT_TEXT: &str = include_str!("../../assets/dictionary.txt");

pub struct Dictionary {
    words: HashSet<String>,
    trie: Trie,
}

impl Dictionary {
    pub fn new() -> Self {
        let mut words = HashSet::new();
        let mut trie = Trie::new();

        for line in DICT_TEXT.lines() {
            let word = line.trim().to_uppercase();
            if word.len() >= 3 && word.chars().all(|c| c.is_ascii_alphabetic()) {
                trie.insert(&word);
                words.insert(word);
            }
        }

        Self { words, trie }
    }

    /// Check if a word is valid (exists in dictionary, 3+ chars)
    pub fn is_word(&self, word: &str) -> bool {
        self.words.contains(&word.to_uppercase())
    }

    /// Check if a prefix could lead to a valid word
    pub fn is_prefix(&self, prefix: &str) -> bool {
        self.trie.is_prefix(&prefix.to_uppercase())
    }

    pub fn word_count(&self) -> usize {
        self.words.len()
    }
}
