#!/bin/bash

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if Elixir 1.17 is installed
if command_exists elixir; then
    ELIXIR_VERSION=$(elixir -v | grep -oP 'Elixir 1\.17\.')
    echo "$ELIXIR_VERSION"
    if [ "$ELIXIR_VERSION" = "Elixir 1.17." ]; then
        echo "Elixir 1.17 is installed."
    else
        echo "Elixir 1.17 is not installed. Installing Elixir 1.17..."
        sudo apt remove elixir -y
        sudo apt autoremove -y
        sudo add-apt-repository ppa:rabbitmq/rabbitmq-erlang -y
        sudo apt update
        sudo apt install -y elixir
    fi
else
    echo "Elixir is not installed. Installing Elixir 1.17..."
    sudo add-apt-repository ppa:rabbitmq/rabbitmq-erlang -y
    sudo apt update
    sudo apt install -y elixir
fi

# Verify the installation
ELIXIR_VERSION=$(elixir -v | grep -oP 'Elixir 1\.17\.')
if [ "$ELIXIR_VERSION" != "Elixir 1.17." ]; then
    echo "Could not install Elixir 1.17. Please install it manually."
    exit 1
fi
