#!/bin/bash

# Start backend servers
echo "Starting backend servers..."
node backend_server.js 3000 &
node backend_server.js 3001 &
sleep 2

# Start the proxy
echo "Starting reverse proxy..."
cargo run &
sleep 2

# Function to make requests
make_request() {
    echo "Making request to $1..."
    curl -s "$1" | jq '.' 2>/dev/null || echo "Raw response:" && curl -s "$1"
    echo -e "\n"
}

# Test load balancing
echo "Testing load balancing..."
for i in {1..4}; do
    make_request "http://localhost:8080/api/users"
    sleep 1
done

# Test rate limiting
echo "Testing rate limiting..."
for i in {1..10}; do
    make_request "http://localhost:8080/api/users"
done

# Test circuit breaker
echo "Testing circuit breaker..."
# Kill one backend to trigger circuit breaker
kill $(lsof -t -i:3000)
sleep 1
for i in {1..3}; do
    make_request "http://localhost:8080/api/users"
done

# Show metrics
echo "Displaying metrics..."
make_request "http://localhost:8080/api/health"

# Cleanup
echo "Cleaning up..."
pkill -f "node backend_server.js"
pkill -f "target/debug/ranx" 
