# Privacy-Preserving Sealed-Bid Auction System

This project implements a **privacy-preserving sealed-bid auction system** using zero-knowledge proofs and cryptographic commitments. It allows users to register, submit encrypted bids, and participate in auctions without revealing their losing bid amounts. The winner is determined in a transparent and verifiable way.

## 🌟 Key Features

* **Bid Privacy**: Bids are submitted as encrypted commitments and remain secret throughout the process.
* **Verifiable Outcomes**: The winner is determined through a verifiable reveal process on a public Bulletin Board.
* **Proof Certificates**: Losing bidders can generate a **Proof Certificate** to demonstrate their bid was lower than the winning price without revealing their actual bid.
* **WASM Integration**: The verification logic runs directly in the browser using high-performance WebAssembly.
* **Public Bulletin Board**: Stores all bids, proofs, and transcript data for offline verification.

## 🏗️ Project Architecture

The project is organized as a Rust workspace with a Next.js frontend:

* **`auction-server`**: Backend server (Axum) handling APIs, user authentication, and data persistence (SQLite/Postgres).
* **`auction-verifier`**: The core verification engine, compiled to WebAssembly (WASM) for frontend use.
* **`auction-crypto`**: Cryptographic primitives (Pedersen commitments, Schnorr proofs, Hash chains).
* **`auction-core`**: Shared data models and business logic (cross-platform compatible).
* **`frontend`**: Modern web interface built with Next.js 14 and Tailwind CSS.

## 🛠️ Prerequisites

* **Rust** (Stable) with the `wasm32-unknown-unknown` target.
* **Node.js** (v18+) & **npm**.
* **wasm-pack**: For compiling Rust to WebAssembly.
* **LLVM/Clang**: Required for compiling the `ring` library for WASM on macOS.

## 🚀 Getting Started

### 1. Backend Setup
From the project root:
```bash
# Start the backend server
cargo run -p auction-server
```

### 2. Frontend & WASM Setup
Open a new terminal in the frontend directory:
```bash
# 1. Compile the WebAssembly module
npm run wasm:build

# 2. Install Node.js dependencies
npm install

# 3. Start the development server
npm run dev
```

The application will be available at http://localhost:3000

### 🔧 Troubleshooting WASM Compilation
If you encounter errors during npm run wasm:build (especially with the ring crate on macOS), ensure your environment is configured to use a WASM-compatible compiler:
```bash
# For Apple Silicon Macs
export CC_wasm32_unknown_unknown="/opt/homebrew/opt/llvm/bin/clang"
export AR_wasm32_unknown_unknown="/opt/homebrew/opt/llvm/bin/llvm-ar"
```
Additionally, the project uses conditional compilation in auction-core to exclude database dependencies (sqlx) when targeting WebAssembly, ensuring a slim and compatible WASM bundle.

### 📂 Directory Structure
```text
.
├── crates/
│   ├── auction-server/   # Backend logic & API
│   ├── auction-verifier/ # Verification logic (WASM source)
│   ├── auction-crypto/   # Cryptographic implementation
│   └── auction-core/     # Shared types (Conditional WASM build)
├── frontend/
│   ├── src/app/          # Next.js App Router pages
│   ├── src/components/   # React components
│   └── src/wasm/         # Generated WASM artifacts
├── Cargo.toml            # Workspace configuration
└── README.md
```