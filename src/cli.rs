use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "christ", about = "A beautiful Bible TUI for Christian developers")]
#[command(version, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Skip the startup banner animation
    #[arg(long, global = true)]
    pub no_banner: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Read a Bible verse, range, or chapter (e.g. "John 3:16", "Genesis 1", "Ps 23:1-6")
    Read {
        /// Bible reference (e.g. "John 3:16", "Genesis 1")
        #[arg(required = true, num_args = 1..)]
        reference: Vec<String>,

        /// Bible translation (default: KJV)
        #[arg(short, long, default_value = "KJV")]
        translation: String,
    },

    /// Search the Bible for a phrase or keyword
    Search {
        /// Search query
        #[arg(required = true, num_args = 1..)]
        query: Vec<String>,

        /// Bible translation to search in
        #[arg(short, long, default_value = "KJV")]
        translation: String,
    },

    /// Display a random Bible verse
    Random {
        /// Bible translation
        #[arg(short, long, default_value = "KJV")]
        translation: String,
    },

    /// Show today's verse of the day
    Today {
        /// Bible translation
        #[arg(short, long, default_value = "KJV")]
        translation: String,
    },

    /// Replay the startup animation
    Intro,

    /// Update christ-cli to the latest version
    Update {
        /// Check for updates without installing
        #[arg(long)]
        check: bool,
    },
}
