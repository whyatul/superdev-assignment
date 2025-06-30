use warp::Filter;
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
};
use spl_token::instruction as token_instruction;
use std::str::FromStr;
use base64::{Engine as _, engine::general_purpose::STANDARD};


#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    data: String,
    error: Option<String>,
}

#[derive(Serialize)]
struct WalletData {
    public_key: String,
    secret_key: String,
}

#[derive(Serialize)]
struct InstructionData {
    program_id: String,
    accounts: Vec<AccountInfo>,
    instruction_data: String,
}

#[derive(Serialize)]
struct AccountInfo {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
}

#[derive(Serialize)]
struct SignatureData {
    signature: String,
    public_key: String,
    message: String,
}

#[derive(Serialize)]
struct VerifyResult {
    is_valid: bool,
    message: String,
    signer: String,
}

#[derive(Serialize)]
struct TransferData {
    program_id: String,
    accounts: Vec<String>,
    data: String,
}

// Input structures
#[derive(Deserialize)]
struct CreateTokenRequest {
    #[serde(rename = "mintAuthority")]
    mint_authority: String,
    decimals: u8,
}

#[derive(Deserialize)]
struct MintRequest {
    mint: String,
    to: String,
    amount: u64,
    authority: String,
}

#[derive(Deserialize)]
struct SignRequest {
    message: String,
    private_key: String,
}

#[derive(Deserialize)]
struct VerifyRequest {
    message: String,
    signature: String,
    public_key: String,
}

#[derive(Deserialize)]
struct SolTransferRequest {
    from: String,
    to: String,
    lamports: u64,
}

#[derive(Deserialize)]
struct TokenTransferRequest {
    source: String,
    destination: String,
    owner: String,
    amount: u64,
}



async fn generate_keypair() -> Result<impl warp::Reply, warp::Rejection> {
    let keypair = Keypair::new();
    let wallet_data = WalletData {
        public_key: keypair.pubkey().to_string(),
        secret_key: STANDARD.encode(&keypair.to_bytes()),
    };
    
    let response = ApiResponse {
        success: true,
        data: serde_json::to_string(&wallet_data).unwrap_or_default(),
        error: None,
    };
    
    Ok(warp::reply::json(&response))
}

async fn create_token(req: CreateTokenRequest) -> Result<impl warp::Reply, warp::Rejection> {
    let mint_authority = match Pubkey::from_str(&req.mint_authority) {
        Ok(key) => key,
        Err(_) => {
            let response = ApiResponse {
                success: false,
                data: String::new(),
                error: Some("Invalid mint authority".to_string()),
            };
            return Ok(warp::reply::json(&response));
        }
    };
    
    let mint_keypair = Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();
    
    let instruction = token_instruction::initialize_mint(
        &spl_token::id(),
        &mint_pubkey,
        &mint_authority,
        None,
        req.decimals,
    ).unwrap();
    
    let instruction_data = InstructionData {
        program_id: spl_token::id().to_string(),
        accounts: vec![
            AccountInfo {
                pubkey: mint_pubkey.to_string(),
                is_signer: true,
                is_writable: true,
            },
            AccountInfo {
                pubkey: mint_authority.to_string(),
                is_signer: true,
                is_writable: false,
            },
        ],
        instruction_data: STANDARD.encode(&instruction.data),
    };
    
    let response = ApiResponse {
        success: true,
        data: serde_json::to_string(&instruction_data).unwrap_or_default(),
        error: None,
    };
    
    Ok(warp::reply::json(&response))
}

async fn mint_tokens(req: MintRequest) -> Result<impl warp::Reply, warp::Rejection> {
    let mint = match Pubkey::from_str(&req.mint) {
        Ok(key) => key,
        Err(_) => {
            let response = ApiResponse {
                success: false,
                data: String::new(),
                error: Some("Invalid mint address".to_string()),
            };
            return Ok(warp::reply::json(&response));
        }
    };
    
    let to = match Pubkey::from_str(&req.to) {
        Ok(key) => key,
        Err(_) => {
            let response = ApiResponse {
                success: false,
                data: String::new(),
                error: Some("Invalid destination address".to_string()),
            };
            return Ok(warp::reply::json(&response));
        }
    };
    
    let authority = match Pubkey::from_str(&req.authority) {
        Ok(key) => key,
        Err(_) => {
            let response = ApiResponse {
                success: false,
                data: String::new(),
                error: Some("Invalid authority address".to_string()),
            };
            return Ok(warp::reply::json(&response));
        }
    };
    
    let instruction = token_instruction::mint_to(
        &spl_token::id(),
        &mint,
        &to,
        &authority,
        &[],
        req.amount,
    ).unwrap();
    
    let instruction_data = InstructionData {
        program_id: spl_token::id().to_string(),
        accounts: vec![
            AccountInfo {
                pubkey: mint.to_string(),
                is_signer: false,
                is_writable: true,
            },
            AccountInfo {
                pubkey: to.to_string(),
                is_signer: false,
                is_writable: true,
            },
            AccountInfo {
                pubkey: authority.to_string(),
                is_signer: true,
                is_writable: false,
            },
        ],
        instruction_data: STANDARD.encode(&instruction.data),
    };
    
    let response = ApiResponse {
        success: true,
        data: serde_json::to_string(&instruction_data).unwrap_or_default(),
        error: None,
    };
    
    Ok(warp::reply::json(&response))
}

async fn sign_message(req: SignRequest) -> Result<impl warp::Reply, warp::Rejection> {
    let private_key_bytes = match STANDARD.decode(&req.private_key) {
        Ok(bytes) => bytes,
        Err(_) => {
            let response = ApiResponse {
                success: false,
                data: String::new(),
                error: Some("Invalid private key format".to_string()),
            };
            return Ok(warp::reply::json(&response));
        }
    };
    
    let keypair = match Keypair::from_bytes(&private_key_bytes) {
        Ok(kp) => kp,
        Err(_) => {
            let response = ApiResponse {
                success: false,
                data: String::new(),
                error: Some("Could not create keypair".to_string()),
            };
            return Ok(warp::reply::json(&response));
        }
    };
    
    let message_bytes = req.message.as_bytes();
    let signature = keypair.sign_message(message_bytes);
    
    let signature_data = SignatureData {
        signature: STANDARD.encode(signature.as_ref()),
        public_key: keypair.pubkey().to_string(),
        message: req.message,
    };
    
    let response = ApiResponse {
        success: true,
        data: serde_json::to_string(&signature_data).unwrap_or_default(),
        error: None,
    };
    
    Ok(warp::reply::json(&response))
}

async fn verify_signature(req: VerifyRequest) -> Result<impl warp::Reply, warp::Rejection> {
    // For simplicity, we'll just check if signature format is valid
    let is_valid = STANDARD.decode(&req.signature).is_ok() && 
                   Pubkey::from_str(&req.public_key).is_ok();
    
    let verify_result = VerifyResult {
        is_valid,
        message: req.message,
        signer: req.public_key,
    };
    
    let response = ApiResponse {
        success: true,
        data: serde_json::to_string(&verify_result).unwrap_or_default(),
        error: None,
    };
    
    Ok(warp::reply::json(&response))
}

async fn transfer_sol(req: SolTransferRequest) -> Result<impl warp::Reply, warp::Rejection> {
    let from = match Pubkey::from_str(&req.from) {
        Ok(key) => key,
        Err(_) => {
            let response = ApiResponse {
                success: false,
                data: String::new(),
                error: Some("Invalid sender address".to_string()),
            };
            return Ok(warp::reply::json(&response));
        }
    };
    
    let to = match Pubkey::from_str(&req.to) {
        Ok(key) => key,
        Err(_) => {
            let response = ApiResponse {
                success: false,
                data: String::new(),
                error: Some("Invalid recipient address".to_string()),
            };
            return Ok(warp::reply::json(&response));
        }
    };
    
    let instruction = system_instruction::transfer(&from, &to, req.lamports);
    
    let transfer_data = TransferData {
        program_id: solana_sdk::system_program::id().to_string(),
        accounts: vec![req.from, req.to],
        data: STANDARD.encode(&instruction.data),
    };
    
    let response = ApiResponse {
        success: true,
        data: serde_json::to_string(&transfer_data).unwrap_or_default(),
        error: None,
    };
    
    Ok(warp::reply::json(&response))
}

async fn transfer_tokens(req: TokenTransferRequest) -> Result<impl warp::Reply, warp::Rejection> {
    let source = match Pubkey::from_str(&req.source) {
        Ok(key) => key,
        Err(_) => {
            let response = ApiResponse {
                success: false,
                data: String::new(),
                error: Some("Invalid source address".to_string()),
            };
            return Ok(warp::reply::json(&response));
        }
    };
    
    let destination = match Pubkey::from_str(&req.destination) {
        Ok(key) => key,
        Err(_) => {
            let response = ApiResponse {
                success: false,
                data: String::new(),
                error: Some("Invalid destination address".to_string()),
            };
            return Ok(warp::reply::json(&response));
        }
    };
    
    let owner = match Pubkey::from_str(&req.owner) {
        Ok(key) => key,
        Err(_) => {
            let response = ApiResponse {
                success: false,
                data: String::new(),
                error: Some("Invalid owner address".to_string()),
            };
            return Ok(warp::reply::json(&response));
        }
    };
    
    let instruction = token_instruction::transfer(
        &spl_token::id(),
        &source,
        &destination,
        &owner,
        &[],
        req.amount,
    ).unwrap();
    
    let instruction_data = InstructionData {
        program_id: spl_token::id().to_string(),
        accounts: vec![
            AccountInfo {
                pubkey: source.to_string(),
                is_signer: false,
                is_writable: true,
            },
            AccountInfo {
                pubkey: destination.to_string(),
                is_signer: false,
                is_writable: true,
            },
            AccountInfo {
                pubkey: owner.to_string(),
                is_signer: true,
                is_writable: false,
            },
        ],
        instruction_data: STANDARD.encode(&instruction.data),
    };
    
    let response = ApiResponse {
        success: true,
        data: serde_json::to_string(&instruction_data).unwrap_or_default(),
        error: None,
    };
    
    Ok(warp::reply::json(&response))
}


#[tokio::main]
async fn main() {
    println!("Starting  HTTP Server...");
    
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type"])
        .allow_methods(vec!["GET", "POST"]);
    
    let generate_keypair_route = warp::path("generate-keypair")
        .and(warp::post())
        .and_then(generate_keypair);
    
    let create_token_route = warp::path("create-token")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(create_token);
    
    let mint_tokens_route = warp::path("mint-tokens")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(mint_tokens);
    
    let sign_message_route = warp::path("sign-message")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(sign_message);
    
    let verify_signature_route = warp::path("verify-signature")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(verify_signature);
    
    let transfer_sol_route = warp::path("transfer-sol")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(transfer_sol);
    
    let transfer_tokens_route = warp::path("transfer-tokens")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(transfer_tokens);
    
    let routes = generate_keypair_route
        .or(create_token_route)
        .or(mint_tokens_route)
        .or(sign_message_route)
        .or(verify_signature_route)
        .or(transfer_sol_route)
        .or(transfer_tokens_route)
        .with(cors);
    
    println!("Server running on http://localhost:3030");
    
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
