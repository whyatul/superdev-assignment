# Solana HTTP Server

A Rust-based HTTP server that provides Solana blockchain functionality through REST APIs. This server allows you to generate keypairs, handle SPL tokens, sign/verify messages, and construct valid on-chain instructions.

## Features

- 🔑 **Keypair Generation**: Generate new Solana keypairs
- 🪙 **Token Operations**: Create tokens and mint instructions
- ✍️ **Message Signing**: Sign and verify messages using Ed25519
- 💸 **Transfers**: Create SOL and SPL token transfer instructions
- 🛡️ **Security**: Input validation and proper error handling
- 🌐 **CORS**: Cross-origin requests enabled

## Quick Start

### Prerequisites

- Rust (latest stable version)
- Cargo

### Installation & Running

1. Clone and navigate to the project:
```bash
cd /home/am/superdev-assignment
```

2. Build and run the server:
```bash
cargo build --release
cargo run
```

The server will start on `http://127.0.0.1:3030`

## API Documentation

All endpoints return JSON responses with the following format:

### Success Response (200)
```json
{
  "success": true,
  "data": { /* endpoint-specific data */ }
}
```

### Error Response (400)
```json
{
  "success": false,
  "error": "Error description"
}
```

## Endpoints

### 1. Generate Keypair
**POST** `/keypair`

Generates a new Solana keypair.

**Response:**
```json
{
  "success": true,
  "data": {
    "pubkey": "base58-encoded-public-key",
    "secret": "base58-encoded-secret-key"
  }
}
```

**Example:**
```bash
curl -X POST http://127.0.0.1:3030/keypair
```

### 2. Create Token
**POST** `/token/create`

Creates a new SPL token initialize mint instruction.

**Request:**
```json
{
  "mintAuthority": "base58-encoded-public-key",
  "mint": "base58-encoded-public-key",
  "decimals": 6
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "program_id": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    "accounts": [
      {
        "pubkey": "mint-address",
        "is_signer": false,
        "is_writable": true
      }
    ],
    "instruction_data": "base64-encoded-data"
  }
}
```

### 3. Mint Token
**POST** `/token/mint`

Creates a mint-to instruction for SPL tokens.

**Request:**
```json
{
  "mint": "mint-address",
  "destination": "destination-token-account",
  "authority": "authority-address",
  "amount": 1000000
}
```

### 4. Sign Message
**POST** `/message/sign`

Signs a message using a private key.

**Request:**
```json
{
  "message": "Hello, Solana!",
  "secret": "base58-encoded-secret-key"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "signature": "base64-encoded-signature",
    "public_key": "base58-encoded-public-key",
    "message": "Hello, Solana!"
  }
}
```

### 5. Verify Message
**POST** `/message/verify`

Verifies a signed message.

**Request:**
```json
{
  "message": "Hello, Solana!",
  "signature": "base64-encoded-signature",
  "pubkey": "base58-encoded-public-key"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "valid": true,
    "message": "Hello, Solana!",
    "pubkey": "base58-encoded-public-key"
  }
}
```

### 6. Send SOL
**POST** `/send/sol`

Creates a SOL transfer instruction.

**Request:**
```json
{
  "from": "sender-address",
  "to": "recipient-address",
  "lamports": 100000
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "program_id": "11111111111111111111111111111111",
    "accounts": [
      "sender-address",
      "recipient-address"
    ],
    "instruction_data": "base64-encoded-instruction-data"
  }
}
```

### 7. Send Token
**POST** `/send/token`

Creates an SPL token transfer instruction.

**Request:**
```json
{
  "destination": "destination-wallet-address",
  "mint": "token-mint-address",
  "owner": "owner-wallet-address",
  "amount": 100000
}
```

## Example Usage

Here are some complete examples using curl:

### Generate a new keypair:
```bash
curl -X POST http://127.0.0.1:3030/keypair
```

### Sign a message:
```bash
curl -X POST http://127.0.0.1:3030/message/sign \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Hello Solana!",
    "secret": "your-base58-secret-key"
  }'
```

### Create SOL transfer:
```bash
curl -X POST http://127.0.0.1:3030/send/sol \
  -H "Content-Type: application/json" \
  -d '{
    "from": "sender-public-key",
    "to": "recipient-public-key",
    "lamports": 1000000
  }'
```

## Input Validation

The server validates all inputs:

- **Public Keys**: Must be valid base58-encoded Solana public keys (32 bytes)
- **Secret Keys**: Must be valid base58-encoded Solana keypairs (64 bytes)
- **Amounts**: Must be greater than 0
- **Decimals**: Must not exceed 9 for token creation
- **Addresses**: Sender and recipient cannot be the same for transfers

## Security Features

- ✅ No private keys stored on server
- ✅ All cryptographic operations use standard libraries
- ✅ Comprehensive input validation
- ✅ Proper error handling without information leakage
- ✅ CORS enabled for web applications

## Error Handling

Common error scenarios:
- Invalid public key format
- Invalid secret key format
- Missing required fields
- Invalid amounts (zero or negative)
- Same sender and recipient addresses

## Dependencies

- **warp**: HTTP server framework
- **solana-sdk**: Solana blockchain SDK
- **spl-token**: SPL token program
- **ed25519-dalek**: Ed25519 cryptography
- **serde**: JSON serialization
- **base64/bs58**: Encoding utilities

## Development

To run in development mode:
```bash
cargo run
```

To run tests:
```bash
cargo test
```

To build for production:
```bash
cargo build --release
```

## License

This project is open source and available under the MIT License.
