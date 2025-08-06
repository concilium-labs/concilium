use clap::Parser;
use concilium_cli::{get_new_node_wallet_handler, get_new_user_wallet_handler, get_transaction_info_handler, send_to_address_handler};
use concilium_core::cli::{Commands, Cli};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("ENV File Not Found");
    let cli = Cli::parse();

    match cli.command {
        Commands::GetNewNodeWallet {} => {
            let (public_key, private_key) = get_new_node_wallet_handler().expect("failed to create new wallet");
            println!("\n==================================================");
            println!("\x1b[1;4m{}\x1b[0m", "Public Key:");
            println!("  {}\n", public_key);

            println!("\x1b[1;4m{}\x1b[0m", "Private Key:");
            println!("  {}", private_key);
            println!("==================================================\n");
        }
        Commands::GetNewUserWallet {} => {
            let (public_key, private_key) = get_new_user_wallet_handler();
            println!("\n==================================================");
            println!("\x1b[1;4m{}\x1b[0m", "Public Key:");
            println!("  {}\n", public_key);

            println!("\x1b[1;4m{}\x1b[0m", "Private Key:");
            println!("  {}", private_key);
            println!("==================================================\n");
        }
        Commands::GetTransactionInfo { txid } => {
            let response = get_transaction_info_handler(txid).await;
            
            match response {
                Ok(data) => {
                    if data.status == false {
                        println!("\n==================================================");
                        println!("\x1b[1;4;31m{}\x1b[0m", "Transaction not found");
                        println!("==================================================\n");
                    } else {
                        let transaction = data.transaction.expect("failed to read transaction");
                        println!("\n==================================================");
                        
                        println!("\x1b[1;4m{}\x1b[0m", "TXID:");
                        println!("  {}\n", transaction.txid);
                        
                        println!("\x1b[1;4m{}\x1b[0m", "From:");
                        println!("  {}\n", transaction.from);

                        println!("\x1b[1;4m{}\x1b[0m", "Signature:");
                        println!("  {}\n", transaction.signature);

                        println!("\x1b[1;4m{}\x1b[0m", "Nonce:");
                        println!("  {}\n", transaction.nonce);

                        println!("\x1b[1;4m{}\x1b[0m", "Created At:");
                        println!("  {}\n", transaction.created_at);

                        println!("\x1b[1;4m{}\x1b[0m", "VIN:");
                        println!("{}\n", serde_json::to_string_pretty(&transaction.vin).expect("failed to read vin"));
                        
                        println!("\x1b[1;4m{}\x1b[0m", "VOUT:");
                        println!("{}", serde_json::to_string_pretty(&transaction.vout).expect("failed to read vout"));
                        println!("==================================================\n");
                    }
                },
                Err(e) => println!("{}", e.get_message())
            }
        }
        Commands::CheckTransactionStatus { txid } => {
            let response = get_transaction_info_handler(txid).await;
            
            match response {
                Ok(data) => {
                    if data.status == false {
                        println!("\n==================================================");
                        println!("\x1b[1;4;31m{}\x1b[0m", "Unsuccessful transaction");
                        println!("==================================================\n");
                    } else {
                        println!("\n==================================================");
                        println!("\x1b[1;4;32m{}\x1b[0m", "Successful transaction");
                        println!("==================================================\n");
                    }
                },
                Err(_) => {
                    println!("\n==================================================");
                    println!("\x1b[1;4;31m{}\x1b[0m", "Unsuccessful transaction");
                    println!("==================================================\n");
                }
            }
        }
        Commands::SendToAddress {sender_private_key, receiver_public_key, amount} => {
            match send_to_address_handler(sender_private_key, receiver_public_key, amount).await {
                Ok(data) => {
                    println!("\n==================================================");
                    println!("\x1b[1;4;32m{}\x1b[0m\n", "Successful transaction");

                    println!("\x1b[1;4m{}\x1b[0m", "TXID:");
                    println!("  {}\n", data.txid);

                    println!("\x1b[1;4m{}\x1b[0m", "Accreditation Council Aggregated Signature:");
                    println!("  {}\n", data.accreditation_council_aggregated_signature);

                    println!("\x1b[1;4m{}\x1b[0m", "Broadcast Aggregated Signature:");
                    println!("  {}", data.broadcast_aggregated_signature);
                    println!("==================================================\n");
                },
                Err(e) => {
                    println!("\n==================================================");
                    println!("\x1b[1;4;31m{}\x1b[0m\n", "Unsuccessful transaction");
                    println!("\x1b[1;4m{}\x1b[0m", "Message:");
                    println!("  {}", e.get_message());
                    println!("==================================================\n");
                }
            }
        }
    }
}
