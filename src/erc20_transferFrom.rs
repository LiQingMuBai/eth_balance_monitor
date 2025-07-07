// src/erc20_transferFrom.rs

use ethers::{
    abi::Abi, // Correct import for Abi
    contract::Contract,
    core::types::{Address, U256},
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    middleware::SignerMiddleware, // Correct import for SignerMiddleware
};
use std::sync::Arc;
abigen!(
    USDTContract,
    r#"[
        function transferFrom(address from, address to, uint256 value) public returns (bool)
    ]"#,
);
/// Executes an ERC-20 transferFrom transaction.
///
/// # Arguments
/// * `rpc_url` - The URL of the Ethereum RPC node (e.g., Infura, Alchemy).
/// * `private_key_spender` - The private key of the account that will call transferFrom (the spender).
///   This account must have been APPROVED by `sender_address`.
/// * `contract_address` - The address of the ERC-20 token contract (e.g., USDT).
/// * `sender_address` - The address from which tokens will be transferred. This address
///   must have previously approved `private_key_spender` to spend its tokens.
/// * `recipient_address` - The address to which tokens will be sent.
/// * `amount` - The amount of tokens to transfer (in the token's smallest unit, considering decimals).
/// * `chain_id` - The chain ID of the network (e.g., 1 for Ethereum Mainnet).
///
/// # Returns
/// A `Result` indicating success or failure. On success, it returns the transaction hash.
pub async fn execute_transfer_from(
    rpc_url: &str,
    private_key_spender: &str,
    contract_address: Address,
    sender_address: Address,
    recipient_address: Address,
    amount: U256,
    chain_id: u64,
) -> Result<ethers::core::types::H256, Box<dyn std::error::Error>> {



    // 配置以太坊提供者（替换为你的节点URL，例如Infura或Alchemy）
    let provider = Provider::<Http>::try_from(rpc_url)?;

    // 发送者的私钥（替换为实际的私钥，注意安全！）
    let private_key = "";
    // let wallet = private_key.parse::<LocalWallet>()?.with_chain_id(1u64); // 主网Chain ID为1
    let wallet = private_key.parse::<LocalWallet>()?.with_chain_id(1u64); // 主网Chain ID为11155111

    // 创建签名中间件
    let client = SignerMiddleware::new(provider, wallet);

    // USDT合约地址（以太坊主网USDT地址）
    let usdt_address = Address::from_str("0x779877A7B0D9E8603169DdbD7836e478b4624789")?;

    // 实例化USDT合约
    let contract = USDTContract::new(usdt_address, client.into());

    // 定义转账参数
    let from_address = Address::from_str("")?; // 资金来源地址
    let to_address = Address::from_str("")?; // 目标地址
    let amount = U256::from(10000000000000000000u64); // 转账金额（以wei为单位，USDT有6位小数，例如1 USDT = 1_000_000）

    // 调用transferFrom方法
    let binding = contract
        .transfer_from(from_address, to_address, amount);
    let tx = binding
        .send()
        .await?;

    // 等待交易确认
    let receipt = tx
        .await?
        .ok_or_else(|| "Transaction receipt not found")?;
}

/// Executes an ERC-20 approve transaction.
/// This function is typically called by the token owner (SENDER_ADDRESS) to allow
/// a 'spender' address (PRIVATE_KEY_SPENDER) to transfer tokens on their behalf.
///
/// # Arguments
/// * `rpc_url` - The URL of the Ethereum RPC node.
/// * `private_key_owner` - The private key of the account that owns the tokens (the sender).
/// * `contract_address` - The address of the ERC-20 token contract.
/// * `spender_address` - The address that will be granted permission to spend tokens.
/// * `amount` - The maximum amount of tokens the spender is allowed to transfer (in smallest unit).
/// * `chain_id` - The chain ID of the network.
///
/// # Returns
/// A `Result` indicating success or failure. On success, it returns the transaction hash.
pub async fn approve_spender(
    rpc_url: &str,
    private_key_owner: &str,
    contract_address: Address,
    spender_address: Address,
    amount: U256,
    chain_id: u64,
) -> Result<ethers::core::types::H256, Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let client = Arc::new(provider);

    let wallet: LocalWallet = private_key_owner.parse()?.with_chain_id(chain_id);
    let client_with_signer = Arc::new(SignerMiddleware::new(client.clone(), wallet.clone()));

    let abi_json = r#"[
        {
            "inputs": [
                {"internalType":"address","name":"spender","type":"address"},
                {"internalType":"uint256","name":"amount","type":"uint256"}
            ],
            "name":"approve",
            "outputs":[{"internalType":"bool","name":"","type":"bool"}],
            "stateMutability":"nonpayable",
            "type":"function"
        }
    ]"#;

    let contract_abi: Abi = abi_json.parse()?;
    let contract = Contract::new(contract_address, contract_abi, client_with_signer.clone());

    println!(
        "\nAttempting to call approve:\n  Owner: {}\n  Spender: {}\n  Amount (in smallest units): {}",
        wallet.address(),
        spender_address,
        amount
    );

    let call = contract
        .method::<_, bool>("approve", (spender_address, amount))?;

    let pending_tx = call.send().await?;
    let tx_hash = *pending_tx.tx_hash();

    println!("Approve transaction sent! Hash: {:?}", tx_hash);

    let receipt = pending_tx
        .await?
        .ok_or("Approve transaction receipt not found")?;

    println!("Approve transaction confirmed! Receipt: {:?}", receipt);

    if receipt.status == Some(1.into()) {
        println!("Approve successful!");
        Ok(tx_hash)
    } else {
        Err(format!("Approve transaction failed with status: {:?}", receipt.status).into())
    }
}