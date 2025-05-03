# ETH Balance Monitor

## Table of Contents
- [Overview](#overview)
- [Features](#features)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [Building for Linux](#building-for-linux)
- [Docker Deployment](#docker-deployment)
- [Systemd Service](#systemd-service)
- [Security Considerations](#security-considerations)
- [Troubleshooting](#troubleshooting)
- [License](#license)

## Overview

A Rust-based service that periodically monitors an Ethereum address and automatically transfers any ETH balance (after deducting gas fees) to a specified recipient address.


## Features

### Core Functionality
- Scheduled balance checks (configurable interval)
- Automatic transfer of remaining balance after gas deduction
- Dynamic gas price calculation
- Transaction confirmation tracking
- Send text notifications to individuals/groups/channels

### Technical Features
- Environment variable configuration
- Cross-platform support (Linux, macOS, Windows)
- Docker container support
- Systemd service integration
- Comprehensive logging

## Installation

### Prerequisites
- Rust 1.70+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Ethereum node access (Infura/Alchemy recommended)

### Create .env file:
```bash
# Required
BOT_TOKEN=your_bot_token_here
CHAT_ID=your_chat_id_here
```

### Installation Methods

**From Source:**
```bash
git clone https://github.com/your-repo/eth-balance-monitor.git
cd eth-balance-monitor
cargo build --release
