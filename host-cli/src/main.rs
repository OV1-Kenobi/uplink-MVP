//! Uplink host-cli — native binary for dev + CI smoke tests.
//!
//! Usage:
//!   uplink identity new                        # generate + display new mnemonic
//!   uplink identity restore <mnemonic>         # restore from mnemonic
//!   uplink identity show                       # show npub of current identity
//!   uplink wallet balance                      # show balance (stub in A0)
//!   uplink wallet receive --sats <N>           # get Lightning invoice
//!   uplink wallet pay <bolt11>                 # pay invoice
//!   uplink stream add --period <secs> ...      # create a new streaming flow
//!   uplink stream list                         # list active streams
//!   uplink tick                                # advance scheduler to now

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "uplink", about = "Uplink streaming-sats CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Identity management
    Identity {
        #[command(subcommand)]
        action: IdentityAction,
    },
    /// Wallet operations
    Wallet {
        #[command(subcommand)]
        action: WalletAction,
    },
    /// Stream management
    Stream {
        #[command(subcommand)]
        action: StreamAction,
    },
    /// Advance scheduler to current time
    Tick,
}

#[derive(Subcommand)]
enum IdentityAction {
    /// Generate a new random identity
    New {
        #[arg(long, default_value = "0")]
        account: u32,
    },
    /// Restore from a BIP-39 mnemonic phrase
    Restore {
        mnemonic: String,
        #[arg(long, default_value = "0")]
        account: u32,
    },
    /// Show the npub of the current identity
    Show,
}

#[derive(Subcommand)]
enum WalletAction {
    Balance,
    Receive { #[arg(long)] sats: u64 },
    Pay { bolt11: String },
}

#[derive(Subcommand)]
enum StreamAction {
    Add {
        #[arg(long)] recipient_npub: String,
        #[arg(long)] msats_per_period: u64,
        #[arg(long)] period_seconds: u64,
    },
    List,
    Pause { stream_id: String },
    Resume { stream_id: String },
    Remove { stream_id: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Identity { action } => handle_identity(action).await?,
        Command::Wallet { action } => handle_wallet(action).await?,
        Command::Stream { action } => handle_stream(action).await?,
        Command::Tick => handle_tick().await?,
    }

    Ok(())
}

async fn handle_identity(action: IdentityAction) -> anyhow::Result<()> {
    match action {
        IdentityAction::New { account } => {
            let id = uplink_identity::UplinkIdentity::generate(account)?;
            println!("npub: {}", id.npub());
            println!("\n⚠️  BACKUP YOUR MNEMONIC — store offline:");
            for (i, word) in id.mnemonic_words().iter().enumerate() {
                print!("{:2}. {:12}  ", i + 1, word);
                if (i + 1) % 4 == 0 { println!(); }
            }
            println!();
        }
        IdentityAction::Restore { mnemonic, account } => {
            let id = uplink_identity::UplinkIdentity::from_mnemonic_str(&mnemonic, account)?;
            println!("Restored. npub: {}", id.npub());
        }
        IdentityAction::Show => {
            println!("(Phase A1: load identity from encrypted storage)");
        }
    }
    Ok(())
}

async fn handle_wallet(_action: WalletAction) -> anyhow::Result<()> {
    println!("Wallet operations available in Phase A3 (LDK native wallet).");
    Ok(())
}

async fn handle_stream(_action: StreamAction) -> anyhow::Result<()> {
    println!("Stream management available in Phase A6 (scheduler + host-cli integration).");
    Ok(())
}

async fn handle_tick() -> anyhow::Result<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    println!("tick at {now} — scheduler integration in Phase A6.");
    Ok(())
}
