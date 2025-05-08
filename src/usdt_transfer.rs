use ethers::{
    prelude::*,
    types::Address,
};
use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::Arc;
use anyhow::Context;
use chrono::Local;
use ethers::utils::format_ether;
use crate::Config;

abigen!(
    USDTContract,
    r#"[
        function transfer(address to, uint256 value) public returns (bool)
    ]"#,
);


pub async fn check_and_transfer(
    client: &SignerMiddleware<Arc<Provider<Http>>, LocalWallet>,
    config: &Config,
) -> anyhow::Result<()> {
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

pub async fn transfer_usdt(
    rpc_url: &str,
    private_key: &str,
    contract_address: &str,
    to_address: &str,
    amount: u64,
) -> Result<H256, Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let wallet: LocalWallet = private_key.parse()?;
    let client = SignerMiddleware::new(provider, wallet.with_chain_id(1u64));
    let contract_address: Address = contract_address.parse()?;
    let contract = USDTContract::new(contract_address, client.into());
    let to_address: Address = Address::from_str(to_address)?;
    let amount = U256::from(amount);
    let tx = contract.transfer(to_address, amount);
    let pending_tx = tx.send().await?;
    let receipt = pending_tx.confirmations(1).await?;
    match receipt {
        Some(receipt) => Ok(receipt.transaction_hash),
        None => Err("Transaction failed or was not included in a block".into()),
    }
}