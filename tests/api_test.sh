#!/bin/bash

BASE_URL="http://localhost:8080/api"

echo "Testing Memorable Password Generation..."
curl -s "$BASE_URL/memorable/generate" | jq .
echo ""

echo "Testing Wordlist Generation..."
cat <<EOF > profile.json
{
    "first_names": ["John"],
    "last_names": ["Doe"],
    "partners": [],
    "kids": [],
    "pets": [],
    "company": [],
    "school": [],
    "city": [],
    "sports": [],
    "music": [],
    "usernames": [],
    "dates": ["1990"],
    "keywords": [],
    "numbers": ["123"]
}
EOF

curl -s -X POST -H "Content-Type: application/json" -d @profile.json "$BASE_URL/personal/generate" | jq '.[:5]' # Show first 5
echo ""

echo "Testing Password Check (Found)..."
cat <<EOF > check_found.json
{
    "profile": {
        "first_names": ["John"],
        "last_names": ["Doe"],
        "dates": ["1990"],
        "numbers": ["123"],
        "partners": [], "kids": [], "pets": [], "company": [], 
        "school": [], "city": [], "sports": [], "music": [], 
        "usernames": [], "keywords": []
    },
    "password": "John_Doe123"
}
EOF
curl -s -X POST -H "Content-Type: application/json" -d @check_found.json "$BASE_URL/check-password" | jq .
echo ""

echo "Testing Password Check (Not Found)..."
cat <<EOF > check_not_found.json
{
    "profile": {  "first_names": ["John"],
        "last_names": ["Doe"],
        "dates": ["1990"],
        "numbers": ["123"],
        "partners": [], "kids": [], "pets": [], "company": [], 
        "school": [], "city": [], "sports": [], "music": [], 
        "usernames": [], "keywords": []
    },
    "password": "ImpossiblePasswordXYZ"
}
EOF
curl -s -X POST -H "Content-Type: application/json" -d @check_not_found.json "$BASE_URL/check-password" | jq .
echo ""

rm profile.json check_found.json check_not_found.json
