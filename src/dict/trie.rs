const ALPHABET_SIZE: usize = 26;

fn char_index(c: char) -> Option<usize> {
    let c = c.to_ascii_uppercase();
    if c.is_ascii_uppercase() {
        Some((c as u8 - b'A') as usize)
    } else {
        None
    }
}

#[derive(Default)]
struct TrieNode {
    children: [Option<Box<TrieNode>>; ALPHABET_SIZE],
    is_end: bool,
}

pub struct Trie {
    root: TrieNode,
}

impl Trie {
    pub fn new() -> Self {
        Self {
            root: TrieNode::default(),
        }
    }

    pub fn insert(&mut self, word: &str) {
        let mut node = &mut self.root;
        for ch in word.chars() {
            if let Some(idx) = char_index(ch) {
                node = node.children[idx].get_or_insert_with(|| Box::new(TrieNode::default()));
            } else {
                return;
            }
        }
        node.is_end = true;
    }

    pub fn is_prefix(&self, prefix: &str) -> bool {
        let mut node = &self.root;
        for ch in prefix.chars() {
            if let Some(idx) = char_index(ch) {
                match &node.children[idx] {
                    Some(child) => node = child,
                    None => return false,
                }
            } else {
                return false;
            }
        }
        true
    }

    #[allow(dead_code)]
    pub fn contains(&self, word: &str) -> bool {
        let mut node = &self.root;
        for ch in word.chars() {
            if let Some(idx) = char_index(ch) {
                match &node.children[idx] {
                    Some(child) => node = child,
                    None => return false,
                }
            } else {
                return false;
            }
        }
        node.is_end
    }
}
