use warp::Filter;
use serde::{Deserialize, Serialize};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer, Signature},
    system_instruction,
};
use spl_token::instruction as token_instruction;
use spl_associated_token_account;
use std::str::FromStr;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use bs58;
use std::env;


#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
struct KeypairData {
    pubkey: String,
    secret: String,
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
struct VerifyData {
    valid: bool,
    message: String,
    pubkey: String,
}

#[derive(Serialize)]
struct SolTransferData {
    program_id: String,
    accounts: Vec<String>,
    instruction_data: String,
}

#[derive(Serialize)]
struct TokenTransferData {
    program_id: String,
    accounts: Vec<TokenAccountInfo>,
    instruction_data: String,
}

#[derive(Serialize)]
struct TokenAccountInfo {
    pubkey: String,
    #[serde(rename = "isSigner")]
    is_signer: bool,
}


#[derive(Deserialize)]
struct CreateTokenRequest {
    #[serde(rename = "mintAuthority")]
    mint_authority: String,
    mint: String,
    decimals: u8,
}

#[derive(Deserialize)]
struct MintTokenRequest {
    mint: String,
    destination: String,
    authority: String,
    amount: u64,
}

#[derive(Deserialize)]
struct SignMessageRequest {
    message: String,
    secret: String,
}

#[derive(Deserialize)]
struct VerifyMessageRequest {
    message: String,
    signature: String,
    pubkey: String,
}

#[derive(Deserialize)]
struct SendSolRequest {
    from: String,
    to: String,
    lamports: u64,
}

#[derive(Deserialize)]
struct SendTokenRequest {
    destination: String,
    mint: String,
    owner: String,
    amount: u64,
}


type ApiResult = Result<Box<dyn warp::Reply>, warp::Rejection>;

fn success_response<T: Serialize>(data: T) -> Box<dyn warp::Reply> {
    Box::new(warp::reply::with_status(
        warp::reply::json(&ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        }),
        warp::http::StatusCode::OK,
    ))
}

fn error_response(message: &str) -> Box<dyn warp::Reply> {
    Box::new(warp::reply::with_status(
        warp::reply::json(&ApiResponse::<()> {
            success: false,
            data: None,
            error: Some(message.to_string()),
        }),
        warp::http::StatusCode::BAD_REQUEST,
    ))
}

async fn generate_keypair() -> ApiResult {
    let keypair = Keypair::new();
    
    let response_data = KeypairData {
        pubkey: keypair.pubkey().to_string(),
        secret: bs58::encode(&keypair.to_bytes()).into_string(),
    };
    
    Ok(success_response(response_data))
}

async fn create_token(req: CreateTokenRequest) -> ApiResult {
    
    let mint_authority = match Pubkey::from_str(&req.mint_authority) {
        Ok(pubkey) => pubkey,
        Err(_) => return Ok(error_response("Invalid mint authority address")),
    };
    
    let mint_pubkey = match Pubkey::from_str(&req.mint) {
        Ok(pubkey) => pubkey,
        Err(_) => return Ok(error_response("Invalid mint address")),
    };
    
    let instruction = match token_instruction::initialize_mint(
        &spl_token::id(),
        &mint_pubkey,
        &mint_authority,
        None, 
        req.decimals,
    ) {
        Ok(instruction) => instruction,
        Err(_) => return Ok(error_response("Failed to create mint instruction")),
    };
    
    let response_data = InstructionData {
        program_id: spl_token::id().to_string(),
        accounts: vec![
            AccountInfo {
                pubkey: mint_pubkey.to_string(),
                is_signer: true,
                is_writable: true,
            },
            AccountInfo {
                pubkey: solana_sdk::sysvar::rent::id().to_string(),
                is_signer: false,
                is_writable: false,
            },
        ],
        instruction_data: STANDARD.encode(&instruction.data),
    };
    
    Ok(success_response(response_data))
}


async fn mint_token(req: MintTokenRequest) -> ApiResult {
    // Validate all addresses
    let mint = match Pubkey::from_str(&req.mint) {
        Ok(pubkey) => pubkey,
        Err(_) => return Ok(error_response("Invalid mint address")),
    };
    
    let destination = match Pubkey::from_str(&req.destination) {
        Ok(pubkey) => pubkey,
        Err(_) => return Ok(error_response("Invalid destination address")),
    };
    
    let authority = match Pubkey::from_str(&req.authority) {
        Ok(pubkey) => pubkey,
        Err(_) => return Ok(error_response("Invalid authority address")),
    };
    
  
    let instruction = match token_instruction::mint_to(
        &spl_token::id(),
        &mint,
        &destination,
        &authority,
        &[],
        req.amount,
    ) {
        Ok(instruction) => instruction,
        Err(_) => return Ok(error_response("Failed to create mint instruction")),
    };
    
    let response_data = InstructionData {
        program_id: spl_token::id().to_string(),
        accounts: vec![
            AccountInfo {
                pubkey: mint.to_string(),
                is_signer: false,
                is_writable: true,
            },
            AccountInfo {
                pubkey: destination.to_string(),
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
    
    Ok(success_response(response_data))
}


async fn sign_message(req: SignMessageRequest) -> ApiResult {
    
    let secret_bytes = match bs58::decode(&req.secret).into_vec() {
        Ok(bytes) => bytes,
        Err(_) => return Ok(error_response("Invalid secret key format")),
    };
    
 
    let keypair = match Keypair::from_bytes(&secret_bytes) {
        Ok(kp) => kp,
        Err(_) => return Ok(error_response("Invalid secret key")),
    };
    

    let message_bytes = req.message.as_bytes();
    let signature = keypair.sign_message(message_bytes);
    
    let response_data = SignatureData {
        signature: STANDARD.encode(signature.as_ref()),
        public_key: keypair.pubkey().to_string(),
        message: req.message,
    };
    
    Ok(success_response(response_data))
}


async fn verify_message(req: VerifyMessageRequest) -> ApiResult {
    
    let pubkey = match Pubkey::from_str(&req.pubkey) {
        Ok(pk) => pk,
        Err(_) => return Ok(error_response("Invalid public key")),
    };
    
    
    let signature_bytes = match STANDARD.decode(&req.signature) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(error_response("Invalid signature format")),
    };
    
    
    let signature = match Signature::try_from(signature_bytes.as_slice()) {
        Ok(sig) => sig,
        Err(_) => return Ok(error_response("Invalid signature")),
    };
    
    
    let message_bytes = req.message.as_bytes();
    let is_valid = signature.verify(&pubkey.to_bytes(), message_bytes);
    
    let response_data = VerifyData {
        valid: is_valid,
        message: req.message,
        pubkey: req.pubkey,
    };
    
    Ok(success_response(response_data))
}


async fn send_sol(req: SendSolRequest) -> ApiResult {
   
    let from = match Pubkey::from_str(&req.from) {
        Ok(pubkey) => pubkey,
        Err(_) => return Ok(error_response("Invalid from address")),
    };
    
    let to = match Pubkey::from_str(&req.to) {
        Ok(pubkey) => pubkey,
        Err(_) => return Ok(error_response("Invalid to address")),
    };
    

    if req.lamports == 0 {
        return Ok(error_response("Lamports amount must be greater than 0"));
    }
    
  
    let instruction = system_instruction::transfer(&from, &to, req.lamports);
    
    let response_data = SolTransferData {
        program_id: solana_sdk::system_program::id().to_string(),
        accounts: vec![req.from, req.to],
        instruction_data: STANDARD.encode(&instruction.data),
    };
    
    Ok(success_response(response_data))
}


async fn send_token(req: SendTokenRequest) -> ApiResult {
   
    let destination = match Pubkey::from_str(&req.destination) {
        Ok(pubkey) => pubkey,
        Err(_) => return Ok(error_response("Invalid destination address")),
    };
    
    let mint = match Pubkey::from_str(&req.mint) {
        Ok(pubkey) => pubkey,
        Err(_) => return Ok(error_response("Invalid mint address")),
    };
    
    let owner = match Pubkey::from_str(&req.owner) {
        Ok(pubkey) => pubkey,
        Err(_) => return Ok(error_response("Invalid owner address")),
    };
    
    
    if req.amount == 0 {
        return Ok(error_response("Amount must be greater than 0"));
    }
    
  
    let source_ata = spl_associated_token_account::get_associated_token_address(&owner, &mint);
    let dest_ata = spl_associated_token_account::get_associated_token_address(&destination, &mint);
    
    
    let instruction = match token_instruction::transfer(
        &spl_token::id(),
        &source_ata,
        &dest_ata,
        &owner,
        &[],
        req.amount,
    ) {
        Ok(instruction) => instruction,
        Err(_) => return Ok(error_response("Failed to create transfer instruction")),
    };
    
    let response_data = TokenTransferData {
        program_id: spl_token::id().to_string(),
        accounts: vec![
            TokenAccountInfo {
                pubkey: source_ata.to_string(),
                is_signer: false,
            },
            TokenAccountInfo {
                pubkey: dest_ata.to_string(),
                is_signer: false,
            },
            TokenAccountInfo {
                pubkey: owner.to_string(),
                is_signer: true,
            },
        ],
        instruction_data: STANDARD.encode(&instruction.data),
    };
    
    Ok(success_response(response_data))
}



#[tokio::main]
async fn main() {
    println!("🚀 Starting Solana HTTP Server...");
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3030".to_string())
        .parse::<u16>()
        .unwrap_or(3030);
    
  
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["content-type", "authorization"])
        .allow_methods(vec!["GET", "POST", "OPTIONS"]);
    
   
    let keypair_route = warp::path("keypair")
        .and(warp::post())
        .and_then(generate_keypair);
    
    let create_token_route = warp::path!("token" / "create")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(create_token);
    
    let mint_token_route = warp::path!("token" / "mint")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(mint_token);
    
    let sign_message_route = warp::path!("message" / "sign")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(sign_message);
    
    let verify_message_route = warp::path!("message" / "verify")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(verify_message);
    
    let send_sol_route = warp::path!("send" / "sol")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(send_sol);
    
    let send_token_route = warp::path!("send" / "token")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(send_token);
    
  
    let routes = keypair_route
        .or(create_token_route)
        .or(mint_token_route)
        .or(sign_message_route)
        .or(verify_message_route)
        .or(send_sol_route)
        .or(send_token_route)
        .with(cors);
    
 
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}
