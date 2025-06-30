#!/bin/bash

# Example script to test the Solana HTTP Server
# Make sure the server is running on http://127.0.0.1:3030

echo "🧪 Testing Solana HTTP Server"
echo "=============================="

# Test 1: Generate keypair
echo -e "\n1️⃣ Generating a new keypair..."
KEYPAIR_RESPONSE=$(curl -s -X POST http://127.0.0.1:3030/keypair)
echo "Response: $KEYPAIR_RESPONSE"

# Extract pubkey and secret from response for further tests
PUBKEY=$(echo $KEYPAIR_RESPONSE | jq -r '.data.pubkey')
SECRET=$(echo $KEYPAIR_RESPONSE | jq -r '.data.secret')

echo "Generated Public Key: $PUBKEY"

# Test 2: Sign a message
echo -e "\n2️⃣ Signing a message..."
SIGN_RESPONSE=$(curl -s -X POST http://127.0.0.1:3030/message/sign \
  -H "Content-Type: application/json" \
  -d "{
    \"message\": \"Hello, Solana!\",
    \"secret\": \"$SECRET\"
  }")
echo "Response: $SIGN_RESPONSE"

# Extract signature for verification
SIGNATURE=$(echo $SIGN_RESPONSE | jq -r '.data.signature')

# Test 3: Verify the message
echo -e "\n3️⃣ Verifying the signed message..."
VERIFY_RESPONSE=$(curl -s -X POST http://127.0.0.1:3030/message/verify \
  -H "Content-Type: application/json" \
  -d "{
    \"message\": \"Hello, Solana!\",
    \"signature\": \"$SIGNATURE\",
    \"pubkey\": \"$PUBKEY\"
  }")
echo "Response: $VERIFY_RESPONSE"

# Test 4: Create SOL transfer instruction
echo -e "\n4️⃣ Creating SOL transfer instruction..."
# Generate another keypair for recipient
RECIPIENT_RESPONSE=$(curl -s -X POST http://127.0.0.1:3030/keypair)
RECIPIENT_PUBKEY=$(echo $RECIPIENT_RESPONSE | jq -r '.data.pubkey')

SOL_TRANSFER_RESPONSE=$(curl -s -X POST http://127.0.0.1:3030/send/sol \
  -H "Content-Type: application/json" \
  -d "{
    \"from\": \"$PUBKEY\",
    \"to\": \"$RECIPIENT_PUBKEY\",
    \"lamports\": 1000000
  }")
echo "Response: $SOL_TRANSFER_RESPONSE"

# Test 5: Test error handling
echo -e "\n5️⃣ Testing error handling (invalid pubkey)..."
ERROR_RESPONSE=$(curl -s -X POST http://127.0.0.1:3030/send/sol \
  -H "Content-Type: application/json" \
  -d "{
    \"from\": \"invalid-pubkey\",
    \"to\": \"$RECIPIENT_PUBKEY\",
    \"lamports\": 1000000
  }")
echo "Response: $ERROR_RESPONSE"

echo -e "\n✅ All tests completed!"
echo "💡 Tip: Install 'jq' for better JSON formatting: sudo apt install jq"
