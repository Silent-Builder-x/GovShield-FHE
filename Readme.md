# GovShield: FHE-Powered Private Voting Protocol (Fully Deployed)

## 🗳️ Project Overview
**GovShield** is a production-ready confidential governance primitive built on **Arcium** and **Solana**. 

In traditional DAO governance, radical transparency leads to the "Bandwagon Effect" and potential voter intimidation. GovShield solves this by performing vote tallies entirely on ciphertexts using **Fully Homomorphic Encryption (FHE)**. This ensures that individual choices remain hidden even from the nodes performing the computation.

## 🚀 Live Deployment Status (Devnet)
The protocol is currently active on the Arcium Multi-Party Execution (MXE) environment.

- **MXE Address:** `H5QyYPNbRmjcEzWnum55n7eC5xqtZohpFQD5wwpZC4xz`
- **Program ID:** `8ZVcFcKkzj3NAWLGhcVKaLGhQVyQiPCbHnCqRzQeySkD`
- **Authority:** `AjUstj3Qg296mz6DFcXAg186zRvNKuFfjB7JK2Z6vS7R`
- **Cluster:** `DzaQCyfybroycrNqE5Gk7LhSbWD2qfCics6qptBFbr95` (Offset 456)
- **Status:** `Active`

## 🧠 Core Innovation
- **Shielded Tallying:** Individual ballots are cast as `Enc<Shared, VotingBallot>`.
- **Homomorphic Summation:** The Arcis circuit performs `total_yes = sum(choice_i * weight_i)` without ever decrypting the `choice_i`.
- **MEV Resistance:** Validators cannot see the trending result until the final tally is published, preventing strategic front-running or late-stage vote manipulation.

## 🛠 Technical Structure
- `/encrypted-ixs`: The Arcis FHE circuit logic. Implements fixed-batch homomorphic addition.
- `/programs/private_voting`: The Solana Anchor program. Manages the voting state and handles secure callbacks from the Arcium network.

## 📦 Getting Started

### Prerequisites
- Solana Agave Toolsuite (v3.x)
- Arcium CLI
- Rust (Stable)

### Build & Deploy
```bash
# Clone the repository
git clone https://github.com/Silent-Builder-x/GovShield-FHE
cd GovShield-FHE

# Build the encrypted logic and ledger program
arcium build

# Deploy to Devnet
arcium deploy --cluster-offset 456 --recovery-set-size 4 -u d
```

### 📄 License
MIT License. Created for the Arcium RTG Developer Track.