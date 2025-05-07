mod usdt_blacklist_checker;
mod usdt_transfer;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::Local;
use ethers::{
    prelude::*,
    types::{Address, TransactionRequest},
    utils::format_ether,
};
use std::env;
use std::str::FromStr;
use teloxide::prelude::*;
use tokio::time;

#[derive(Debug)]
struct Config {
    to_address: String,
    contract_address: String,
    bot_token: String,
    chat_id: String,
    rpc_url: String,
    sender_private_key: String,
    sender_address: Address,
    checker_address: String,
    recipient_address: Address,
    check_interval_minutes: u64,
    min_balance_to_transfer: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let config = Config {
        to_address: std::env::var("TO_ADDRESS").context("Missing TO_ADDRESS")?,
        contract_address: std::env::var("CONTRACT_ADDRESS").context("Missing CONTRACT_ADDRESS")?,
        bot_token: std::env::var("BOT_TOKEN").context("Missing BOT_TOKEN")?,
        chat_id: std::env::var("CHAT_ID").context("Missing CHAT_ID")?,
        rpc_url: std::env::var("RPC_URL").context("Missing RPC_URL")?,
        sender_private_key: std::env::var("PRIVATE_KEY").context("Missing PRIVATE_KEY")?,
        checker_address: std::env::var("CHECKER_ADDRESS").context("Missing CHECKER_ADDRESS")?,
        sender_address: std::env::var("SENDER_ADDRESS")
            .context("Missing SENDER_ADDRESS")?
            .parse()?,
        recipient_address: std::env::var("RECIPIENT_ADDRESS")
            .context("Missing RECIPIENT_ADDRESS")?
            .parse()?,
        check_interval_minutes: std::env::var("CHECK_INTERVAL_MINUTES")
            .unwrap_or("1".to_string())
            .parse()?,
        min_balance_to_transfer: std::env::var("MIN_BALANCE_TO_TRANSFER")
            .unwrap_or("0.01".to_string())
            .parse()?,
    };



    let transfer_amount: u64 = env::var("TRANSFER_AMOUNT")
        .unwrap_or_else(|_| panic!("TRANSFER_AMOUNT not found in .env"))
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse TRANSFER_AMOUNT as u64"));

    let tx_hash = usdt_transfer::transfer_usdt(
        &config.rpc_url,
        &config.sender_private_key,
        &config.contract_address,
        &config.to_address,
        transfer_amount,
    )
        .await?;

    println!("Transaction successful with hash: {:?}", tx_hash);


    println!("Starting ETH balance monitor with config: {:#?}", config);

    let provider =
        Provider::<Http>::try_from(&config.rpc_url)?.interval(Duration::from_millis(500));
    let provider = Arc::new(provider);

    let wallet = config
        .sender_private_key
        .parse::<LocalWallet>()?
        .with_chain_id(1u64); //  chain_id = 1
                              // .with_chain_id(11155111u64); //  chain_id = 1

    let client = SignerMiddleware::new(provider.clone(), wallet);

    let mut interval = time::interval(Duration::from_secs(15));

    loop {
        interval.tick().await;

        // Address to check
        let address_to_check = &config.checker_address;

        // Check blacklist status
        match usdt_blacklist_checker::check_usdt_blacklist(address_to_check, &config.rpc_url).await
        {
            Ok(is_blacklisted) => {
                if is_blacklisted {
                    println!(
                        "ADDRESS {} IS {} BLACKLISTED BY Tether",
                        address_to_check,
                        if is_blacklisted { "" } else { "not " }
                    )
                }
            }

            Err(e) => eprintln!("Error checking blacklist status: {:?}", e),
        }

        if let Err(e) = check_and_transfer(&client, &config).await {
            eprintln!("[{}] Error: {}", Local::now(), e);
        }
    }
}

async fn check_and_transfer(
    client: &SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
    config: &Config,
) -> Result<()> {
    let now = Local::now();
    println!("\n[{}] Checking balance...", now);

    let gas_price = client.get_gas_price().await?;
    let gas_limit = U256::from(21_000u64);
    let gas_cost = gas_price
        .checked_mul(gas_limit)
        .context("Gas cost calculation overflow")?;

    let balance = client.get_balance(config.sender_address, None).await?;
    let balance_eth: f64 = format_ether(balance).parse()?;
    let gas_cost_eth: f64 = format_ether(gas_cost).parse()?;

    println!("[{}] Current balance: {:.6} ETH", now, balance_eth);
    println!("[{}] Estimated gas cost: {:.6} ETH", now, gas_cost_eth);

    if balance_eth < config.min_balance_to_transfer {
        println!(
            "[{}] Balance below minimum threshold ({:.6} ETH)",
            now, config.min_balance_to_transfer
        );
        return Ok(());
    }

    if balance <= gas_cost {
        println!("[{}] Insufficient balance to cover gas costs", now);
        return Ok(());
    }

    let transfer_amount = balance
        .checked_sub(gas_cost)
        .context("Transfer amount calculation error")?;
    let transfer_amount_eth: f64 = format_ether(transfer_amount).parse()?;

    println!(
        "[{}] Preparing to transfer {:.6} ETH",
        now, transfer_amount_eth
    );

    let tx = TransactionRequest::new()
        .to(config.recipient_address)
        .value(transfer_amount)
        .gas(gas_limit)
        .gas_price(gas_price)
        .from(config.sender_address);

    let pending_tx = client.send_transaction(tx, None).await?;
    println!("[{}] Transaction sent: {:?}", now, pending_tx.tx_hash());

    let receipt = pending_tx
        .confirmations(3)
        .await?
        .context("Transaction not confirmed")?;

    println!(
        "[{}] Transaction confirmed in block: {:?}",
        now, receipt.block_number
    );

    Ok(())
}
