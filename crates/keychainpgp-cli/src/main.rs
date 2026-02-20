//! KeychainPGP CLI -- headless command-line interface.

mod commands;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "keychainpgp",
    about = "Simple OpenPGP encryption for the command line",
    version,
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new key pair
    Generate {
        /// Your display name
        #[arg(long)]
        name: String,

        /// Your email address
        #[arg(long)]
        email: String,

        /// Protect with a passphrase (will prompt if not given)
        #[arg(long)]
        passphrase: Option<String>,
    },

    /// Encrypt a message (reads from stdin, writes to stdout)
    Encrypt {
        /// Recipient key fingerprint(s)
        #[arg(short, long, required = true)]
        recipient: Vec<String>,
    },

    /// Decrypt a message (reads from stdin, writes to stdout)
    Decrypt {
        /// Passphrase for the private key (will prompt if needed)
        #[arg(long)]
        passphrase: Option<String>,
    },

    /// Sign a message (reads from stdin, writes to stdout)
    Sign {
        /// Fingerprint of the signing key (uses first own key if omitted)
        #[arg(long)]
        key: Option<String>,

        /// Passphrase for the private key
        #[arg(long)]
        passphrase: Option<String>,
    },

    /// Verify a signed message (reads from stdin, writes content to stdout)
    Verify {
        /// Fingerprint or email of the expected signer
        #[arg(long)]
        signer: String,
    },

    /// Inspect a key file and display its metadata
    Inspect {
        /// Path to the key file (or - for stdin)
        file: String,
    },

    /// Key management commands
    Keys {
        #[command(subcommand)]
        action: KeysAction,
    },
}

#[derive(Subcommand)]
enum KeysAction {
    /// List all keys in the keyring
    List,

    /// Import a key from a file
    Import {
        /// Path to the key file
        file: std::path::PathBuf,
    },

    /// Export a key to stdout
    Export {
        /// Fingerprint of the key to export
        fingerprint: String,
    },

    /// Delete a key from the keyring
    Delete {
        /// Fingerprint of the key to delete
        fingerprint: String,
    },

    /// Search keys by name, email, or fingerprint
    Search {
        /// Search query
        query: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"))
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();

    match cli.command {
        Commands::Generate {
            name,
            email,
            passphrase,
        } => commands::generate::run(&name, &email, passphrase.as_deref())?,

        Commands::Encrypt { recipient } => commands::encrypt::run(&recipient)?,

        Commands::Decrypt { passphrase } => commands::decrypt::run(passphrase.as_deref())?,

        Commands::Sign { key, passphrase } => {
            commands::sign::run(key.as_deref(), passphrase.as_deref())?;
        }

        Commands::Verify { signer } => commands::verify::run(&signer)?,

        Commands::Inspect { file } => commands::inspect::run(&file)?,

        Commands::Keys { action } => match action {
            KeysAction::List => commands::keys::list()?,
            KeysAction::Import { file } => commands::keys::import(&file)?,
            KeysAction::Export { fingerprint } => commands::keys::export(&fingerprint)?,
            KeysAction::Delete { fingerprint } => commands::keys::delete(&fingerprint)?,
            KeysAction::Search { query } => commands::keys::search(&query)?,
        },
    }

    Ok(())
}
