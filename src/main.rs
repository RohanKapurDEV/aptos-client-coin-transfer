use anyhow::{Context, Result};
use aptos_sdk::coin_client::CoinClient;
use aptos_sdk::rest_client::{Client, FaucetClient};
use aptos_sdk::types::LocalAccount;
use once_cell::sync::Lazy;
use std::str::FromStr;
use url::Url;

// Use APTOS_NODE_URL environment variable to set the node URL or default to hardcoded value
static NODE_URL: Lazy<Url> = Lazy::new(|| {
    Url::from_str(
        std::env::var("APTOS_NODE_URL")
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("https://fullnode.devnet.aptoslabs.com"),
    )
    .unwrap()
});

// Use APTOS_FAUCET_URL environment variable to set the node URL or default to hardcoded value
static FAUCET_URL: Lazy<Url> = Lazy::new(|| {
    Url::from_str(
        std::env::var("APTOS_FAUCET_URL")
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("https://faucet.devnet.aptoslabs.com"),
    )
    .unwrap()
});

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize clients
    let rest_client = Client::new(NODE_URL.clone());
    let faucet_client = FaucetClient::new(FAUCET_URL.clone(), NODE_URL.clone());
    let coin_client = CoinClient::new(&rest_client);

    // Initialize local accounts for alice and bob
    // alice is marked as mutable since it needs to be for the coin_client.transfer call
    let mut alice = LocalAccount::generate(&mut rand::rngs::OsRng);
    let bob = LocalAccount::generate(&mut rand::rngs::OsRng);

    println!("\n===== Local Accounts =====");
    println!("Alice: {}", alice.address().to_hex_literal());
    println!("Bob: {}", bob.address().to_hex_literal());

    // Create and fund Alice's onchain account. Create Bob's onchain account
    faucet_client
        .fund(alice.address(), 20_000)
        .await
        .context("Failed to fund Alice")?;
    faucet_client
        .create_account(bob.address())
        .await
        .context("Failed to create onchain account for Bob")?;

    println!("\n===== Initial balances =====");
    println!(
        "Alice: {:?}",
        coin_client
            .get_account_balance(&alice.address())
            .await
            .context("Could not fetch Alice's balance")?
    );
    println!(
        "Bob: {:?}",
        coin_client
            .get_account_balance(&bob.address())
            .await
            .context("Could not fetch Bob's balance")?
    );

    // Transfer 1000 coins from Alice to Bob
    let tx_hash = coin_client
        .transfer(&mut alice, bob.address(), 1000, None)
        .await
        .context("Failed to transfer coins from Alice to Bob")?;

    rest_client
        .wait_for_transaction(&tx_hash)
        .await
        .context("Failed to wait for transaction")?;

    println!("\n===== Intermediate balances =====");
    println!(
        "Alice: {:?}",
        coin_client
            .get_account_balance(&alice.address())
            .await
            .context("Could not fetch Alice's balance")?
    );
    println!(
        "Bob: {:?}",
        coin_client
            .get_account_balance(&bob.address())
            .await
            .context("Could not fetch Bob's balance")?
    );

    // Transfer 1000 coins from Alice to Bob
    let tx_hash = coin_client
        .transfer(&mut alice, bob.address(), 1000, None)
        .await
        .context("Failed to transfer coins from Alice to Bob")?;

    rest_client
        .wait_for_transaction(&tx_hash)
        .await
        .context("Failed to wait for transaction")?;

    println!("\n===== Final balances =====");
    println!(
        "Alice: {:?}",
        coin_client
            .get_account_balance(&alice.address())
            .await
            .context("Could not fetch Alice's balance")?
    );
    println!(
        "Bob: {:?}",
        coin_client
            .get_account_balance(&bob.address())
            .await
            .context("Could not fetch Bob's balance")?
    );

    Ok(())
}
