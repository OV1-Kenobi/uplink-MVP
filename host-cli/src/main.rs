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
    #[arg(long, env = "UPLINK_PASSWORD", default_value = "default-dev-password")]
    password: String,

    #[arg(long, default_value = "uplink.db")]
    db_path: String,

    #[command(subcommand)]
    command: Command,

    #[arg(long, default_value = "regtest")]
    network: String,

    #[arg(long, default_value = "http://localhost:3000")]
    esplora_url: String,

    #[arg(long, default_value = "uplink_ldk")]
    ldk_dir: String,
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
    Address,
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
    let db_path = std::path::Path::new(&cli.db_path);
    let store = uplink_storage::PlatformStore::open(db_path, &cli.password)?;

    match cli.command {
        Command::Identity { action } => handle_identity(action, &store).await?,
        Command::Wallet { action } => {
            let id = load_identity(&store).await?;
            let network = cli.network.parse::<bitcoin::Network>()?;
            let wallet = uplink_wallet::native::NativeLdkWallet::new(
                &id,
                &cli.ldk_dir,
                network,
                &cli.esplora_url
            )?;
            handle_wallet(action, &wallet).await?
        }
        Command::Stream { action } => handle_stream(action).await?,
        Command::Tick => handle_tick().await?,
    }

    Ok(())
}

async fn handle_identity(action: IdentityAction, store: &dyn uplink_storage::KvStore) -> anyhow::Result<()> {
    match action {
        IdentityAction::New { account } => {
            let id = uplink_identity::UplinkIdentity::generate(account)?;
            println!("npub: {}", id.npub());

            // Persist (mnemonic phrase)
            store.put("identity_mnemonic", id.mnemonic_phrase().as_bytes()).await?;
            store.put("identity_account", &account.to_be_bytes()).await?;

            println!("\n⚠️  BACKUP YOUR MNEMONIC — store offline:");
            for (i, word) in id.mnemonic_words().iter().enumerate() {
                print!("{:2}. {:12}  ", i + 1, word);
                if (i + 1) % 4 == 0 { println!(); }
            }
            println!();
        }
        IdentityAction::Restore { mnemonic, account } => {
            let id = uplink_identity::UplinkIdentity::from_mnemonic_str(&mnemonic, account)?;

            // Persist
            store.put("identity_mnemonic", id.mnemonic_phrase().as_bytes()).await?;
            store.put("identity_account", &account.to_be_bytes()).await?;

            println!("Restored. npub: {}", id.npub());
        }
        IdentityAction::Show => {
            let mnemonic_bytes = store.get("identity_mnemonic").await?
                .ok_or_else(|| anyhow::anyhow!("No identity found. Run 'identity new' or 'identity restore'."))?;
            let mnemonic = String::from_utf8(mnemonic_bytes)?;

            let account_bytes = store.get("identity_account").await?
                .unwrap_or_else(|| 0u32.to_be_bytes().to_vec());
            let account = u32::from_be_bytes(account_bytes.try_into().unwrap_or([0u8; 4]));

            let id = uplink_identity::UplinkIdentity::from_mnemonic_str(&mnemonic, account)?;
            println!("Current Identity:");
            println!("  npub:    {}", id.npub());
            println!("  account: {}", id.account_index());
        }
    }
    Ok(())
}

async fn load_identity(store: &dyn uplink_storage::KvStore) -> anyhow::Result<uplink_identity::UplinkIdentity> {
    let mnemonic_bytes = store.get("identity_mnemonic").await?
        .ok_or_else(|| anyhow::anyhow!("No identity found. Run 'identity new' or 'identity restore'."))?;
    let mnemonic = String::from_utf8(mnemonic_bytes)?;

    let account_bytes = store.get("identity_account").await?
        .unwrap_or_else(|| 0u32.to_be_bytes().to_vec());
    let account = u32::from_be_bytes(account_bytes.try_into().unwrap_or([0u8; 4]));

    Ok(uplink_identity::UplinkIdentity::from_mnemonic_str(&mnemonic, account)?)
}

async fn handle_wallet(action: WalletAction, wallet: &uplink_wallet::native::NativeLdkWallet) -> anyhow::Result<()> {
    use uplink_wallet::WalletExecutor;

    match action {
        WalletAction::Balance => {
            wallet.sync()?;
            let balance = wallet.balance()?;
            println!("Wallet Balance:");
            println!("  Lightning:  {} msats", balance.lightning_msats);
            println!("  On-chain:   {} sats", balance.onchain_confirmed_sats);
        }
        WalletAction::Address => {
            let addr = wallet.receive_onchain_address()?;
            println!("On-chain Address: {}", addr);
        }
        WalletAction::Receive { sats } => {
            let invoice = wallet.receive_invoice(sats * 1000, "Uplink host-cli top-up")?;
            println!("BOLT11 Invoice:\n\n{}", invoice);
        }
        WalletAction::Pay { bolt11 } => {
            println!("Initiating payment...");
            let result = wallet.pay_invoice(&bolt11, 1000, "cli-payment")?;
            println!("Payment Succeeded!");
            println!("  Preimage: {}", result.preimage_hex);
            println!("  Paid:     {} msats", result.total_msats_paid);
        }
    }
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
