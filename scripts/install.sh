#!/bin/bash
set -e

COMPOSE_URL="https://raw.githubusercontent.com/Necko1/necko-xray/refs/heads/master/docker-compose.yml"
WRAPPER_URL="https://raw.githubusercontent.com/Necko1/necko-xray/refs/heads/master/necko-xray"
EXAMPLE_CONFIG_URL="https://raw.githubusercontent.com/Necko1/necko-xray/refs/heads/master/profiles/example.json"

INSTALL_DIR="/opt/necko-xray"
BIN_PATH="/usr/local/bin/necko-xray"
CONTAINER="necko-xray"

if [ "$EUID" -ne 0 ]; then
  echo "Error: Please run this script as root (sudo)."
  exit 1
fi

if ! command -v docker &> /dev/null; then
  echo "Error: Docker is not installed."
  printf "Do you want to install it? [y/N] "
  read -r response < /dev/tty

  case $response in
    [Yy]* )
      echo "Installing Docker..."
      curl -fsSL https://get.docker.com | sh

      if ! command -v docker &> /dev/null; then
         echo "Docker installation failed. Please install manually."
         exit 1
      fi
      ;;
    * )
      echo "Please install Docker first."
      exit 1
      ;;
  esac

fi

echo "Starting necko-xray Installation"

mkdir -p "$INSTALL_DIR"
cd "$INSTALL_DIR"

if docker inspect -f '{{.State.Running}}' "$CONTAINER" >/dev/null 2>&1; then
  echo "Stopping existing container..."
  docker compose down
fi

echo "Downloading configuration..."
curl -sSL -o docker-compose.yml "$COMPOSE_URL"

echo "Downloading profiles/example.json..."
mkdir -p profiles
curl -sSL -o profiles/example.json "$EXAMPLE_CONFIG_URL"

if [ ! -f ".env" ]; then
  echo "Generating .env file from example..."

  EXAMPLE_URL="https://raw.githubusercontent.com/Necko1/necko-xray/refs/heads/master/.env.example"
  curl -sSL -o .env "$EXAMPLE_URL"

  PG_PASS=$(openssl rand -hex 16)
  VALKEY_PASS=$(openssl rand -hex 16)

  if [[ "$OSTYPE" == "darwin"* ]]; then
    SED_OPTS=(-i "")
  else
    SED_OPTS=(-i)
  fi

  sed "${SED_OPTS[@]}" \
      -e "s/^POSTGRES_PASSWORD=.*/POSTGRES_PASSWORD=$PG_PASS/" \
      -e "s/^VALKEY_PASSWORD=.*/VALKEY_PASSWORD=$VALKEY_PASS/" \
      -e "s|postgresql://postgres:change_this@|postgresql://postgres:$PG_PASS@|" \
      -e "s|redis://:change_this@|redis://:$VALKEY_PASS@|" \
      .env

  echo ".env file created successfully."
else
  echo ".env file already exists. Skipping generation."
fi

echo "Pulling images and starting services..."
docker compose pull
docker compose up -d

echo "Installing CLI tool..."
curl -sSL -o "$BIN_PATH" "$WRAPPER_URL"
chmod +x "$BIN_PATH"

echo "Installation Complete."
echo "Run 'sudo necko-xray help' to get started."