use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum GenerationLevel {
    /// Fast — basic patterns only (~10K candidates)
    Quick,
    /// Balanced — all standard patterns (~100K candidates)
    Standard,
    /// Thorough — deep combinations and leet (~500K+ candidates)
    Deep,
    /// Maximum — everything including rules and full combos (~1M+ candidates)
    Insane,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    /// One password per line
    Plain,
    /// JSON array
    Json,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum MemStyle {
    /// Adjective-Noun-Verb (HappyTiger42!)
    Classic,
    /// Random word chain (correct-horse-battery)
    Passphrase,
    /// Subject-Verb-Object (TigerEatsFish)
    Story,
    /// Same starting letter (BraveBearBounces)
    Alliterative,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum MemCase {
    Title,
    Lower,
    Upper,
    Random,
    Alternating,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum NumPosition {
    Start,
    End,
    Between,
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "JIGSAW — The Intelligent Password Toolkit",
    long_about = "JIGSAW generates targeted wordlists from personal profiles,\ncreates memorable passwords, and performs mask/Markov attacks.\n\nExamples:\n  jigsaw --personal --profile target.json --level deep\n  jigsaw --memorable --words 4 --mem-sep \"-\" --count 10\n  jigsaw --mask '?u?l?l?d?d' --output wordlist.txt\n  jigsaw server --port 8080\n  jigsaw --interactive"
)]
pub struct JigsawArgs {
    #[command(subcommand)]
    pub command: Option<Commands>,

    // ═══════════════════════════════════════════════
    // GLOBAL OPTIONS
    // ═══════════════════════════════════════════════

    /// Output file path (default: stdout)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Plain)]
    pub format: OutputFormat,

    /// Number of threads (default: auto)
    #[arg(short, long)]
    pub threads: Option<usize>,

    /// Run in interactive wizard mode
    #[arg(short, long)]
    pub interactive: bool,

    // ═══════════════════════════════════════════════
    // MASK ATTACK
    // ═══════════════════════════════════════════════

    /// Mask pattern (e.g. ?u?l?l?d?d)
    #[arg(short, long)]
    pub mask: Option<String>,

    /// Rule file path
    #[arg(short, long)]
    pub rules: Option<PathBuf>,

    // ═══════════════════════════════════════════════
    // MARKOV ENGINE
    // ═══════════════════════════════════════════════

    /// Train a Markov model from this wordlist
    #[arg(long, value_name = "WORDLIST")]
    pub train: Option<PathBuf>,

    /// Path to Markov model file
    #[arg(long, value_name = "MODEL_PATH")]
    pub model: Option<PathBuf>,

    /// Run in Markov generation mode
    #[arg(long)]
    pub markov: bool,

    /// Number of candidates for Markov mode
    #[arg(long, default_value_t = 10000)]
    pub count: usize,

    // ═══════════════════════════════════════════════
    // PERSONAL ATTACK
    // ═══════════════════════════════════════════════

    /// Run in Personal Attack mode
    #[arg(long)]
    pub personal: bool,

    /// Path to a Personal Profile JSON
    #[arg(long, value_name = "PROFILE_PATH")]
    pub profile: Option<PathBuf>,

    /// Generation intensity level
    #[arg(long, value_enum, default_value_t = GenerationLevel::Standard)]
    pub level: GenerationLevel,

    /// Minimum password length filter
    #[arg(long)]
    pub min_length: Option<usize>,

    /// Maximum password length filter
    #[arg(long)]
    pub max_length: Option<usize>,

    /// Check if this password exists in generated wordlist
    #[arg(long, value_name = "PASSWORD")]
    pub check: Option<String>,

    // ═══════════════════════════════════════════════
    // MEMORABLE PASSWORD
    // ═══════════════════════════════════════════════

    /// Generate memorable password(s)
    #[arg(long)]
    pub memorable: bool,

    /// Number of words in memorable password
    #[arg(long, default_value_t = 3)]
    pub words: usize,

    /// Separator between words
    #[arg(long, default_value = "")]
    pub mem_sep: String,

    /// Memorable password style
    #[arg(long, value_enum, default_value_t = MemStyle::Classic)]
    pub mem_style: MemStyle,

    /// Case style for memorable password
    #[arg(long, value_enum, default_value_t = MemCase::Title)]
    pub mem_case: MemCase,

    /// Include a number in memorable password
    #[arg(long, default_value_t = true)]
    pub mem_number: bool,

    /// Skip number in memorable password
    #[arg(long)]
    pub no_number: bool,

    /// Number position in memorable password
    #[arg(long, value_enum, default_value_t = NumPosition::End)]
    pub num_pos: NumPosition,

    /// Maximum number value (9, 99, 999, 9999)
    #[arg(long, default_value_t = 99)]
    pub num_max: u32,

    /// Include special character  
    #[arg(long, default_value_t = true)]
    pub mem_special: bool,

    /// Skip special character
    #[arg(long)]
    pub no_special: bool,

    /// Special char position
    #[arg(long, value_enum, default_value_t = NumPosition::End)]
    pub special_pos: NumPosition,

    /// How many memorable passwords to generate
    #[arg(long, default_value_t = 1)]
    pub mem_count: usize,

    /// Minimum memorable password length
    #[arg(long, default_value_t = 12)]
    pub mem_min_len: usize,

    /// Maximum memorable password length
    #[arg(long, default_value_t = 32)]
    pub mem_max_len: usize,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the REST API server
    Server {
        /// Port to listen on
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
}
