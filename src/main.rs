use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::Local;
use ethers::{
    prelude::*,
    types::{Address, TransactionRequest},
    utils::format_ether,
};
use tokio::time;

// 配置结构体
#[derive(Debug)]
struct Config {
    rpc_url: String,
    sender_private_key: String,
    sender_address: Address,
    recipient_address: Address,
    check_interval_minutes: u64,
    min_balance_to_transfer: f64, // 最小转账余额 (ETH)
}

#[tokio::main]
async fn main() -> Result<()> {
    // 加载环境变量
    dotenv::dotenv().ok();

    // 初始化配置
    let config = Config {
        rpc_url: std::env::var("RPC_URL").context("Missing RPC_URL")?,
        sender_private_key: std::env::var("PRIVATE_KEY").context("Missing PRIVATE_KEY")?,
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

    println!("Starting ETH balance monitor with config: {:#?}", config);

    // 创建提供者 (使用自动重试中间件)
    let provider = Provider::<Http>::try_from(&config.rpc_url)?
        .interval(Duration::from_millis(500));
    let provider = Arc::new(provider);

    // 创建钱包
    let wallet = config.sender_private_key
        .parse::<LocalWallet>()?
        .with_chain_id(1u64); // 主网 chain_id = 1
        // .with_chain_id(11155111u64); // 主网 chain_id = 1

    // 创建客户端 (钱包 + 提供者)
    let client = SignerMiddleware::new(provider.clone(), wallet);

    // 每隔15定时检查
    let mut interval = time::interval(Duration::from_secs(15));

    loop {
        interval.tick().await;

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

    // 1. 获取当前 gas 价格
    let gas_price = client.get_gas_price().await?;
    let gas_limit = U256::from(21_000u64); // 标准转账 gas 限制
    let gas_cost = gas_price.checked_mul(gas_limit)
        .context("Gas cost calculation overflow")?;

    // 2. 获取发送者余额
    let balance = client.get_balance(config.sender_address, None).await?;
    let balance_eth: f64 = format_ether(balance).parse()?;
    let gas_cost_eth: f64 = format_ether(gas_cost).parse()?;

    println!("[{}] Current balance: {:.6} ETH", now, balance_eth);
    println!("[{}] Estimated gas cost: {:.6} ETH", now, gas_cost_eth);

    // 3. 检查余额是否足够
    if balance_eth < config.min_balance_to_transfer {
        println!("[{}] Balance below minimum threshold ({:.6} ETH)", now, config.min_balance_to_transfer);
        return Ok(());
    }

    if balance <= gas_cost {
        println!("[{}] Insufficient balance to cover gas costs", now);
        return Ok(());
    }

    // 4. 计算可转账金额 (余额 - gas 费用)
    let transfer_amount = balance.checked_sub(gas_cost)
        .context("Transfer amount calculation error")?;
    let transfer_amount_eth: f64 = format_ether(transfer_amount).parse()?;

    println!("[{}] Preparing to transfer {:.6} ETH", now, transfer_amount_eth);

    // 5. 构建交易
    let tx = TransactionRequest::new()
        .to(config.recipient_address)
        .value(transfer_amount)
        .gas(gas_limit)
        .gas_price(gas_price)
        .from(config.sender_address);

    // 6. 发送交易
    let pending_tx = client.send_transaction(tx, None).await?;
    println!("[{}] Transaction sent: {:?}", now, pending_tx.tx_hash());

    // 7. 等待交易确认 (最多等待 3 个区块)
    let receipt = pending_tx
        .confirmations(3)
        .await?
        .context("Transaction not confirmed")?;

    println!("[{}] Transaction confirmed in block: {:?}", now, receipt.block_number);

    Ok(())
}
