use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "concilium cli")]
#[command(about = "concilium cli")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Get a new BLS wallet for use by network nodes")]
    GetNewNodeWallet {},
    #[command(about = "Get a new wallet based on ED25519 to send transactions between users")]
    GetNewUserWallet {},
    #[command(about = "Get transaction information")]
    GetTransactionInfo {
        #[arg(short, long)]
        txid: String,
    },
    #[command(about = "Check transaction status")]
    CheckTransactionStatus {
        #[arg(short, long)]
        txid: String,
    },
    #[command(about = "Submit transaction")]
    SendToAddress {
        #[arg(short, long)]
        sender_private_key: String,
        #[arg(short, long)]
        receiver_public_key: String,
        #[arg(short, long)]
        amount: f32,
    },
}