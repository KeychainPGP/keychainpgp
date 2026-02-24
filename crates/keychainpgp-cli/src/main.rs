//! KeychainPGP CLI -- headless command-line interface.

mod commands;

use std::io::IsTerminal;

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

        /// Protect with a passphrase (WARNING: visible in process list; prefer interactive prompt)
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
        /// Passphrase for the private key (WARNING: visible in process list; prefer interactive prompt)
        #[arg(long)]
        passphrase: Option<String>,
    },

    /// Sign a message (reads from stdin, writes to stdout)
    Sign {
        /// Fingerprint of the signing key (uses first own key if omitted)
        #[arg(long)]
        key: Option<String>,

        /// Passphrase for the private key (WARNING: visible in process list; prefer interactive prompt)
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

/// Prompt for a passphrase interactively (hidden input).
/// Returns `None` if the user enters an empty string.
fn prompt_passphrase(prompt: &str) -> Option<String> {
    match rpassword::prompt_password(prompt) {
        Ok(p) if p.is_empty() => None,
        Ok(p) => Some(p),
        Err(_) => None,
    }
}

/// Resolve passphrase: use CLI arg if given, otherwise prompt interactively.
fn resolve_passphrase(cli_passphrase: Option<String>, prompt: &str) -> Option<String> {
    if cli_passphrase.is_some() {
        return cli_passphrase;
    }
    // Only prompt if stdin is a TTY (not piped)
    if std::io::stdin().is_terminal() {
        prompt_passphrase(prompt)
    } else {
        None
    }
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
        } => {
            let passphrase =
                resolve_passphrase(passphrase, "Passphrase (leave empty for no protection): ");
            commands::generate::run(&name, &email, passphrase.as_deref())?;
        }

        Commands::Encrypt { recipient } => commands::encrypt::run(&recipient)?,

        Commands::Decrypt { passphrase } => {
            let passphrase =
                resolve_passphrase(passphrase, "Passphrase (leave empty if key has none): ");
            commands::decrypt::run(passphrase.as_deref())?;
        }

        Commands::Sign { key, passphrase } => {
            let passphrase =
                resolve_passphrase(passphrase, "Passphrase (leave empty if key has none): ");
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
