use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct JigsawArgs {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Mask pattern (e.g. ?u?l?l?d?d)
    #[arg(short, long)]
    pub mask: Option<String>,

    /// Rule file path (e.g. rules/best64.rule)
    #[arg(short, long)]
    pub rules: Option<PathBuf>,

    /// Number of threads to use (default: auto)
    #[arg(short, long)]
    pub threads: Option<usize>,

    /// Output file path (default: stdout)
    #[arg(short, long)]
    pub output: Option<PathBuf>,


    /// Run in interactive wizard mode
    #[arg(short, long)]
    pub interactive: bool,

    /* Markov Engine Flags */

    /// Train a Markov model from this wordlist file
    #[arg(long, value_name = "WORDLIST")]
    pub train: Option<PathBuf>,

    /// Path to Markov model file (for saving after --train or loading for --markov)
    #[arg(long, value_name = "MODEL_PATH")]
    pub model: Option<PathBuf>,

    /// Run in Markov generation mode
    #[arg(long)]
    pub markov: bool,

    /// Number of candidates to generate (required for Markov mode)
    #[arg(long, default_value_t = 10000)]
    pub count: usize,

    /* Personal Attack Flags */

    /// Run in Personal Attack mode
    #[arg(long)]
    pub personal: bool,

    /// Path to a Personal Profile JSON file
    #[arg(long, value_name = "PROFILE_PATH")]
    pub profile: Option<PathBuf>,

    /// Generate a memorable password
    #[arg(long)]
    pub memorable: bool,

    /// Check if this password exists in the generated wordlist
    #[arg(long, value_name = "PASSWORD")]
    pub check: Option<String>,
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
