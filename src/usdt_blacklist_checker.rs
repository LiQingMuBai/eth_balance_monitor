use web3::types::{Address, H256};
use web3::contract::{Contract, Options};
use web3::transports::Http;
use web3::Web3;
use std::env;

pub async fn check_usdt_blacklist(address_to_check: &str, eth_node_url: &str) -> web3::contract::Result<bool> {
    // Parse the address to check
    let address: Address = address_to_check
        .parse()
        .map_err(|_| web3::Error::InvalidResponse("Invalid address to check".to_string()))?;
    // USDT contract address (Ethereum mainnet)
    let usdt_contract_address: Address = "0xdac17f958d2ee523a2206206994597c13d831ec7"
        .parse()
        .map_err(|_| web3::Error::InvalidResponse("Invalid USDT contract address".to_string()))?;
    // Initialize Web3 client
    let transport = Http::new(eth_node_url)?;
    let web3 = Web3::new(transport);
    // USDT contract ABI (minimal, containing only the isBlackListed function)
    let abi = r#"
    [
        {
            "constant": true,
            "inputs": [
                {
                    "name": "_maker",
                    "type": "address"
                }
            ],
            "name": "isBlackListed",
            "outputs": [
                {
                    "name": "",
                    "type": "bool"
                }
            ],
            "type": "function"
        }
    ]
    "#;
    // Create contract instance
    let contract = Contract::from_json(
        web3.eth(),
        usdt_contract_address,
        abi.as_bytes(),
    )?;
    // Call the isBlackListed function
    let result: bool = contract
        .query("isBlackListed", (address,), None, Options::default(), None)
        .await?;
    Ok(result)
}