use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Serialize, Deserialize};

// ═══════════════════════════════════════════════════════════════
// CONFIGURATION
// ═══════════════════════════════════════════════════════════════

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CaseStyle {
    Title,       // Happy
    Lower,       // happy
    Upper,       // HAPPY
    Random,      // hApPy
    Alternating, // HaPpY
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Position {
    Start,
    End,
    Between, // between words
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MemorableStyle {
    Classic,      // Adjective-Noun-Verb-Number (HappyTiger42!)
    Passphrase,   // word-word-word-word (correct-horse-battery-staple)
    Story,        // Subject-Verb-Object (TigerEatsFish)
    Alliterative, // Same starting letter (BraveBearBounces)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemorableConfig {
    pub word_count: usize,
    pub separator: String,
    pub case_style: CaseStyle,
    pub include_number: bool,
    pub number_position: Position,
    pub number_max: u32,
    pub include_special: bool,
    pub special_position: Position,
    pub style: MemorableStyle,
    pub count: usize,
    pub min_length: usize,
    pub max_length: usize,
}

impl Default for MemorableConfig {
    fn default() -> Self {
        Self {
            word_count: 3,
            separator: String::new(),
            case_style: CaseStyle::Title,
            include_number: true,
            number_position: Position::End,
            number_max: 99,
            include_special: true,
            special_position: Position::End,
            style: MemorableStyle::Classic,
            count: 1,
            min_length: 12,
            max_length: 32,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// WORD POOLS
// ═══════════════════════════════════════════════════════════════

const ADJECTIVES: &[&str] = &[
    "happy", "sunny", "fast", "clever", "brave", "calm", "eager", "fair",
    "gentle", "jolly", "kind", "lively", "nice", "proud", "silly", "witty",
    "bold", "cool", "deep", "epic", "fierce", "grand", "honest", "iron",
    "jade", "keen", "loud", "mystic", "noble", "odd", "prime", "quick",
    "rare", "sharp", "tough", "ultra", "vivid", "warm", "zesty", "arctic",
    "blaze", "cosmic", "dark", "electric", "frozen", "golden", "hidden",
    "infinite", "jungle", "kinetic", "lunar", "mega", "neon", "omega",
    "phantom", "quantum", "rapid", "sonic", "turbo", "viral", "wild",
    "ancient", "blazing", "crystal", "divine", "eternal", "flying",
    "glowing", "hyper", "icy", "lucky", "magic", "ninja", "pixel",
    "royal", "shadow", "titan", "wicked", "crimson", "silver", "stormy",
];

const NOUNS: &[&str] = &[
    "panda", "tiger", "eagle", "lion", "bear", "wolf", "fox", "hawk",
    "owl", "deer", "cat", "dog", "fish", "bird", "frog", "shark",
    "whale", "seal", "crab", "dragon", "phoenix", "raven", "cobra",
    "falcon", "panther", "viper", "jaguar", "horse", "orca", "mantis",
    "sphinx", "griffin", "kraken", "hydra", "cipher", "matrix", "nexus",
    "vertex", "atlas", "blade", "crown", "forge", "haven", "knight",
    "laser", "orbit", "prism", "quest", "realm", "storm", "tower",
    "vault", "spark", "flame", "frost", "comet", "nova", "pulse",
    "arrow", "castle", "ember", "galaxy", "harbor", "island", "meteor",
    "ocean", "planet", "river", "shield", "thunder", "wave", "zenith",
    "anchor", "beacon", "canyon", "dawn", "echo", "flint", "glacier",
];

const VERBS: &[&str] = &[
    "run", "jump", "swim", "fly", "walk", "sing", "dance", "read",
    "write", "draw", "cook", "eat", "sleep", "dream", "wake", "look",
    "see", "hear", "touch", "feel", "hunt", "race", "dive", "climb",
    "spin", "glide", "soar", "dash", "leap", "charge", "strike",
    "slash", "crush", "blast", "spark", "drift", "roam", "forge",
    "guard", "shine", "glow", "burn", "surge", "chase", "break",
    "build", "craft", "launch", "ride", "surf", "hack", "code",
    "type", "click", "ping", "push", "pull", "lift", "drop",
    "bolt", "whirl", "swoop", "prowl", "stalk", "howl", "growl",
    "stomp", "smash", "flash", "zoom", "twist", "shake", "drum",
];

const ADVERBS: &[&str] = &[
    "always", "never", "often", "rarely", "fast", "slowly", "boldly",
    "deeply", "freely", "gladly", "justly", "keenly", "loudly", "madly",
    "nearly", "only", "partly", "quickly", "really", "surely", "truly",
    "vastly", "wisely", "wildly", "daily", "gently", "highly", "mostly",
];

const COLORS: &[&str] = &[
    "red", "blue", "green", "gold", "black", "white", "silver", "purple",
    "cyan", "pink", "amber", "ivory", "coral", "jade", "ruby", "onyx",
    "azure", "crimson", "scarlet", "violet", "indigo", "teal", "bronze",
    "copper", "pearl", "cobalt", "emerald", "slate", "grey", "rose",
];

const SPECIALS: &[char] = &[
    '!', '@', '#', '$', '%', '&', '*', '?', '+', '=', '^', '~',
];

// ═══════════════════════════════════════════════════════════════
// GENERATION ENGINE
// ═══════════════════════════════════════════════════════════════

pub fn generate_memorable_password() -> String {
    generate_with_config(&MemorableConfig::default())
}

pub fn generate_with_config(config: &MemorableConfig) -> String {
    let mut rng = rand::thread_rng();
    // Retry loop to satisfy length constraints
    for _ in 0..100 {
        let result = build_password(&mut rng, config);
        if result.len() >= config.min_length && result.len() <= config.max_length {
            return result;
        }
    }
    // Fallback: return whatever we get
    build_password(&mut rng, config)
}

pub fn generate_batch(config: &MemorableConfig) -> Vec<String> {
    (0..config.count)
        .map(|_| generate_with_config(config))
        .collect()
}

fn build_password(rng: &mut impl Rng, config: &MemorableConfig) -> String {
    let words = pick_words(rng, config);
    let styled: Vec<String> = words.iter()
        .map(|w| apply_case(w, &config.case_style, rng))
        .collect();

    let mut parts: Vec<String> = styled;

    // Insert number
    if config.include_number {
        let num = if config.number_max <= 9 {
            rng.gen_range(0..=config.number_max).to_string()
        } else if config.number_max <= 99 {
            format!("{:02}", rng.gen_range(0..=config.number_max))
        } else if config.number_max <= 999 {
            format!("{:03}", rng.gen_range(0..=config.number_max))
        } else {
            rng.gen_range(0..=config.number_max).to_string()
        };

        match config.number_position {
            Position::Start => parts.insert(0, num),
            Position::End => parts.push(num),
            Position::Between => {
                let pos = if parts.len() > 1 { rng.gen_range(1..parts.len()) } else { parts.len() };
                parts.insert(pos, num);
            }
        }
    }

    // Insert special
    if config.include_special {
        let sym = SPECIALS.choose(rng).unwrap().to_string();
        match config.special_position {
            Position::Start => parts.insert(0, sym),
            Position::End => parts.push(sym),
            Position::Between => {
                let pos = if parts.len() > 1 { rng.gen_range(1..parts.len()) } else { parts.len() };
                parts.insert(pos, sym);
            }
        }
    }

    parts.join(&config.separator)
}

fn pick_words(rng: &mut impl Rng, config: &MemorableConfig) -> Vec<String> {
    match config.style {
        MemorableStyle::Classic => pick_classic(rng, config.word_count),
        MemorableStyle::Passphrase => pick_passphrase(rng, config.word_count),
        MemorableStyle::Story => pick_story(rng, config.word_count),
        MemorableStyle::Alliterative => pick_alliterative(rng, config.word_count),
    }
}

fn pick_classic(rng: &mut impl Rng, count: usize) -> Vec<String> {
    // Pattern: Adj Noun (Verb) (Adj) ...
    let pools: &[&[&str]] = &[ADJECTIVES, NOUNS, VERBS, COLORS, ADVERBS, ADJECTIVES];
    let mut words = Vec::new();
    for i in 0..count {
        let pool = pools[i % pools.len()];
        words.push(pool.choose(rng).unwrap().to_string());
    }
    words
}

fn pick_passphrase(rng: &mut impl Rng, count: usize) -> Vec<String> {
    // All from a merged pool for maximum entropy
    let mut all: Vec<&str> = Vec::new();
    all.extend_from_slice(ADJECTIVES);
    all.extend_from_slice(NOUNS);
    all.extend_from_slice(VERBS);
    all.extend_from_slice(COLORS);
    all.extend_from_slice(ADVERBS);

    let mut words = Vec::new();
    for _ in 0..count {
        words.push(all.choose(rng).unwrap().to_string());
    }
    words
}

fn pick_story(rng: &mut impl Rng, count: usize) -> Vec<String> {
    // Pattern: Subject Verb Object ...
    let mut words = Vec::new();
    let patterns: &[&[&str]] = &[NOUNS, VERBS, NOUNS, ADVERBS, ADJECTIVES, NOUNS];
    for i in 0..count {
        let pool = patterns[i % patterns.len()];
        words.push(pool.choose(rng).unwrap().to_string());
    }
    words
}

fn pick_alliterative(rng: &mut impl Rng, count: usize) -> Vec<String> {
    // All words start with the same letter
    let letter_idx = rng.gen_range(b'a'..=b'z') as char;

    let mut all: Vec<&str> = Vec::new();
    all.extend_from_slice(ADJECTIVES);
    all.extend_from_slice(NOUNS);
    all.extend_from_slice(VERBS);
    all.extend_from_slice(COLORS);

    let filtered: Vec<&&str> = all.iter()
        .filter(|w| w.starts_with(letter_idx))
        .collect();

    if filtered.len() < count {
        // Fallback to classic if not enough words for this letter
        return pick_classic(rng, count);
    }

    let mut words = Vec::new();
    let mut pool = filtered.clone();
    for _ in 0..count {
        let idx = rng.gen_range(0..pool.len());
        words.push(pool[idx].to_string());
        pool.remove(idx);
        if pool.is_empty() { break; }
    }
    words
}

fn apply_case(word: &str, style: &CaseStyle, rng: &mut impl Rng) -> String {
    match style {
        CaseStyle::Title => {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        }
        CaseStyle::Lower => word.to_lowercase(),
        CaseStyle::Upper => word.to_uppercase(),
        CaseStyle::Random => {
            word.chars().map(|c| {
                if rng.gen_bool(0.5) { c.to_uppercase().next().unwrap_or(c) }
                else { c.to_lowercase().next().unwrap_or(c) }
            }).collect()
        }
        CaseStyle::Alternating => {
            word.chars().enumerate().map(|(i, c)| {
                if i % 2 == 0 { c.to_uppercase().next().unwrap_or(c) }
                else { c.to_lowercase().next().unwrap_or(c) }
            }).collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_generates() {
        let pw = generate_memorable_password();
        assert!(pw.len() >= 8, "Password too short: {}", pw);
    }

    #[test]
    fn test_config_word_count() {
        let config = MemorableConfig {
            word_count: 5,
            separator: "-".to_string(),
            include_number: false,
            include_special: false,
            min_length: 0,
            max_length: 100,
            ..Default::default()
        };
        let pw = generate_with_config(&config);
        assert_eq!(pw.matches('-').count(), 4, "5 words should have 4 separators: {}", pw);
    }

    #[test]
    fn test_batch() {
        let config = MemorableConfig {
            count: 10,
            min_length: 0,
            max_length: 100,
            ..Default::default()
        };
        let batch = generate_batch(&config);
        assert_eq!(batch.len(), 10);
    }

    #[test]
    fn test_passphrase_style() {
        let config = MemorableConfig {
            style: MemorableStyle::Passphrase,
            separator: "-".to_string(),
            word_count: 4,
            case_style: CaseStyle::Lower,
            include_number: false,
            include_special: false,
            min_length: 0,
            max_length: 100,
            ..Default::default()
        };
        let pw = generate_with_config(&config);
        assert!(pw.chars().all(|c| c.is_lowercase() || c == '-'), "Should be lowercase: {}", pw);
    }

    #[test]
    fn test_upper_case() {
        let config = MemorableConfig {
            case_style: CaseStyle::Upper,
            include_number: false,
            include_special: false,
            min_length: 0,
            max_length: 100,
            ..Default::default()
        };
        let pw = generate_with_config(&config);
        assert!(pw.chars().all(|c| c.is_uppercase()), "Should be uppercase: {}", pw);
    }
}
