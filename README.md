# Organ Matching Program on Solana Blockchain

Welcome to the **Organ Matching Program**, a decentralized application built on the Solana blockchain using the Anchor framework. This program aims to streamline and secure the process of matching organ donors with recipients by leveraging blockchain technology's transparency, immutability, and security features.

---

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Architecture](#architecture)
- [Data Structures](#data-structures)
- [Installation](#installation)
- [Usage](#usage)
  - [Initialize Program State](#initialize-program-state)
  - [Manage Medical Authority](#manage-medical-authority)
  - [Upsert Recipient](#upsert-recipient)
  - [Add Donor](#add-donor)
  - [Find Best Match](#find-best-match)
  - [Confirm Match](#confirm-match)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

The Organ Matching Program is designed to enhance the efficiency and fairness of organ transplantation by automating the matching process based on various medical criteria. It ensures data integrity and provides a transparent audit trail for all transactions, making the process more trustworthy for all stakeholders.

---

## Features

- **Decentralized Matching Algorithm**: Automates the process of finding the best match between donors and recipients based on medical urgency, compatibility, and other factors.
- **Role-Based Access Control**: Introduces roles like Admin, Medical Authority, Donor, and Recipient with specific permissions.
- **Data Validation**: Implements rigorous validation for recipient and donor data to ensure accuracy and compliance.
- **Event Logging**: Emits events for critical actions like recipient updates, match findings, and match confirmations.
- **Security Measures**: Incorporates checks to prevent unauthorized actions and ensures data privacy.
- **Scalability**: Optimized to handle a large number of recipients and donors efficiently.

---

## Architecture

The program is structured into several instructions and account types to manage the organ matching process:

- **Program State**: Stores global state variables like the admin's public key and recipient count.
- **Medical Authority**: Accounts representing authorized medical professionals who can manage recipients and donors.
- **Recipient Account**: Stores recipient data and status.
- **Donor Account**: Stores donor data and status.
- **Match Account**: Stores match information between a donor and a recipient.

---

## Data Structures

### Accounts

- **ProgramState**

  ```rust
  pub struct ProgramState {
      pub admin: Pubkey,
      pub recipient_count: u32,
      pub paused: bool,
  }
  ```

- **MedicalAuthority**

  ```rust
  pub struct MedicalAuthority {
      pub authority: Pubkey,
      pub is_active: bool,
      pub verified_matches: u32,
  }
  ```

- **RecipientAccount**

  ```rust
  pub struct RecipientAccount {
      pub authority: Pubkey,
      pub data: RecipientData,
      pub status: RecipientStatus,
  }
  ```

- **DonorAccount**

  ```rust
  pub struct DonorAccount {
      pub authority: Pubkey,
      pub data: DonorData,
      pub created_at: i64,
      pub status: DonorStatus,
  }
  ```

- **MatchAccount**

  ```rust
  pub struct MatchAccount {
      pub recipient: Pubkey,
      pub donor: Pubkey,
      pub score: u64,
      pub timestamp: i64,
      pub status: MatchStatus,
  }
  ```

### Data Types

- **RecipientData**

  ```rust
  pub struct RecipientData {
      pub medical_urgency: u8,
      pub geographical_distance: u32,
      pub hla_markers: [u8; 5],
      pub blood_type: BloodType,
      pub organ_type: OrganType,
      pub age: u8,
      pub created_at: i64,
      pub last_updated: i64,
      pub medical_notes: String,
  }
  ```

- **DonorData**

  ```rust
  pub struct DonorData {
      pub hla_markers: [u8; 5],
      pub blood_type: BloodType,
      pub organ_type: OrganType,
      pub medical_notes: String,
  }
  ```

### Enums

- **BloodType**

  ```rust
  pub enum BloodType {
      APositive,
      ANegative,
      BPositive,
      BNegative,
      ABPositive,
      ABNegative,
      OPositive,
      ONegative,
  }
  ```

- **OrganType**

  ```rust
  pub enum OrganType {
      Kidney,
      Liver,
      Heart,
      Lung,
      Pancreas,
  }
  ```

- **RecipientStatus**

  ```rust
  pub enum RecipientStatus {
      Active,
      Matched,
      Removed,
  }
  ```

- **DonorStatus**

  ```rust
  pub enum DonorStatus {
      Active,
      Matched,
      Withdrawn,
  }
  ```

- **MatchStatus**

  ```rust
  pub enum MatchStatus {
      Pending,
      Confirmed,
      Rejected,
  }
  ```

---

## Installation

### Prerequisites

- **Rust and Cargo**: Install from [rustup.rs](https://rustup.rs/).
- **Solana CLI**: Follow the instructions at [docs.solana.com/cli/install-solana-cli-tools](https://docs.solana.com/cli/install-solana-cli-tools).
- **Anchor CLI**: Install using Cargo:

  ```bash
  cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
  avm install latest
  avm use latest
  ```

### Clone the Repository

```bash
git clone https://github.com/yourusername/organ-matching-program.git
cd organ-matching-program
```

### Build the Program

```bash
anchor build
```

---

## Usage

### Initialize Program State

Initialize the program state with the admin's public key.

#### Command

```bash
anchor test -- --nocapture
```

#### Code Example

```rust
let program_state = Keypair::new();
let admin = payer.pubkey();

let tx = anchor_lang::InstructionData::data(&Initialize { admin });

anchor_lang::solana_program::system_instruction::create_account(
    &payer.pubkey(),
    &program_state.pubkey(),
    rent_lamports,
    ProgramState::LEN as u64,
    &program_id,
);
```

### Manage Medical Authority

Add or update a medical authority.

#### Command

```bash
anchor test -- --nocapture
```

#### Code Example

```rust
let authority = medical_authority_keypair.pubkey();
let is_active = true;

let tx = anchor_lang::InstructionData::data(&ManageMedicalAuthority {
    authority,
    is_active,
});
```

### Upsert Recipient

Add or update a recipient's information.

#### Command

```bash
anchor test -- --nocapture
```

#### Code Example

```rust
let recipient_data = RecipientData {
    medical_urgency: 85,
    geographical_distance: 200,
    hla_markers: [1, 2, 3, 4, 5],
    blood_type: BloodType::ONegative,
    organ_type: OrganType::Kidney,
    age: 45,
    created_at: 0,
    last_updated: 0,
    medical_notes: "Recipient medical notes.".to_string(),
};

let tx = anchor_lang::InstructionData::data(&UpsertRecipient { recipient_data });
```

### Add Donor

Add a new donor's information.

#### Command

```bash
anchor test -- --nocapture
```

#### Code Example

```rust
let donor_data = DonorData {
    hla_markers: [1, 2, 3, 4, 5],
    blood_type: BloodType::ONegative,
    organ_type: OrganType::Kidney,
    medical_notes: "Donor medical notes.".to_string(),
};

let tx = anchor_lang::InstructionData::data(&AddDonor { donor_data });
```

### Find Best Match

Find the best recipient match for a donor.

#### Command

```bash
anchor test -- --nocapture
```

#### Code Example

```rust
let tx = anchor_lang::InstructionData::data(&FindBestMatch {});
```

### Confirm Match

Confirm a match between a donor and a recipient.

#### Command

```bash
anchor test -- --nocapture
```

#### Code Example

```rust
let tx = anchor_lang::InstructionData::data(&ConfirmMatch {});
```

---

## Testing

The program includes unit tests in the `tests` module. You can run these tests using the following command:

```bash
anchor test
```

---

## Contributing

Contributions are welcome! Please follow these steps:

1. **Fork the Repository**: Click the "Fork" button at the top right of this page.
2. **Clone Your Fork**: Replace `yourusername` with your GitHub username.

   ```bash
   git clone https://github.com/yourusername/organ-matching-program.git
   ```

3. **Create a Branch**:

   ```bash
   git checkout -b feature/your-feature-name
   ```

4. **Make Your Changes**: Implement your feature or fix.
5. **Commit Your Changes**:

   ```bash
   git commit -m "Description of your changes"
   ```

6. **Push to Your Fork**:

   ```bash
   git push origin feature/your-feature-name
   ```

7. **Create a Pull Request**: Go to your fork on GitHub and click "New pull request".

---

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

## Acknowledgements

- **Anchor Framework**: For providing a robust framework for Solana program development.
- **Solana Community**: For their extensive documentation and support.

---

## Contact

For any questions or support, please open an issue or contact the repository maintainer at [your-email@example.com](mailto:your-email@example.com).

---

## Disclaimer

This program is provided "as is" without any warranties. It is intended for educational and demonstration purposes. Before deploying to a production environment, ensure thorough testing and security auditing. Always consult with legal and technical professionals when handling sensitive data.

---

## Notes

- **Data Privacy**: Ensure compliance with all relevant data protection regulations when handling sensitive medical data.
- **Security**: Be cautious of re-initialization attacks and other security vulnerabilities. Review the code and apply best practices.
- **Scalability**: Consider transaction size limits when dealing with large numbers of accounts. Implement batching or off-chain indexing as needed.

---

# Quick Start Guide

## Prerequisites

Ensure you have the following installed:

- **Rust and Cargo**
- **Solana CLI**
- **Anchor CLI**

## Setting Up

1. **Clone the Repository**

   ```bash
   git clone https://github.com/yourusername/organ-matching-program.git
   cd organ-matching-program
   ```

2. **Install Dependencies**

   ```bash
   anchor build
   ```

3. **Start a Local Solana Cluster**

   ```bash
   solana-test-validator
   ```

4. **Deploy the Program**

   In a new terminal window:

   ```bash
   anchor deploy
   ```

## Interacting with the Program

You can interact with the program using Anchor's testing framework or by building a client application.

### Using Anchor Tests

Modify the `tests/organ_matching.ts` file to include your test cases, then run:

```bash
anchor test
```

### Building a Client Application

Use Anchor's client library to build a frontend or CLI application that interacts with the program.

---

**Example: Interacting with the Program Using Anchor's Client**

```typescript
import * as anchor from '@project-serum/anchor';

// Set up the provider and program
const provider = anchor.Provider.local();
anchor.setProvider(provider);

const program = anchor.workspace.OrganMatching;

// Call the initialize instruction
await program.rpc.initialize(adminPublicKey, {
  accounts: {
    programState: programStatePublicKey,
    payer: provider.wallet.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  },
});
```

---

## Developer Notes

- **Error Handling**: The program uses custom error codes defined in the `ErrorCode` enum. Ensure your client handles these errors gracefully.
- **Event Listening**: You can listen for emitted events like `RecipientUpdated`, `MatchFound`, and `MatchConfirmed` to react to changes in the program state.
- **Program Upgrades**: If you need to upgrade the program, ensure you follow Solana's upgrade procedures and maintain compatibility with existing accounts.

---

## Conclusion

The Organ Matching Program demonstrates how blockchain technology can be applied to critical real-world applications like healthcare. By ensuring transparency, security, and efficiency, this program aims to improve the organ transplantation process for all parties involved.

We encourage developers and healthcare professionals to contribute, provide feedback, and help advance this initiative.

---

Thank you for your interest in the Organ Matching Program!
