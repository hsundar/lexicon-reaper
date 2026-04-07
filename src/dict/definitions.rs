use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

/// Cache for word definitions fetched from the Free Dictionary API.
/// Lookups happen in a background thread so they never block the game.
pub struct DefinitionCache {
    cache: Arc<Mutex<HashMap<String, Option<String>>>>,
}

impl DefinitionCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Request a definition for a word. The lookup runs in the background.
    /// Call `get` later to retrieve the result.
    pub fn request(&self, word: &str) {
        let word_upper = word.to_uppercase();
        let word_lower = word.to_lowercase();

        // Already cached or in-flight?
        {
            let cache = self.cache.lock().unwrap();
            if cache.contains_key(&word_upper) {
                return;
            }
        }

        // Mark as in-flight with None
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(word_upper.clone(), None);
        }

        let cache = Arc::clone(&self.cache);
        thread::spawn(move || {
            let definition = fetch_definition(&word_lower);
            let mut cache = cache.lock().unwrap();
            cache.insert(word_upper, definition);
        });
    }

    /// Get a cached definition. Returns:
    /// - Some(Some(def)) if fetched successfully
    /// - Some(None) if fetch is in-flight or failed
    /// - None if never requested
    pub fn get(&self, word: &str) -> Option<Option<String>> {
        let cache = self.cache.lock().unwrap();
        cache.get(&word.to_uppercase()).cloned()
    }
}

/// Fetch a short definition from the Free Dictionary API.
/// Returns the first meaning's first definition, truncated to ~80 chars.
fn fetch_definition(word: &str) -> Option<String> {
    let url = format!(
        "https://api.dictionaryapi.dev/api/v2/entries/en/{}",
        word
    );

    let response = ureq::get(&url).call().ok()?;
    let body = response.into_string().ok()?;
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;

    let entries = json.as_array()?;
    let first_entry = entries.first()?;
    let meanings = first_entry.get("meanings")?.as_array()?;
    let first_meaning = meanings.first()?;

    let part_of_speech = first_meaning
        .get("partOfSpeech")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let definitions = first_meaning.get("definitions")?.as_array()?;
    let first_def = definitions.first()?;
    let definition = first_def.get("definition")?.as_str()?;

    // Truncate long definitions
    let abbrev = if part_of_speech.is_empty() {
        String::new()
    } else {
        format!("({}.) ", abbreviate_pos(part_of_speech))
    };

    let max_len = 70;
    let def_text = if definition.len() > max_len {
        format!("{}{}...", abbrev, &definition[..max_len])
    } else {
        format!("{}{}", abbrev, definition)
    };

    Some(def_text)
}

fn abbreviate_pos(pos: &str) -> &str {
    match pos {
        "noun" => "n",
        "verb" => "v",
        "adjective" => "adj",
        "adverb" => "adv",
        "pronoun" => "pron",
        "preposition" => "prep",
        "conjunction" => "conj",
        "interjection" => "interj",
        other => other,
    }
}
