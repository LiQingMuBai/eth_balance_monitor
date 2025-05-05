use ethers::prelude::*;
use ethers::utils::parse_units;
use std::sync::Arc;

abigen!(
    ERC20,
    r#"[
        function transfer(address recipient, uint256 amount) external returns (bool)
    ]"#
);

pub struct UsdtTransfer {
    contract: ERC20<SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>>,
}

impl UsdtTransfer {
    pub async fn new(
        private_key: &str,
        provider_url: &str,
    ) -> anyhow::Result<Self> {
        let provider = Provider::<Http>::try_from(provider_url)?;
        let chain_id = provider.get_chainid().await?.as_u64();
        let wallet: LocalWallet = private_key.parse::<LocalWallet>()?.with_chain_id(chain_id);
        let client = SignerMiddleware::new(provider, wallet);
        let client = Arc::new(client);

        let contract = ERC20::new("0xdAC17F958D2ee523a2206206994597C13D831ec7".parse()?, client);
        Ok(Self { contract })
    }

    pub async fn transfer(&self, to: &str, amount_usdt: f64) -> anyhow::Result<TxHash> {
        let recipient: Address = to.parse()?;
        let amount = parse_units(amount_usdt, 6)?; // 6 decimals for USDT
        let tx = self.contract.transfer(recipient, amount).send().await?;
        Ok(tx.tx_hash())
    }
}
