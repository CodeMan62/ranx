#!/bin/bash

echo "Testing Ranx reverse proxy"
echo "=========================="
echo ""

# Test the API endpoint
echo "Testing API endpoint (should be forwarded to localhost:3000/users)"
curl -v http://localhost:8080/api/users

echo ""
echo ""

# Test the web endpoint
echo "Testing web endpoint (should be forwarded to localhost:8000/)"
curl -v http://localhost:8080/

echo ""
echo "=========================="
echo "Tests completed" 
