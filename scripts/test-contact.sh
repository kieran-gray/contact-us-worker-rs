#!/bin/bash

# Test script for the contact-us endpoint
# Usage: ./test-contact.sh [URL] [TOKEN]
# Default URL: http://localhost:8787
# Default TOKEN: test-token-12345

URL="${1:-http://localhost:8787}"
TOKEN="${2:-test-token-12345}"

echo "Testing Contact Us endpoint at: $URL/api/v1/contact-us/"

PAYLOAD=$(cat <<EOF
{
  "token": "$TOKEN",
  "category": "IDEA",
  "email": "test@example.com",
  "name": "John Doe",
  "message": "This is a test message from the test script. Testing the contact-us API endpoint.",
  "data": {
    "source": "test-script",
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  }
}
EOF
)

echo "Sending request..."
echo ""

RESPONSE=$(curl -s -w "\n%{http_code}" -X POST \
  "$URL/api/v1/contact-us/" \
  -H "Content-Type: application/json" \
  -H "Origin: http://localhost:5173" \
  -d "$PAYLOAD")

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | sed '$d')

echo "Status Code: $HTTP_CODE"
echo ""
echo "Response:"
echo "$BODY" | jq '.' 2>/dev/null || echo "$BODY"
echo ""

if [ "$HTTP_CODE" -eq 200 ]; then
  echo "Success"
  exit 0
else
  echo "Failed"
  exit 1
fi
