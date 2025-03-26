use dotenv::dotenv;
use std::env;
use web3::{
    transports::Http,
    types::{Address, Bytes, H160, TransactionParameters, U256},
    Web3,
};
use secp256k1::SecretKey;

#[tokio::main]
async fn main() -> web3::Result<()> {
    // Load environment variables
    dotenv().ok();
    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");
    let private_key_hex = env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set");
    let recipient = env::var("RECIPIENT").expect("RECIPIENT must be set");
    let amount = U256::from_dec_str(&env::var("AMOUNT").expect("AMOUNT must be set")).expect("Invalid amount");
    let erc20_token = env::var("ERC20_TOKEN_ADDR").expect("ERC20_TOKEN_ADDR must be set");

    // Connect to Ethereum node
    let transport = Http::new(&rpc_url)?;
    let web3 = Web3::new(transport);

    // Convert private key from hex to SecretKey
    let private_key_bytes = hex::decode(&private_key_hex).expect("Invalid private key hex");
    let secret_key = SecretKey::from_slice(&private_key_bytes).expect("Invalid private key");

    // Get the sender's address from private key
    let secp = secp256k1::Secp256k1::new();
    let public_key = secp256k1::PublicKey::from_secret_key(&secp, &secret_key);
    let sender = public_key.serialize_uncompressed();
    let sender_address: H160 = H160::from_slice(&keccak256(&sender[1..])[12..]);
    println!("sender: {:?}", sender_address);

    // ERC20 token contract address
    let token_address: Address = erc20_token.parse().expect("Invalid token address");
    let recipient_address: Address = recipient.parse().expect("Invalid recipient address");

    // Get the sender's nonce
    let nonce = web3.eth().transaction_count(sender_address, None).await?;

    // Get current gas price
    let gas_price = web3.eth().gas_price().await?;

    // Create transfer function data
    let transfer_function = "a9059cbb"; // keccak256("transfer(address,uint256)")
    let mut data = String::from(transfer_function);
    // Pad recipient address to 32 bytes
    data.push_str(&format!("000000000000000000000000{:x}", recipient_address));
    // Pad amount to 32 bytes
    data.push_str(&format!("{:064x}", amount));
    
    // Create transaction
    let tx = TransactionParameters {
        nonce: Some(nonce),
        to: Some(token_address),
        value: U256::zero(),
        gas_price: Some(gas_price),
        gas: U256::from(100000), // Typical gas limit for ERC20 transfers
        data: Bytes::from(hex::decode(&data).expect("Invalid data")),
        chain_id: Some(11155111), // Sepolia chain ID
        ..Default::default()
    };
    println!("tx: {:?}", tx);

    // Sign and send transaction
    let signed = web3.accounts().sign_transaction(tx, &secret_key).await?;
    let result = web3.eth().send_raw_transaction(signed.raw_transaction).await?;

    println!("Transaction sent! Hash: {:?}", result);
    Ok(())
}

fn keccak256(bytes: &[u8]) -> [u8; 32] {
    use tiny_keccak::{Hasher, Keccak};
    let mut output = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(bytes);
    hasher.finalize(&mut output);
    output
}
