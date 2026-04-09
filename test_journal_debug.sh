#!/bin/bash
# Test script to debug journal date formatting

# First, let's see what the API returns for date formatter
echo "Testing Logseq API date formatter..."
echo '{"method":"logseq.App.getDateFormatter","args":[]}' | \
  curl -s -X POST http://localhost:12315/api \
    -H "Authorization: Bearer ${LOGSEQ_API_TOKEN}" \
    -H "Content-Type: application/json" \
    -d @- | jq .

# Now let's try to get today's journal with common formats
echo -e "\nTrying to get journal page with ISO format (2026-04-09)..."
echo '{"method":"logseq.Editor.getPage","args":["2026-04-09"]}' | \
  curl -s -X POST http://localhost:12315/api \
    -H "Authorization: Bearer ${LOGSEQ_API_TOKEN}" \
    -H "Content-Type: application/json" \
    -d @- | jq .

echo -e "\nTrying to get journal page with MMM do, yyyy format (Apr 9th, 2026)..."
echo '{"method":"logseq.Editor.getPage","args":["Apr 9th, 2026"]}' | \
  curl -s -X POST http://localhost:12315/api \
    -H "Authorization: Bearer ${LOGSEQ_API_TOKEN}" \
    -H "Content-Type: application/json" \
    -d @- | jq .

echo -e "\nListing all pages to find journal pages..."
echo '{"method":"logseq.Editor.getAllPages","args":[]}' | \
  curl -s -X POST http://localhost:12315/api \
    -H "Authorization: Bearer ${LOGSEQ_API_TOKEN}" \
    -H "Content-Type: application/json" \
    -d @- | jq '[.[] | select(.name | test("2026|Apr"))] | .[0:5]'
