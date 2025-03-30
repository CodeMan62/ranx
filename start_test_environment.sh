#!/bin/bash

echo "Starting test environment for Ranx reverse proxy"
echo "=============================================="

# Start the API server in the background
echo "Starting mock API server on port 3000..."
python3 ./mock_api_server.py &
API_PID=$!

# Start another API server (for load balancing) in the background
echo "Starting second mock API server on port 3001..."
python3 ./mock_api_server.py 3001 &
API2_PID=$!

# Start the web server in the background
echo "Starting mock web server on port 8000..."
python3 ./mock_web_server.py &
WEB_PID=$!

# Wait for servers to initialize
echo "Waiting for servers to initialize..."
sleep 2

# Start the proxy server
echo "Starting Ranx reverse proxy on port 8080..."
cargo run

# Cleanup function to kill background processes
cleanup() {
    echo "Shutting down test environment..."
    kill $API_PID $API2_PID $WEB_PID
    exit 0
}

# Set up trap to catch Ctrl+C and cleanup
trap cleanup INT

# Wait for the proxy to exit
wait $PROXY_PID

# Clean up
cleanup 
