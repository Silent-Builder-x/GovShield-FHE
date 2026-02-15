# GovShield: MPC-Powered Private Voting Protocol (Fully Deployed)

## 🗳️ Project Overview

**GovShield** is a production-ready confidential governance primitive built on **Arcium** and **Solana**.

In traditional DAO governance, radical transparency leads to the "Bandwagon Effect" and potential voter intimidation. GovShield solves this by performing vote tallies entirely on secret shares using **Secure Multi-Party Computation (MPC)**. This ensures that individual choices remain hidden even from the nodes performing the computation.

## 🚀 Live Deployment Status (Devnet v0.8.3)

The protocol is currently active on the Arcium Multi-Party Execution (MXE) environment.

### 🖥️ Interactive Demo

[Launch ArcGov Terminal](https://silent-builder-x.github.io/GovShield-FHE/)

## 🧠 Core Innovation

- **Shielded Tallying:** Individual ballots are cast as `Enc<Shared, VotingBallot>`.
- **Oblivious Summation:** The Arcis circuit performs `total_yes = sum(choice_i * weight_i)` entirely in the encrypted domain without reconstructing `choice_i`.
- **MEV Resistance:** Validators cannot see the trending result until the final tally is published, preventing strategic front-running or late-stage vote manipulation.

## 🛠 Technical Structure

- `/encrypted-ixs`: The Arcis MPC circuit logic. Implements fixed-batch oblivious addition.
- `/programs/private_voting`: The Solana Anchor program. Manages the voting state and handles secure callbacks from the Arcium network.

## 📦 Getting Started

### Prerequisites

- Solana Agave Toolsuite (v1.18+)
- Arcium CLI
- Rust (Stable)

### Build & Deploy

```
# Clone the repository
git clone [https://github.com/Silent-Builder-x/GovShield-MPC](https://github.com/Silent-Builder-x/GovShield-MPC)
cd GovShield-MPC

# Build the encrypted logic and ledger program
arcium build

# Deploy to Devnet
arcium deploy --cluster-offset 456 --recovery-set-size 4 -u d

```

### 📄 License

MIT License. Created for the Arcium RTG Developer Track.