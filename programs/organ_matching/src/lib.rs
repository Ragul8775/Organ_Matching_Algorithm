use anchor_lang::prelude::*;


declare_id!("CF3KfcaDXNM7VriAbjHz2MxSFFZUYqCrmPKn62pZEnjd");

#[program]
pub mod organ_matching {
    use super::*;

    // Initialize the program state with admin
    pub fn initialize(ctx: Context<Initialize>, admin: Pubkey) -> Result<()> {
        let state = &mut ctx.accounts.program_state;
        state.admin = admin;
        state.recipient_count = 0;
        state.paused = false;
        Ok(())
    }

    // Add or update medical authority
    pub fn manage_medical_authority(
        ctx: Context<ManageMedicalAuthority>,
        authority: Pubkey,
        is_active: bool,
    ) -> Result<()> {
        require!(
            ctx.accounts.program_state.admin == ctx.accounts.admin.key(),
            ErrorCode::UnauthorizedAdmin
        );

        let auth_account = &mut ctx.accounts.medical_authority;
        auth_account.authority = authority;
        auth_account.is_active = is_active;
        auth_account.verified_matches = 0;
        
        Ok(())
    }

    // Add or update recipient with validation
    pub fn upsert_recipient(
        ctx: Context<UpsertRecipient>,
        recipient_data: RecipientData,
    ) -> Result<()> {
        require!(
            ctx.accounts.medical_authority.is_active,
            ErrorCode::UnauthorizedMedicalAuthority
        );
        
        validate_recipient_data(&recipient_data)?;

        let recipient = &mut ctx.accounts.recipient;
        let is_new = recipient.data.created_at == 0;

        if is_new {
            recipient.authority = *ctx.accounts.patient.key;
            recipient.data = recipient_data;
            recipient.data.created_at = Clock::get()?.unix_timestamp;
            recipient.data.last_updated = recipient.data.created_at;
            recipient.status = RecipientStatus::Active;
            
            let state = &mut ctx.accounts.program_state;
            state.recipient_count = state.recipient_count.checked_add(1)
                .ok_or(ErrorCode::MathOverflow)?;
        } else {
            require!(
                recipient.authority == *ctx.accounts.patient.key,
                ErrorCode::UnauthorizedUpdate
            );
            
            // Update only mutable fields
            recipient.data.medical_urgency = recipient_data.medical_urgency;
            recipient.data.geographical_distance = recipient_data.geographical_distance;
            recipient.data.last_updated = Clock::get()?.unix_timestamp;
        }

        emit!(RecipientUpdated {
            recipient: recipient.key(),
            medical_urgency: recipient.data.medical_urgency,
            timestamp: recipient.data.last_updated,
        });

        Ok(())
    }

    // Add donor with validation
    pub fn add_donor(
        ctx: Context<AddDonor>,
        donor_data: DonorData,
    ) -> Result<()> {
        require!(
            ctx.accounts.medical_authority.is_active,
            ErrorCode::UnauthorizedMedicalAuthority
        );

        validate_donor_data(&donor_data)?;

        let donor = &mut ctx.accounts.donor;
        donor.authority = *ctx.accounts.authority.key;
        donor.data = donor_data;
        donor.created_at = Clock::get()?.unix_timestamp;
        donor.status = DonorStatus::Active;

        Ok(())
    }

    // Find best match with improved efficiency
    pub fn find_best_match(ctx: Context<FindBestMatch>) -> Result<()> {
        require!(
            ctx.accounts.medical_authority.is_active,
            ErrorCode::UnauthorizedMedicalAuthority
        );
        
        require!(
            ctx.accounts.donor.status == DonorStatus::Active,
            ErrorCode::InvalidDonorStatus
        );
    
        let donor_data = &ctx.accounts.donor.data;
        let current_time = Clock::get()?.unix_timestamp;
        
        let mut best_match: Option<(Pubkey, u64)> = None;
        let mut highest_score = 0u64;
    
        // Process each remaining account
        for account_info in ctx.remaining_accounts.iter().cloned() {
            // Verify account ownership
            if account_info.owner != ctx.program_id {
                continue;
            }
    
            // Try to deserialize the recipient account
            let recipient = match Account::<RecipientAccount>::try_from(&account_info) {
                Ok(r) => r,
                Err(_) => continue,
            };
    
            // Skip inactive recipients
            if recipient.status != RecipientStatus::Active {
                continue;
            }
    
            // Calculate match score
            if let Some(score) = calculate_match_score(
                donor_data,
                &recipient.data,
                current_time,
            )? {
                if score > highest_score {
                    highest_score = score;
                    best_match = Some((*account_info.key, score));
                }
            }
        }
    
        // Process the best match
        match best_match {
            Some((recipient_pubkey, score)) => {
                let match_account = &mut ctx.accounts.match_account;
                match_account.recipient = recipient_pubkey;
                match_account.donor = ctx.accounts.donor.key();
                match_account.score = score;
                match_account.timestamp = current_time;
                match_account.status = MatchStatus::Pending;
    
                emit!(MatchFound {
                    donor: ctx.accounts.donor.key(),
                    recipient: recipient_pubkey,
                    score,
                    timestamp: current_time,
                });
    
                Ok(())
            }
            None => Err(ErrorCode::NoCompatibleRecipient.into())
        }
    }

    // Helper function to calculate match score

    // Confirm match by medical authority
    pub fn confirm_match(ctx: Context<ConfirmMatch>) -> Result<()> {
        require!(
            ctx.accounts.medical_authority.is_active,
            ErrorCode::UnauthorizedMedicalAuthority
        );

        let match_account = &mut ctx.accounts.match_account;
        require!(
            match_account.status == MatchStatus::Pending,
            ErrorCode::InvalidMatchStatus
        );

        let recipient = &mut ctx.accounts.recipient;
        let donor = &mut ctx.accounts.donor;

        // Update statuses
        match_account.status = MatchStatus::Confirmed;
        recipient.status = RecipientStatus::Matched;
        donor.status = DonorStatus::Matched;

        // Update medical authority stats
        let auth_account = &mut ctx.accounts.medical_authority;
        auth_account.verified_matches = auth_account.verified_matches
            .checked_add(1)
            .ok_or(ErrorCode::MathOverflow)?;

        emit!(MatchConfirmed {
            match_id: match_account.key(),
            donor: donor.key(),
            recipient: recipient.key(),
            medical_authority: auth_account.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}

fn calculate_match_score(
    donor: &DonorData,
    recipient: &RecipientData,
    current_time: i64,
) -> Result<Option<u64>> {
    // Basic compatibility checks
    if donor.blood_type != recipient.blood_type || 
       donor.organ_type != recipient.organ_type {
        return Ok(None);
    }

    // HLA matching score (0-50 points)
    let mut hla_score = 0u64;
    for (d, r) in donor.hla_markers.iter().zip(recipient.hla_markers.iter()) {
        if d == r {
            hla_score += 10;
        }
    }

    // Medical urgency score (0-100 points)
    let urgency_score = recipient.medical_urgency as u64;

    // Wait time score (0-50 points)
    let wait_time = current_time - recipient.created_at;
    let wait_score = std::cmp::min(50, (wait_time / (30 * 24 * 60 * 60)) as u64);

    // Age score for pediatric priority (0-50 points)
    let age_score = if recipient.age <= 18 {
        50u64
    } else {
        0u64
    };

    // Geographical score (0-50 points)
    let geo_score = 50u64.saturating_sub(recipient.geographical_distance as u64 / 100);

    // Calculate total score with overflow checking
    let total_score = hla_score
        .checked_add(urgency_score)
        .and_then(|score| score.checked_add(wait_score))
        .and_then(|score| score.checked_add(age_score))
        .and_then(|score| score.checked_add(geo_score))
        .ok_or(ErrorCode::MathOverflow)?;

    Ok(Some(total_score))
}


// Account structures
#[account]
pub struct ProgramState {
    pub admin: Pubkey,
    pub recipient_count: u32,
    pub paused: bool,
}

#[account]
pub struct MedicalAuthority {
    pub authority: Pubkey,
    pub is_active: bool,
    pub verified_matches: u32,
}

#[account]
pub struct RecipientAccount {
    pub authority: Pubkey,
    pub data: RecipientData,
    pub status: RecipientStatus,
}

#[account]
pub struct DonorAccount {
    pub authority: Pubkey,
    pub data: DonorData,
    pub created_at: i64,
    pub status: DonorStatus,
}

#[account]
pub struct MatchAccount {
    pub recipient: Pubkey,
    pub donor: Pubkey,
    pub score: u64,
    pub timestamp: i64,
    pub status: MatchStatus,
}

// Data structures
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
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

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DonorData {
    pub hla_markers: [u8; 5],
    pub blood_type: BloodType,
    pub organ_type: OrganType,
    pub medical_notes: String,
}

// Enums for better type safety
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
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

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum OrganType {
    Kidney,
    Liver,
    Heart,
    Lung,
    Pancreas,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum RecipientStatus {
    Active,
    Matched,
    Removed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum DonorStatus {
    Active,
    Matched,
    Withdrawn,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum MatchStatus {
    Pending,
    Confirmed,
    Rejected,
}

// Events
#[event]
pub struct RecipientUpdated {
    pub recipient: Pubkey,
    pub medical_urgency: u8,
    pub timestamp: i64,
}

#[event]
pub struct MatchFound {
    pub donor: Pubkey,
    pub recipient: Pubkey,
    pub score: u64,
    pub timestamp: i64,
}

#[event]
pub struct MatchConfirmed {
    pub match_id: Pubkey,
    pub donor: Pubkey,
    pub recipient: Pubkey,
    pub medical_authority: Pubkey,
    pub timestamp: i64,
}

// Constants
const MAX_BATCH_SIZE: usize = 100;
const MAX_MEDICAL_NOTES_LENGTH: usize = 1000;

// Validation functions
fn validate_recipient_data(data: &RecipientData) -> Result<()> {
    require!(
        data.medical_urgency <= 100,
        ErrorCode::InvalidMedicalUrgency
    );
    require!(
        data.age <= 120,
        ErrorCode::InvalidAge
    );
    require!(
        data.medical_notes.len() <= MAX_MEDICAL_NOTES_LENGTH,
        ErrorCode::MedicalNotesTooLong
    );
    Ok(())
}

fn validate_donor_data(data: &DonorData) -> Result<()> {
    require!(
        data.medical_notes.len() <= MAX_MEDICAL_NOTES_LENGTH,
        ErrorCode::MedicalNotesTooLong
    );
    Ok(())
}


// Context structs
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 4 + 1,
        seeds = [b"program_state"],
        bump
    )]
    pub program_state: Account<'info, ProgramState>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ManageMedicalAuthority<'info> {
    #[account(mut)]
    pub program_state: Account<'info, ProgramState>,
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + 32 + 1 + 4,
        seeds = [b"medical_authority", authority.key().as_ref()],
        bump
    )]
    pub medical_authority: Account<'info, MedicalAuthority>,
    pub admin: Signer<'info>,
     /// CHECK: This account is used for its public key in the seeds
     pub authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpsertRecipient<'info> {
    #[account(mut)]
    pub program_state: Account<'info, ProgramState>,
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + 32 + RecipientData::LEN + 1,
        seeds = [b"recipient", patient.key().as_ref()],
        bump
    )]
    pub recipient: Account<'info, RecipientAccount>,
    pub medical_authority: Account<'info, MedicalAuthority>,
    pub patient: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddDonor<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + 32 + DonorData::LEN + 8 + 1,
        seeds = [b"donor", authority.key().as_ref()],
        bump
    )]
    pub donor: Account<'info, DonorAccount>,
    pub medical_authority: Account<'info, MedicalAuthority>,
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FindBestMatch<'info> {
    #[account(
        constraint = donor.status == DonorStatus::Active
    )]
    pub donor: Account<'info, DonorAccount>,
    pub medical_authority: Account<'info, MedicalAuthority>,
    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 32 + 8 + 8 + 1,
        seeds = [b"match", donor.key().as_ref()],
        bump
    )]
    pub match_account: Account<'info, MatchAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ConfirmMatch<'info> {
    #[account(mut)]
    pub match_account: Account<'info, MatchAccount>,
    #[account(mut)]
    pub recipient: Account<'info, RecipientAccount>,
    #[account(mut)]
    pub donor: Account<'info, DonorAccount>,
    #[account(mut)]
    pub medical_authority: Account<'info, MedicalAuthority>,
    pub authority: Signer<'info>,
}

// Implementation blocks for account sizes
impl RecipientData {
    const LEN: usize = 
        1 +  // medical_urgency
        4 +  // geographical_distance
        5 +  // hla_markers
        1 +  // blood_type
        1 +  // organ_type
        1 +  // age
        8 +  // created_at
        8 +  // last_updated
        4 + MAX_MEDICAL_NOTES_LENGTH; // medical_notes (String)
}

impl DonorData {
    const LEN: usize = 
        5 +  // hla_markers
        1 +  // blood_type
        1 +  // organ_type
        4 + MAX_MEDICAL_NOTES_LENGTH; // medical_notes (String)
}

// Error codes
#[error_code]
pub enum ErrorCode {
    #[msg("No compatible recipient found")]
    NoCompatibleRecipient,
    #[msg("Recipient account not found")]
    RecipientAccountNotFound,
    #[msg("Unauthorized admin")]
    UnauthorizedAdmin,
    #[msg("Unauthorized medical authority")]
    UnauthorizedMedicalAuthority,
    #[msg("Unauthorized update")]
    UnauthorizedUpdate,
    #[msg("Invalid donor status")]
    InvalidDonorStatus,
    #[msg("Invalid match status")]
    InvalidMatchStatus,
    #[msg("Invalid medical urgency value")]
    InvalidMedicalUrgency,
    #[msg("Invalid age value")]
    InvalidAge,
    #[msg("Medical notes too long")]
    MedicalNotesTooLong,
    #[msg("Math overflow occurred")]
    MathOverflow,
}

// Helper functions for blood type compatibility
impl BloodType {
    pub fn is_compatible_donor(&self, recipient: &BloodType) -> bool {
        match (self, recipient) {
            (BloodType::ONegative, _) => true,
            (BloodType::OPositive, BloodType::OPositive | BloodType::APositive | BloodType::BPositive | BloodType::ABPositive) => true,
            (BloodType::ANegative, BloodType::ANegative | BloodType::ABNegative) => true,
            (BloodType::APositive, BloodType::APositive | BloodType::ABPositive) => true,
            (BloodType::BNegative, BloodType::BNegative | BloodType::ABNegative) => true,
            (BloodType::BPositive, BloodType::BPositive | BloodType::ABPositive) => true,
            (BloodType::ABNegative, BloodType::ABNegative) => true,
            (BloodType::ABPositive, BloodType::ABPositive) => true,
            _ => false,
        }
    }
}

// Tests module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blood_type_compatibility() {
        assert!(BloodType::ONegative.is_compatible_donor(&BloodType::ABPositive));
        assert!(BloodType::ONegative.is_compatible_donor(&BloodType::ONegative));
        assert!(!BloodType::ABPositive.is_compatible_donor(&BloodType::ONegative));
    }

    #[test]
    fn test_calculate_match_score() {
        let donor = DonorData {
            hla_markers: [1, 1, 1, 1, 1],
            blood_type: BloodType::ONegative,
            organ_type: OrganType::Kidney,
            medical_notes: String::new(),
        };

        let recipient = RecipientData {
            medical_urgency: 80,
            geographical_distance: 100,
            hla_markers: [1, 1, 1, 1, 1],
            blood_type: BloodType::ONegative,
            organ_type: OrganType::Kidney,
            age: 15,
            created_at: 0,
            last_updated: 0,
            medical_notes: String::new(),
        };

        let current_time = 30 * 24 * 60 * 60; // 30 days
        let score = calculate_match_score(&donor, &recipient, current_time)
            .unwrap()
            .unwrap();
        
        assert!(score > 0);
    }
}