// This is a simple HTTP server for Solana blockchain operations
// It's written in a beginner-friendly way with lots of comments!

// These are all the libraries we need
use warp::Filter;                   // For creating web server
use serde::{Deserialize, Serialize}; // For converting to/from JSON
use solana_sdk::{                   // For Solana blockchain operations
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
};
use std::str::FromStr;              // For converting strings
use base64::{Engine as _, engine::general_purpose::STANDARD as Base64}; // For base64 encoding
use bs58;                           // For base58 encoding (Solana uses this)

// ====== RESPONSE STRUCTURES ======
// These define what our server sends back to users

// Every API response has this format
#[derive(Serialize)]
struct ApiResponse {
    success: bool,     // true = good, false = error
    data: String,      // the actual response data as JSON string
    error: String,     // error message if something went wrong
}

// When we generate a new keypair
#[derive(Serialize)]
struct KeypairData {
    pubkey: String,    // public key (like an address)
    secret: String,    // secret key (keep this private!)
}

// When we create Solana instructions
#[derive(Serialize)]
struct InstructionData {
    program_id: String,       // which Solana program to call
    accounts: Vec<AccountInfo>, // list of accounts involved
    instruction_data: String, // the instruction data (base64 encoded)
}

// Info about each account in an instruction
#[derive(Serialize)]
struct AccountInfo {
    pubkey: String,      // account's public key
    is_signer: bool,     // does this account need to sign?
    is_writable: bool,   // will this account's data change?
}

// When we sign a message
#[derive(Serialize)]
struct SignData {
    signature: String,    // the signature we created
    public_key: String,   // public key that signed it
    message: String,      // original message
}

// When we verify a signature
#[derive(Serialize)]
struct VerifyData {
    valid: bool,         // true if signature is valid
    message: String,     // message that was checked
    pubkey: String,      // public key that should have signed it
}

// For SOL transfers
#[derive(Serialize)]
struct SolTransferData {
    program_id: String,      // system program ID
    accounts: Vec<String>,   // account addresses involved
    instruction_data: String, // transfer instruction data
}

// For token transfers
#[derive(Serialize)]
struct TokenAccount {
    pubkey: String,        // account address
    #[serde(rename = "isSigner")]
    is_signer: bool,       // needs to sign transaction?
}

#[derive(Serialize)]
struct TokenTransferData {
    program_id: String,           // SPL token program ID
    accounts: Vec<TokenAccount>,  // accounts involved
    instruction_data: String,     // transfer instruction data
}

// ====== REQUEST STRUCTURES ======
// These define what users send to our server

// To create a new token
#[derive(Deserialize)]
struct CreateTokenRequest {
    #[serde(rename = "mintAuthority")]
    mint_authority: String,  // who can mint new tokens
    mint: String,           // token mint address
    decimals: u8,          // decimal places (6 = like USDC)
}

// To mint tokens
#[derive(Deserialize)]
struct MintTokenRequest {
    mint: String,          // which token to mint
    destination: String,   // where to send new tokens
    authority: String,     // who's allowed to mint
    amount: u64,          // how many tokens to mint
}

// To sign a message
#[derive(Deserialize)]
struct SignMessageRequest {
    message: String,    // message to sign
    secret: String,     // secret key for signing
}

// To verify a signature
#[derive(Deserialize)]
struct VerifyMessageRequest {
    message: String,     // original message
    signature: String,   // signature to check
    pubkey: String,     // public key that should have signed
}

// To send SOL
#[derive(Deserialize)]
struct SendSolRequest {
    from: String,        // sender's public key
    to: String,          // recipient's public key
    lamports: u64,       // amount (1 SOL = 1,000,000,000 lamports)
}

// To send tokens
#[derive(Deserialize)]
struct SendTokenRequest {
    destination: String,  // who gets the tokens
    mint: String,        // which token to send
    owner: String,       // who owns the tokens now
    amount: u64,         // how many tokens to send
}

// ====== HELPER FUNCTIONS ======
// These make our code simpler and easier to read

// Create a successful response
fn success_response(data: String) -> Result<warp::reply::Json, warp::Rejection> {
    let response = ApiResponse {
        success: true,
        data: data,
        error: String::new(),
    };
    Ok(warp::reply::json(&response))
}

// Create an error response
fn error_response(error: &str) -> Result<warp::reply::Json, warp::Rejection> {
    let response = ApiResponse {
        success: false,
        data: String::new(),
        error: error.to_string(),
    };
    Ok(warp::reply::json(&response))
}

// Check if a string is a valid Solana public key
fn is_valid_pubkey(key_str: &str) -> Result<Pubkey, String> {
    match Pubkey::from_str(key_str) {
        Ok(pubkey) => Ok(pubkey),
        Err(_) => Err("Invalid public key format".to_string()),
    }
}

// ====== ENDPOINT HANDLERS ======
// These functions handle each API endpoint

// Generate a new keypair
// POST /keypair
async fn handle_generate_keypair() -> Result<warp::reply::Json, warp::Rejection> {
    println!("🔑 Generating new keypair...");
    
    // Step 1: Create a new random keypair
    let keypair = Keypair::new();
    
    // Step 2: Get the public key as a string
    let pubkey_string = bs58::encode(keypair.pubkey().to_bytes()).into_string();
    
    // Step 3: Get the secret key as a string
    let secret_string = bs58::encode(&keypair.to_bytes()).into_string();
    
    // Step 4: Create our response data
    let keypair_data = KeypairData {
        pubkey: pubkey_string,
        secret: secret_string,
    };
    
    // Step 5: Convert to JSON string and return
    match serde_json::to_string(&keypair_data) {
        Ok(json_string) => {
            println!("✅ Keypair generated successfully");
            success_response(json_string)
        },
        Err(e) => {
            println!("❌ Error converting to JSON: {}", e);
            error_response("Failed to serialize response")
        }
    }
}

// Create a new SPL token
// POST /token/create
async fn handle_create_token(request: CreateTokenRequest) -> Result<warp::reply::Json, warp::Rejection> {
    println!("🪙 Creating new token...");
    
    // Step 1: Validate the mint authority public key
    let mint_authority = match is_valid_pubkey(&request.mint_authority) {
        Ok(pubkey) => pubkey,
        Err(e) => {
            println!("❌ Invalid mint authority: {}", e);
            return error_response("Invalid mint authority public key");
        }
    };
    
    // Step 2: Validate the mint public key
    let mint = match is_valid_pubkey(&request.mint) {
        Ok(pubkey) => pubkey,
        Err(e) => {
            println!("❌ Invalid mint address: {}", e);
            return error_response("Invalid mint public key");
        }
    };
    
    // Step 3: Check decimals are reasonable
    if request.decimals > 9 {
        println!("❌ Too many decimals: {}", request.decimals);
        return error_response("Decimals cannot exceed 9");
    }
    
    // Step 4: Create the initialize mint instruction
    let instruction = match spl_token::instruction::initialize_mint(
        &spl_token::id(),     // SPL Token program ID
        &mint,                // mint account
        &mint_authority,      // mint authority
        Some(&mint_authority), // freeze authority (same as mint)
        request.decimals,     // decimal places
    ) {
        Ok(inst) => inst,
        Err(e) => {
            println!("❌ Failed to create instruction: {}", e);
            return error_response("Failed to create mint instruction");
        }
    };
    
    // Step 5: Convert accounts to our format
    let mut accounts = Vec::new();
    for account in instruction.accounts.iter() {
        accounts.push(AccountInfo {
            pubkey: account.pubkey.to_string(),
            is_signer: account.is_signer,
            is_writable: account.is_writable,
        });
    }
    
    // Step 6: Encode instruction data
    let instruction_data = Base64.encode(&instruction.data);
    
    // Step 7: Create response
    let response_data = InstructionData {
        program_id: instruction.program_id.to_string(),
        accounts: accounts,
        instruction_data: instruction_data,
    };
    
    // Step 8: Convert to JSON and return
    match serde_json::to_string(&response_data) {
        Ok(json_string) => {
            println!("✅ Token creation instruction generated");
            success_response(json_string)
        },
        Err(e) => {
            println!("❌ Error converting to JSON: {}", e);
            error_response("Failed to serialize response")
        }
    }
}

// Mint tokens to an account
// POST /token/mint
async fn handle_mint_token(request: MintTokenRequest) -> Result<warp::reply::Json, warp::Rejection> {
    println!("🏭 Minting tokens...");
    
    // Step 1: Validate mint public key
    let mint = match is_valid_pubkey(&request.mint) {
        Ok(pubkey) => pubkey,
        Err(e) => {
            println!("❌ Invalid mint: {}", e);
            return error_response("Invalid mint public key");
        }
    };
    
    // Step 2: Validate destination public key
    let destination = match is_valid_pubkey(&request.destination) {
        Ok(pubkey) => pubkey,
        Err(e) => {
            println!("❌ Invalid destination: {}", e);
            return error_response("Invalid destination public key");
        }
    };
    
    // Step 3: Validate authority public key
    let authority = match is_valid_pubkey(&request.authority) {
        Ok(pubkey) => pubkey,
        Err(e) => {
            println!("❌ Invalid authority: {}", e);
            return error_response("Invalid authority public key");
        }
    };
    
    // Step 4: Create mint instruction
    let instruction = match spl_token::instruction::mint_to(
        &spl_token::id(),  // SPL Token program
        &mint,             // mint account
        &destination,      // destination account
        &authority,        // mint authority
        &[],              // no additional signers
        request.amount,    // amount to mint
    ) {
        Ok(inst) => inst,
        Err(e) => {
            println!("❌ Failed to create mint instruction: {}", e);
            return error_response("Failed to create mint instruction");
        }
    };
    
    // Step 5: Convert accounts to our format
    let mut accounts = Vec::new();
    for account in instruction.accounts.iter() {
        accounts.push(AccountInfo {
            pubkey: account.pubkey.to_string(),
            is_signer: account.is_signer,
            is_writable: account.is_writable,
        });
    }
    
    // Step 6: Encode instruction data
    let instruction_data = Base64.encode(&instruction.data);
    
    // Step 7: Create response
    let response_data = InstructionData {
        program_id: instruction.program_id.to_string(),
        accounts: accounts,
        instruction_data: instruction_data,
    };
    
    // Step 8: Convert to JSON and return
    match serde_json::to_string(&response_data) {
        Ok(json_string) => {
            println!("✅ Mint instruction generated");
            success_response(json_string)
        },
        Err(e) => {
            println!("❌ Error converting to JSON: {}", e);
            error_response("Failed to serialize response")
        }
    }
}

// Sign a message with a private key
// POST /sign
async fn handle_sign_message(request: SignMessageRequest) -> Result<warp::reply::Json, warp::Rejection> {
    println!("✍️ Signing message...");
    
    // Step 1: Decode the secret key
    let secret_bytes = match bs58::decode(&request.secret).into_vec() {
        Ok(bytes) => bytes,
        Err(e) => {
            println!("❌ Invalid secret key format: {}", e);
            return error_response("Invalid secret key format");
        }
    };
    
    // Step 2: Create keypair from secret
    let keypair = match Keypair::from_bytes(&secret_bytes) {
        Ok(kp) => kp,
        Err(e) => {
            println!("❌ Failed to create keypair: {}", e);
            return error_response("Invalid secret key");
        }
    };
    
    // Step 3: Convert message to bytes
    let message_bytes = request.message.as_bytes();
    
    // Step 4: Sign the message
    let signature = keypair.sign_message(message_bytes);
    
    // Step 5: Create response data
    let sign_data = SignData {
        signature: bs58::encode(signature.as_ref()).into_string(),
        public_key: bs58::encode(keypair.pubkey().to_bytes()).into_string(),
        message: request.message,
    };
    
    // Step 6: Convert to JSON and return
    match serde_json::to_string(&sign_data) {
        Ok(json_string) => {
            println!("✅ Message signed successfully");
            success_response(json_string)
        },
        Err(e) => {
            println!("❌ Error converting to JSON: {}", e);
            error_response("Failed to serialize response")
        }
    }
}

// Verify a message signature
// POST /verify
async fn handle_verify_message(request: VerifyMessageRequest) -> Result<warp::reply::Json, warp::Rejection> {
    println!("🔍 Verifying signature...");
    
    // Step 1: Validate the public key
    let pubkey = match is_valid_pubkey(&request.pubkey) {
        Ok(pk) => pk,
        Err(e) => {
            println!("❌ Invalid public key: {}", e);
            return error_response("Invalid public key format");
        }
    };
    
    // Step 2: Decode the signature
    let signature_bytes = match bs58::decode(&request.signature).into_vec() {
        Ok(bytes) => bytes,
        Err(e) => {
            println!("❌ Invalid signature format: {}", e);
            return error_response("Invalid signature format");
        }
    };
    
    // Step 3: Create signature object
    let signature = match solana_sdk::signature::Signature::try_from(signature_bytes.as_slice()) {
        Ok(sig) => sig,
        Err(e) => {
            println!("❌ Invalid signature: {}", e);
            return error_response("Invalid signature");
        }
    };
    
    // Step 4: Verify the signature
    let message_bytes = request.message.as_bytes();
    let is_valid = signature.verify(pubkey.as_ref(), message_bytes);
    
    // Step 5: Create response
    let verify_data = VerifyData {
        valid: is_valid,
        message: request.message,
        pubkey: request.pubkey,
    };
    
    // Step 6: Convert to JSON and return
    match serde_json::to_string(&verify_data) {
        Ok(json_string) => {
            println!("✅ Signature verification complete: {}", is_valid);
            success_response(json_string)
        },
        Err(e) => {
            println!("❌ Error converting to JSON: {}", e);
            error_response("Failed to serialize response")
        }
    }
}

// Create instruction to send SOL
// POST /send-sol
async fn handle_send_sol(request: SendSolRequest) -> Result<warp::reply::Json, warp::Rejection> {
    println!("💰 Creating SOL transfer instruction...");
    
    // Step 1: Validate sender public key
    let from_pubkey = match is_valid_pubkey(&request.from) {
        Ok(pubkey) => pubkey,
        Err(e) => {
            println!("❌ Invalid sender address: {}", e);
            return error_response("Invalid sender public key");
        }
    };
    
    // Step 2: Validate recipient public key
    let to_pubkey = match is_valid_pubkey(&request.to) {
        Ok(pubkey) => pubkey,
        Err(e) => {
            println!("❌ Invalid recipient address: {}", e);
            return error_response("Invalid recipient public key");
        }
    };
    
    // Step 3: Create transfer instruction
    let instruction = system_instruction::transfer(
        &from_pubkey,    // from account
        &to_pubkey,      // to account
        request.lamports, // amount in lamports
    );
    
    // Step 4: Get account addresses
    let accounts = vec![
        from_pubkey.to_string(),  // sender (must sign and pay)
        to_pubkey.to_string(),    // recipient
    ];
    
    // Step 5: Encode instruction data
    let instruction_data = Base64.encode(&instruction.data);
    
    // Step 6: Create response
    let response_data = SolTransferData {
        program_id: instruction.program_id.to_string(),
        accounts: accounts,
        instruction_data: instruction_data,
    };
    
    // Step 7: Convert to JSON and return
    match serde_json::to_string(&response_data) {
        Ok(json_string) => {
            println!("✅ SOL transfer instruction created");
            success_response(json_string)
        },
        Err(e) => {
            println!("❌ Error converting to JSON: {}", e);
            error_response("Failed to serialize response")
        }
    }
}

// Create instruction to send SPL tokens
// POST /send-token
async fn handle_send_token(request: SendTokenRequest) -> Result<warp::reply::Json, warp::Rejection> {
    println!("🪙 Creating token transfer instruction...");
    
    // Step 1: Validate destination public key
    let destination = match is_valid_pubkey(&request.destination) {
        Ok(pubkey) => pubkey,
        Err(e) => {
            println!("❌ Invalid destination: {}", e);
            return error_response("Invalid destination public key");
        }
    };
    
    // Step 2: Validate mint public key
    let mint = match is_valid_pubkey(&request.mint) {
        Ok(pubkey) => pubkey,
        Err(e) => {
            println!("❌ Invalid mint: {}", e);
            return error_response("Invalid mint public key");
        }
    };
    
    // Step 3: Validate owner public key
    let owner = match is_valid_pubkey(&request.owner) {
        Ok(pubkey) => pubkey,
        Err(e) => {
            println!("❌ Invalid owner: {}", e);
            return error_response("Invalid owner public key");
        }
    };
    
    // Step 4: Calculate token account addresses
    let source_account = spl_associated_token_account::get_associated_token_address(&owner, &mint);
    let dest_account = spl_associated_token_account::get_associated_token_address(&destination, &mint);
    
    // Step 5: Create transfer instruction
    let instruction = match spl_token::instruction::transfer(
        &spl_token::id(),    // SPL Token program
        &source_account,     // source token account
        &dest_account,       // destination token account
        &owner,              // owner of source account
        &[],                // no additional signers
        request.amount,      // amount to transfer
    ) {
        Ok(inst) => inst,
        Err(e) => {
            println!("❌ Failed to create transfer instruction: {}", e);
            return error_response("Failed to create transfer instruction");
        }
    };
    
    // Step 6: Create account list
    let accounts = vec![
        TokenAccount {
            pubkey: source_account.to_string(),
            is_signer: false,
        },
        TokenAccount {
            pubkey: dest_account.to_string(),
            is_signer: false,
        },
        TokenAccount {
            pubkey: owner.to_string(),
            is_signer: true,  // owner must sign
        },
    ];
    
    // Step 7: Encode instruction data
    let instruction_data = Base64.encode(&instruction.data);
    
    // Step 8: Create response
    let response_data = TokenTransferData {
        program_id: instruction.program_id.to_string(),
        accounts: accounts,
        instruction_data: instruction_data,
    };
    
    // Step 9: Convert to JSON and return
    match serde_json::to_string(&response_data) {
        Ok(json_string) => {
            println!("✅ Token transfer instruction created");
            success_response(json_string)
        },
        Err(e) => {
            println!("❌ Error converting to JSON: {}", e);
            error_response("Failed to serialize response")
        }
    }
}

// ====== MAIN FUNCTION ======
// This is where our server starts

#[tokio::main]
async fn main() {
    println!("🚀 Starting Solana HTTP Server...");
    println!("📚 This server provides simple Solana blockchain operations");
    
    // Create CORS filter to allow requests from web browsers
    let cors = warp::cors()
        .allow_any_origin()      // Allow requests from any website
        .allow_headers(vec!["content-type"]) // Allow JSON content
        .allow_methods(vec!["GET", "POST"]); // Allow GET and POST requests
    
    // ====== ROUTE DEFINITIONS ======
    // Each route connects a URL to a handler function
    
    // GET / - Health check endpoint
    let health = warp::path::end()
        .and(warp::get())
        .map(|| {
            println!("🏥 Health check requested");
            warp::reply::html("
                <h1>🚀 Solana HTTP Server is Running!</h1>
                <p>This server provides simple Solana blockchain operations.</p>
                <h2>Available Endpoints:</h2>
                <ul>
                    <li><strong>POST /keypair</strong> - Generate a new keypair</li>
                    <li><strong>POST /token/create</strong> - Create a new SPL token</li>
                    <li><strong>POST /token/mint</strong> - Mint tokens to an account</li>
                    <li><strong>POST /sign</strong> - Sign a message</li>
                    <li><strong>POST /verify</strong> - Verify a signature</li>
                    <li><strong>POST /send-sol</strong> - Create SOL transfer instruction</li>
                    <li><strong>POST /send-token</strong> - Create token transfer instruction</li>
                </ul>
                <p>All endpoints return JSON responses with success/error status.</p>
            ")
        });
    
    // POST /keypair - Generate new keypair
    let keypair_route = warp::path("keypair")
        .and(warp::post())
        .and_then(handle_generate_keypair);
    
    // POST /token/create - Create new token
    let create_token_route = warp::path!("token" / "create")
        .and(warp::post())
        .and(warp::body::json()) // Expect JSON in request body
        .and_then(handle_create_token);
    
    // POST /token/mint - Mint tokens
    let mint_token_route = warp::path!("token" / "mint")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_mint_token);
    
    // POST /sign - Sign message
    let sign_route = warp::path("sign")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_sign_message);
    
    // POST /verify - Verify signature
    let verify_route = warp::path("verify")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_verify_message);
    
    // POST /send-sol - Create SOL transfer instruction
    let send_sol_route = warp::path("send-sol")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_send_sol);
    
    // POST /send-token - Create token transfer instruction
    let send_token_route = warp::path("send-token")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_send_token);
    
    // Combine all routes together
    let routes = health
        .or(keypair_route)
        .or(create_token_route)
        .or(mint_token_route)
        .or(sign_route)
        .or(verify_route)
        .or(send_sol_route)
        .or(send_token_route)
        .with(cors); // Add CORS to all routes
    
    // Start the server
    println!("🌐 Server starting on http://localhost:3031");
    println!("💡 Press Ctrl+C to stop the server");
    println!("📝 Available endpoints:");
    println!("   GET  /           - Health check");
    println!("   POST /keypair    - Generate keypair");
    println!("   POST /token/create - Create SPL token");
    println!("   POST /token/mint   - Mint tokens");
    println!("   POST /sign         - Sign message");
    println!("   POST /verify       - Verify signature");
    println!("   POST /send-sol     - SOL transfer");
    println!("   POST /send-token   - Token transfer");
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], 3031)) // Listen on all interfaces, port 3031
        .await;
}