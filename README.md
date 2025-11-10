# Polymarket Clone on Solana

A decentralized prediction market platform built on Solana blockchain, inspired by Polymarket.

## Overview

This project implements a prediction market protocol on Solana where users can:
- Create markets for various outcomes
- Place limit orders
- Trade positions
- Settle markets based on real-world outcomes

## Technology Stack

- Solana Blockchain
- Anchor Framework
- Rust (for on-chain programs)
- TypeScript (for client integration)

## Project Structure

```
├── app/              # Frontend application
├── programs/         # Solana smart contracts
│   └── polymarket-clone/
│       └── src/     # Contract source code
├── migrations/       # Deployment scripts
└── tests/           # Integration tests
```

## Prerequisites

- Node.js 14+
- Rust and Cargo
- Solana CLI tools
- Anchor Framework

## Installation

1. Clone the repository:
```bash
git clone https://github.com/shri-prakhar/Prediction_market_on_solana.git
cd Prediction_market_on_solana
```

2. Install dependencies:
```bash
npm install
```

3. Build the program:
```bash
anchor build
```

## Testing

Run the test suite:
```bash
anchor test
```

## Deployment

1. Update your Solana cluster configuration in `Anchor.toml`
2. Deploy the program:
```bash
anchor deploy
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Inspired by Polymarket
- Built with Solana and Anchor Framework