use ethers::prelude::*;               // Ethereum interaction via ethers-rs
use std::sync::Arc;                   // Thread-safe reference counting
use std::time::Duration;              // For delay between checks
use dotenv::dotenv;                   // Load environment variables from .env
use std::env;                         // Access environment variables
use chrono::Local;

macro_rules! println_time {
    ($($arg:tt)*) => {
        println!("[{}] {}", Local::now().format("%Y-%m-%d %H:%M:%S"), format_args!($($arg)*))
    };
}

// Define the USDT contract with only the isBlackListed function
abigen!(
    UsdtContract,
    r#"[function isBlackListed(address) view returns (bool)]"#,
);

// USDT contract address on Ethereum Mainnet
const USDT_CONTRACT_ADDRESS: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env configuration
    dotenv().ok();
    // Load environment variables
    let provider_url = env::var("ETH_RPC_URL")?;
    let telegram_bot_token = env::var("TELEGRAM_BOT_TOKEN")?;
    let telegram_chat_id = env::var("TELEGRAM_CHAT_ID")?;
    let target_address = env::var("TARGET_ADDRESS")?;

    // Connect to Ethereum provider (e.g., Infura)
    let provider = Provider::<Http>::try_from(provider_url)?;
    let client = Arc::new(provider);

    // Instantiate the USDT contract
    let usdt_address: Address = USDT_CONTRACT_ADDRESS.parse()?;
    let contract = UsdtContract::new(usdt_address, client.clone());

    // Parse the target Ethereum address to monitor
    let target: Address = target_address.parse()?;

    // Check initial blacklist status
    let mut last_status = contract.is_black_listed(target).call().await?;
    println_time!("Initial blacklist status: {}", last_status);


    // Periodically check the blacklist status
    loop {
        let current_status = contract.is_black_listed(target).call().await?;
        if current_status {
            println_time!("Address {} remains on USDT's blacklist.", target);
        }
        if !current_status {
            println_time!("Address {} has been removed from USDT blacklist", target);
            let msg = format!("🚨 Address {} has been unblacklisted by USDT contract!", target);
            send_telegram_message(&telegram_bot_token, &telegram_chat_id, &msg).await?;
        }
        // If address was blacklisted before and now it's not, send Telegram notification
        if last_status && !current_status {
            println_time!("Address {} has been removed from USDT blacklist", target);
            let msg = format!("🚨 Address {} has been unblacklisted by USDT contract!", target);
            send_telegram_message(&telegram_bot_token, &telegram_chat_id, &msg).await?;
        }
        // Update status for next loop iteration
        last_status = current_status;

        // Wait 30 seconds before checking again
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

// Function to send a message via Telegram Bot
async fn send_telegram_message(token: &str, chat_id: &str, text: &str) -> Result<(), reqwest::Error> {
    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage",
        token
    );

    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "text": text
        }))
        .send()
        .await?;

    // Print result
    if res.status().is_success() {
        println_time!("✅ Telegram message sent.");
    } else {
        eprintln!("❌ Failed to send Telegram message: {:?}", res.text().await);
    }

    Ok(())
}

// mod usdt_blacklist_checker;
// mod usdt_transfer;
// mod telegram;
// use std::sync::Arc;
// use std::time::Duration;
// use anyhow::{Context, Result};
// use chrono::Local;
// use ethers::{
//     prelude::*,
//     types::{Address, TransactionRequest},
//     utils::format_ether,
// };
// use std::env;
// use std::str::FromStr;
// use teloxide::prelude::*;
// use tokio::time;
// use telegram::TelegramBot;
//
// use reqwest::Client;
// use serde::Serialize;
// use std::error::Error;
// #[derive(Debug)]
// struct Config {
//     to_address: String,
//     contract_address: String,
//     bot_token: String,
//     chat_id: String,
//     rpc_url: String,
//     sender_private_key: String,
//     sender_address: Address,
//     checker_address: String,
//     recipient_address: Address,
//     check_interval_minutes: u64,
//     min_balance_to_transfer: f64,
// }
//
// #[derive(Serialize)]
// struct TelegramMessage {
//     chat_id: String,
//     text: String,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     parse_mode: Option<String>,
// }
//
//
// macro_rules! send_telegram {
//     ($bot_token:expr, $chat_id:expr, $text:expr) => {
//         send_telegram_message($bot_token, $chat_id, $text, None).await
//     };
//     ($bot_token:expr, $chat_id:expr, $text:expr, $parse_mode:expr) => {
//         send_telegram_message($bot_token, $chat_id, $text, Some($parse_mode.to_string())).await
//     };
// }
//
// async fn send_telegram_message(
//     bot_token: &str,
//     chat_id: &str,
//     text: &str,
//     parse_mode: Option<String>,
// ) -> Result<(), Box<dyn Error>> {
//     let client = Client::new();
//     let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);
//
//     let message = TelegramMessage {
//         chat_id: chat_id.to_string(),
//         text: text.to_string(),
//         parse_mode,
//     };
//
//     let response = client
//         .post(&url)
//         .json(&message)
//         .send()
//         .await?;
//
//     if !response.status().is_success() {
//         let error_text = response.text().await?;
//         return Err(format!("Telegram API error: {}", error_text).into());
//     }
//
//     Ok(())
// }
//
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     dotenv::dotenv().ok();
//     let config = Config {
//         to_address: std::env::var("TO_ADDRESS").context("Missing TO_ADDRESS")?,
//         contract_address: std::env::var("CONTRACT_ADDRESS").context("Missing CONTRACT_ADDRESS")?,
//         bot_token: std::env::var("BOT_TOKEN").context("Missing BOT_TOKEN")?,
//         chat_id: std::env::var("CHAT_ID").context("Missing CHAT_ID")?,
//         rpc_url: std::env::var("RPC_URL").context("Missing RPC_URL")?,
//         sender_private_key: std::env::var("PRIVATE_KEY").context("Missing PRIVATE_KEY")?,
//         checker_address: std::env::var("CHECKER_ADDRESS").context("Missing CHECKER_ADDRESS")?,
//         sender_address: std::env::var("SENDER_ADDRESS")
//             .context("Missing SENDER_ADDRESS")?
//             .parse()?,
//         recipient_address: std::env::var("RECIPIENT_ADDRESS")
//             .context("Missing RECIPIENT_ADDRESS")?
//             .parse()?,
//         check_interval_minutes: std::env::var("CHECK_INTERVAL_MINUTES")
//             .unwrap_or("1".to_string())
//             .parse()?,
//         min_balance_to_transfer: std::env::var("MIN_BALANCE_TO_TRANSFER")
//             .unwrap_or("0.01".to_string())
//             .parse()?,
//     };
//     let transfer_amount: u64 = env::var("TRANSFER_AMOUNT")
//         .unwrap_or_else(|_| panic!("TRANSFER_AMOUNT not found in .env"))
//         .parse()
//         .unwrap_or_else(|_| panic!("Failed to parse TRANSFER_AMOUNT as u64"));
//     let tx_hash = usdt_transfer::transfer_usdt(
//         &config.rpc_url,
//         &config.sender_private_key,
//         &config.contract_address,
//         &config.to_address,
//         transfer_amount,
//     )
//         .await?;
//
//     let provider =
//         Provider::<Http>::try_from(&config.rpc_url)?.interval(Duration::from_millis(500));
//     let provider = Arc::new(provider);
//     let wallet = config
//         .sender_private_key
//         .parse::<LocalWallet>()?
//         .with_chain_id(1u64); //  chain_id = 1
//                               // .with_chain_id(11155111u64); //  chain_id = 1
//     let client = SignerMiddleware::new(provider.clone(), wallet);
//     let mut interval = time::interval(Duration::from_secs(5));
//     // let bot = TelegramBot::new(config.bot_token.to_string(), config.chat_id.to_string());
//     loop {
//         interval.tick().await;
//
//         // Address to check
//         let address_to_check = &config.checker_address;
//
//         // Check blacklist status
//         match usdt_blacklist_checker::check_usdt_blacklist(address_to_check, &config.rpc_url).await
//         {
//             Ok(is_blacklisted) => {
//                 if is_blacklisted {
//                     println!(
//                         "ADDRESS {} IS {} BLACKLISTED BY Tether",
//                         address_to_check,
//                         if is_blacklisted { "" } else { "not " }
//                     )
//                 }else{
//                     send_telegram!(&config.bot_token, &config.chat_id, "Hello from Rust macro!");
//                     // bot.send_message("ADDRESS IS UNLOCKED ,PLEASE CHECK").await?;
//                 }
//             }
//             Err(e) => eprintln!("Error checking blacklist status: {:?}", e),
//         }
//         if let Err(e) = usdt_transfer::check_and_transfer(
//             &client,
//             &config
//         ).await {
//             eprintln!("[{}] Error: {}", Local::now(), e);
//         }
//     }
// }
