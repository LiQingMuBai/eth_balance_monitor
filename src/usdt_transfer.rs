use ethers::{
    prelude::*,
    types::Address,
};
use std::convert::TryFrom;
use std::str::FromStr;


abigen!(
    USDTContract,
    r#"[
        function transfer(address to, uint256 value) public returns (bool)
    ]"#,
);


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