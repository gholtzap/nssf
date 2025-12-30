#!/bin/bash

set -e

echo "Setting up NSSF repository..."

if [ ! -f .env ]; then
    echo "Creating .env file..."

    read -p "Enter MongoDB URI: " MONGODB_URI
    read -p "Enter MongoDB Database Name [nssf]: " MONGODB_DB_NAME
    MONGODB_DB_NAME=${MONGODB_DB_NAME:-nssf}
    read -p "Enter MongoDB Collection Name [slices]: " MONGODB_COLLECTION_NAME
    MONGODB_COLLECTION_NAME=${MONGODB_COLLECTION_NAME:-slices}
    read -p "Enter PORT [8080]: " PORT
    PORT=${PORT:-8080}

    cat > .env << EOF
MONGODB_URI=${MONGODB_URI}
MONGODB_DB_NAME=${MONGODB_DB_NAME}
MONGODB_COLLECTION_NAME=${MONGODB_COLLECTION_NAME}
PORT=${PORT}
EOF

    echo ".env file created successfully"
else
    echo ".env file already exists, skipping..."
fi

echo "Installing npm dependencies..."
npm install

echo "Building TypeScript project..."
npm run build

echo ""
echo "Setup complete! You can now:"
echo "  - Run development server: npm run dev"
echo "  - Run tests: npm test"
echo "  - Start production server: npm start"
