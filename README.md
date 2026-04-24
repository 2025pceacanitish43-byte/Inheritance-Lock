# 🔐 InheritanceLock — Soroban Smart Contract

> A time-locked inheritance vault on Stellar. Lock funds for an heir and release them automatically once a target date (e.g. the heir's 18th birthday) is reached — no lawyers, no intermediaries, no trust required.

---

## 📖 Project Description

**InheritanceLock** is a Soroban smart contract deployed on the Stellar blockchain that allows a benefactor (parent, guardian, or anyone) to lock any SEP-41 token — including wrapped XLM — for a designated heir. The funds are held in the contract and cannot be touched by anyone until the configured `unlock_time` (a Unix timestamp) has passed.

Once the unlock time is reached, the heir calls `claim()` and receives the full balance instantly, on-chain, with zero intermediaries. The benefactor retains an emergency `revoke()` escape hatch before the claim is made.

---

## ⚙️ What It Does

| Step | Who | Action |
|------|-----|--------|
| 1 | Benefactor | Calls `deposit()` — sets the heir address, token, amount, and a future Unix timestamp. Tokens are transferred into the contract. |
| 2 | — | Time passes. The contract holds the funds securely. No one can move them. |
| 3 | Heir | Calls `claim()` after the unlock timestamp. Funds are transferred to the heir's wallet. |
| *(opt)* | Benefactor | Calls `revoke()` at any time *before* the heir claims, to retrieve funds (e.g. wrong address set). |

---

## ✨ Features

### 🔒 Time-Locked Vault
Funds are locked until a precise Unix timestamp. This maps directly to real-world milestones like a child's 18th or 21st birthday. The contract enforces the lock at the ledger level — not by trusting any human.

### 🪙 Token-Agnostic
Works with any [SEP-41](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md) compliant token on Stellar, including Stellar's wrapped native XLM (`CNAOEEIZBP4KO5NVBZUJHJ7XJTKGQDVKJQOQQDGJ3IE`) and any other custom asset.

### 👤 Heir-Only Withdrawal
Only the designated heir address (set at deposit time) can call `claim()`. Auth is enforced at the Soroban level — no other wallet can touch the funds.

### 🚨 Emergency Revocation
The benefactor can call `revoke()` at any time before the heir claims, recovering all deposited funds. Useful in case of an address mistake or change of circumstance.

### 📡 On-Chain Events
Every major action (`deposited`, `claimed`, `revoked`) emits a Soroban event, making it easy to track activity via explorers like [Stellar Expert](https://stellar.expert) or your own indexer.

### 🧮 Read-Only Helpers
Inspect the contract state at any time without a transaction:

| Function | Returns |
|---|---|
| `get_unlock_time()` | Timestamp when funds unlock |
| `get_amount()` | Total locked token amount |
| `is_claimed()` | Whether the heir has claimed |
| `time_remaining()` | Seconds until unlock (negative = unlocked) |

### 🛡️ Re-entrancy Safe
The `Claimed` flag is set to `true` **before** the token transfer in both `claim()` and `revoke()`, preventing any re-entrancy edge cases.

---

## 🗂️ Project Structure

```
inheritance-lock/
├── Cargo.toml          # Rust workspace & Soroban SDK dependency
└── src/
    └── lib.rs          # Contract source (all logic in one file)
```

---

## 🚀 Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) + `wasm32-unknown-unknown` target
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/install-and-setup)

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked stellar-cli --features opt
```

### Build

```bash
cd inheritance-lock
stellar contract build
# Output: target/wasm32-unknown-unknown/release/inheritance_lock.wasm
```

### Deploy to Testnet

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/inheritance_lock.wasm \
  --source <YOUR_SECRET_KEY> \
  --network testnet
```

### Deposit Funds

```bash
stellar contract invoke \
  --source <BENEFACTOR_SECRET_KEY> \
  --network testnet \
  -- deposit \
  --benefactor <BENEFACTOR_ADDRESS> \
  --heir <HEIR_ADDRESS> \
  --token <TOKEN_CONTRACT_ADDRESS> \
  --amount 1000000000 \
  --unlock_time 1893456000   # Unix timestamp — e.g. Jan 1 2030
```

### Claim (after unlock time)

```bash
stellar contract invoke \
  --source <HEIR_SECRET_KEY> \
  --network testnet \
  -- claim
```

---

## 📐 Architecture

```
Benefactor Wallet
      │
      │  deposit(heir, token, amount, unlock_time)
      ▼
┌─────────────────────────────────┐
│       InheritanceLock           │
│                                 │
│  storage:                       │
│   • Benefactor address          │
│   • Heir address                │
│   • Token contract address      │
│   • Locked amount               │
│   • Unlock timestamp            │
│   • Claimed flag                │
│                                 │
│  now < unlock_time → LOCKED 🔒  │
│  now ≥ unlock_time → OPEN  🔓   │
└─────────────────────────────────┘
      │
      │  claim() — called by Heir after unlock_time
      ▼
  Heir Wallet  ✅
```

---

## ⚠️ Disclaimer

This contract is provided for educational purposes. It has not been audited. Do not deploy on Mainnet with real funds without a professional security audit.

---

## 📄 License

MIT
wallet address: GDMWSLX2ZG47NS3XXOCXYVDDUPAJ5WU4UFRZMMJYB25I3R4OZP2OLEHK

contract address: CBGJZLEOUIUJNDQZOFQ3LODIVKXNEKEPEFI5XX3RZAJFPG75XAD3CP3X

https://stellar.expert/explorer/testnet/contract/CBGJZLEOUIUJNDQZOFQ3LODIVKXNEKEPEFI5XX3RZAJFPG75XAD3CP3X

<img width="1592" height="859" alt="image" src="https://github.com/user-attachments/assets/da43cfbe-dfef-4cbe-afb5-4a546dd31638" />
