use std::{collections::HashMap, path::PathBuf};
use anyhow::{Context, Result};
use uuid::Uuid;
use auction_crypto::pedersen::PedersenGenerators;
use auction_client::{
    bid::{create_proof_of_opening, create_sealed_bid, BidSecret},
    client::AuctionClient,
    errors::ClientError,
    identity::{Identity, IdentityFile},
    verify::ClientVerifier,
};

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8080";

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let base_url = std::env::var("AUCTION_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.into());

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "generate-identity" => cmd_generate_identity(&args).await,
        "register"          => cmd_register(&args, &base_url).await,
        "login"             => cmd_login(&args, &base_url).await,
        "create-auction"    => cmd_create_auction(&args, &base_url).await,
        "list-auctions"     => cmd_list_auctions(&base_url).await,
        "open-auction"      => cmd_open_auction(&args, &base_url).await,
        "close-auction"     => cmd_close_auction(&args, &base_url).await,
        "place-bid"         => cmd_place_bid(&args, &base_url).await,
        "reveal-winner"     => cmd_reveal_winner(&args, &base_url).await,
        "submit-loser-proof"=> cmd_submit_loser_proof(&args, &base_url).await,
        "verify-auction"    => cmd_verify_auction(&args, &base_url).await,
        "show-board"        => cmd_show_board(&args, &base_url).await,
        _ => { print_usage(); Ok(()) }
    }
}

fn print_usage() {
    eprintln!("auction-cli <command> [args]");
    eprintln!("Commands:");
    eprintln!("  generate-identity <username> <output.json>");
    eprintln!("  register <identity.json> <password>");
    eprintln!("  login <identity.json> <password> -> token.txt");
    eprintln!("  create-auction <token> <title> <description> <min_bid> <max_bid> <step> <duration_seconds>");
    eprintln!("  list-auctions");
    eprintln!("  open-auction <token> <auction_id>");
    eprintln!("  close-auction <token> <auction_id>");
    eprintln!("  place-bid <token> <identity.json> <auction_id> <value> <secrets_dir>");
    eprintln!("  reveal-winner <token> <identity.json> <auction_id> <secrets_dir>");
    eprintln!("  submit-loser-proof <token> <identity.json> <auction_id> <secrets_dir>");
    eprintln!("  verify-auction <auction_id>");
    eprintln!("  show-board <auction_id>");
}

async fn cmd_generate_identity(args: &[String]) -> Result<()> {
    let username = args.get(2).context("missing username")?;
    let output = args.get(3).context("missing output path")?;
    let identity = Identity::generate(username.clone());
    let file = identity.to_file();
    let json = serde_json::to_string_pretty(&file)?;
    std::fs::write(output, json)?;
    println!("Identity generated: {}", identity.public_key_hex());
    Ok(())
}

async fn cmd_register(args: &[String], base_url: &str) -> Result<()> {
    let identity_path = args.get(2).context("missing identity path")?;
    let password = args.get(3).context("missing password")?;
    let identity = load_identity(identity_path)?;
    let client = AuctionClient::new(base_url);
    let resp = client.register(&identity.username, password, &identity.public_key_hex()).await?;
    println!("Registered: {} ({})", resp.username, resp.user_id);
    Ok(())
}

async fn cmd_login(args: &[String], base_url: &str) -> Result<()> {
    let identity_path = args.get(2).context("missing identity path")?;
    let password = args.get(3).context("missing password")?;
    let identity = load_identity(identity_path)?;
    let client = AuctionClient::new(base_url);
    let resp = client.login(&identity.username, password).await?;
    println!("{}", resp.jwt_token);
    Ok(())
}

async fn cmd_create_auction(args: &[String], base_url: &str) -> Result<()> {
    let token = args.get(2).context("missing token")?;
    let title = args.get(3).context("missing title")?;
    let description = args.get(4).context("missing description")?;
    let min_bid: u64 = args.get(5).context("missing min_bid")?.parse()?;
    let max_bid: Option<u64> = args.get(6).map(|s| s.parse().unwrap_or(0)).filter(|&v| v > 0);
    let step: u64 = args.get(7).context("missing step")?.parse()?;
    let duration: i64 = args.get(8).context("missing duration_seconds")?.parse()?;

    let client = AuctionClient::new(base_url).with_token(token.clone());
    let resp = client.create_auction(title, description, min_bid, max_bid, step, duration).await?;
    println!("Created auction: {} ({})", resp.title, resp.id);
    Ok(())
}

async fn cmd_list_auctions(base_url: &str) -> Result<()> {
    let client = AuctionClient::new(base_url);
    let resp = client.list_auctions().await?;
    for a in &resp.auctions {
        println!("{} | {} | {:?}", a.id, a.title, a.status);
    }
    println!("Total: {}", resp.total);
    Ok(())
}

async fn cmd_open_auction(args: &[String], base_url: &str) -> Result<()> {
    let token = args.get(2).context("missing token")?;
    let id: Uuid = args.get(3).context("missing auction_id")?.parse()?;
    let client = AuctionClient::new(base_url).with_token(token.clone());
    let resp = client.open_auction(id).await?;
    println!("Auction {} is now {:?}", resp.id, resp.status);
    Ok(())
}

async fn cmd_close_auction(args: &[String], base_url: &str) -> Result<()> {
    let token = args.get(2).context("missing token")?;
    let id: Uuid = args.get(3).context("missing auction_id")?.parse()?;
    let client = AuctionClient::new(base_url).with_token(token.clone());
    let resp = client.close_auction(id).await?;
    println!("Auction {} is now {:?}", resp.id, resp.status);
    Ok(())
}

async fn cmd_place_bid(args: &[String], base_url: &str) -> Result<()> {
    let token = args.get(2).context("missing token")?;
    let identity_path = args.get(3).context("missing identity path")?;
    let auction_id: Uuid = args.get(4).context("missing auction_id")?.parse()?;
    let value: u64 = args.get(5).context("missing value")?.parse()?;
    let secrets_dir = args.get(6).context("missing secrets_dir")?;

    let identity = load_identity(identity_path)?;
    let gens = PedersenGenerators::standard();
    let bid_data = create_sealed_bid(auction_id, value, &gens, &identity);

    let secret_path = format!("{secrets_dir}/{auction_id}.json");
    std::fs::create_dir_all(secrets_dir)?;
    std::fs::write(&secret_path, serde_json::to_string_pretty(&bid_data.secret)?)?;

    let client = AuctionClient::new(base_url).with_token(token.clone());
    let resp = client.submit_bid(auction_id, &bid_data.commitment_hex, &bid_data.bidder_signature_hex).await?;
    println!("Bid submitted: {} (BB seq {})", resp.bid_id, resp.bb_sequence);
    println!("Secret saved to: {secret_path}");
    Ok(())
}

async fn cmd_reveal_winner(args: &[String], base_url: &str) -> Result<()> {
    let token = args.get(2).context("missing token")?;
    let identity_path = args.get(3).context("missing identity path")?;
    let auction_id: Uuid = args.get(4).context("missing auction_id")?.parse()?;
    let secrets_dir = args.get(5).context("missing secrets_dir")?;

    let _identity = load_identity(identity_path)?;
    let secret = load_secret(secrets_dir, auction_id)?;
    let gens = PedersenGenerators::standard();
    let proof = create_proof_of_opening(&secret, &gens)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let proof_json = serde_json::to_string(&proof)?;

    let client = AuctionClient::new(base_url).with_token(token.clone());
    let bids = client.list_bids(auction_id).await?;
    let my_bid = bids.bids.iter()
        .find(|b| b.commitment_hex == secret.commitment_hex)
        .context("bid not found on server")?;

    let resp = client.reveal_winner(auction_id, my_bid.bid_id, secret.value, &proof_json).await?;
    println!("Winner revealed: value={}, reveal_id={}", resp.revealed_value, resp.reveal_id);
    Ok(())
}

async fn cmd_submit_loser_proof(args: &[String], base_url: &str) -> Result<()> {
    let token = args.get(2).context("missing token")?;
    let identity_path = args.get(3).context("missing identity path")?;
    let auction_id: Uuid = args.get(4).context("missing auction_id")?.parse()?;
    let secrets_dir = args.get(5).context("missing secrets_dir")?;

    let _identity = load_identity(identity_path)?;
    let secret = load_secret(secrets_dir, auction_id)?;
    let gens = PedersenGenerators::standard();
    let proof = create_proof_of_opening(&secret, &gens)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let proof_json = serde_json::to_string(&proof)?;

    let client = AuctionClient::new(base_url).with_token(token.clone());
    let bids = client.list_bids(auction_id).await?;
    let my_bid = bids.bids.iter()
        .find(|b| b.commitment_hex == secret.commitment_hex)
        .context("bid not found on server")?;

    let resp = client.submit_loser_proof(auction_id, my_bid.bid_id, secret.value, &proof_json).await?;
    println!("Loser proof submitted: proof_id={}, verified={}", resp.proof_id, resp.verified);
    Ok(())
}

async fn cmd_verify_auction(args: &[String], base_url: &str) -> Result<()> {
    let auction_id: Uuid = args.get(2).context("missing auction_id")?.parse()?;
    let client = AuctionClient::new(base_url);
    let verifier = ClientVerifier::from_server(&client).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let result = verifier.verify_auction(&client, auction_id).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("Auction {} verification:", auction_id);
    println!("  chain_integrity   : {}", result.chain_integrity.passed);
    println!("  server_signatures : {}", result.server_signatures.passed);
    println!("  winner_proof      : {}", result.winner_proof.passed);
    println!("  loser_proofs      : {}", result.loser_proofs.passed);
    println!("  fully_valid       : {}", result.fully_valid);
    if let Some(e) = &result.chain_integrity.error   { println!("    chain error: {e}"); }
    if let Some(e) = &result.winner_proof.error      { println!("    winner error: {e}"); }
    if let Some(e) = &result.loser_proofs.error      { println!("    loser error: {e}"); }
    Ok(())
}

async fn cmd_show_board(args: &[String], base_url: &str) -> Result<()> {
    let auction_id: Uuid = args.get(2).context("missing auction_id")?.parse()?;
    let client = AuctionClient::new(base_url);
    let board = client.get_bulletin_board(auction_id).await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("Bulletin board for {} ({} entries):", auction_id, board.total);
    println!("Head hash: {}", board.head_hash_hex);
    for e in &board.entries {
        println!("  [{}] {:?} | hash: {}...", e.sequence, e.entry_kind, &e.entry_hash_hex[..16]);
    }
    Ok(())
}

fn load_identity(path: &str) -> Result<Identity> {
    let json = std::fs::read_to_string(path)?;
    let file: IdentityFile = serde_json::from_str(&json)?;
    Identity::from_file(file).map_err(|e| anyhow::anyhow!("{e}"))
}

fn load_secret(secrets_dir: &str, auction_id: Uuid) -> Result<BidSecret> {
    let path = format!("{secrets_dir}/{auction_id}.json");
    let json = std::fs::read_to_string(&path)
        .with_context(|| format!("secret not found at {path}"))?;
    Ok(serde_json::from_str(&json)?)
}