use crate::helpers::{DataLen, check_signer, load_acc_data_mut_unchecked, load_ix_data};
use crate::state::{
    FUNDRAISER_SEED, FundraiserData, InitializeFundraiserIxData, MIN_AMOUNT_TO_RAISE,
    SECONDS_PER_DAY,
};
use pinocchio::account_info::AccountInfo;
use pinocchio::instruction::Seed;
use pinocchio::instruction::Signer;
use pinocchio::program_error::{ProgramError, ToStr};
use pinocchio::pubkey::Pubkey;
use pinocchio::sysvars::rent::Rent;
use pinocchio::sysvars::{Sysvar, clock::Clock};
use pinocchio::{ProgramResult, msg, pubkey};
//use pinocchio_associated_token_account::solana_program;
use pinocchio_associated_token_account::instructions::Create as Create_ATA;
use pinocchio_system::instructions::{CreateAccount, CreateAccountWithSeed};
use pinocchio_token::instructions::InitializeAccount;
use pinocchio_token::state::{Mint, TokenAccount};
//use spl_token::solana_program::program_pack::Pack;
use crate::state::ContributorData;

pub fn process_refund_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [
        contributor, // mut signer
        maker,
        mint_to_raise,       // mint
        fundraiser,          // mut
        contributor_account, // init if needed. seeds = [b"contributor", fundraiser.key().as_ref(), contributor.key().as_ref()],
        contributor_ata,     // mut ata
        vault,               // mut
        token_program,
        system_program,
        associated_token_program,
        rent_sysvar,
        remaining @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Check if the fundraising duration has been reached
    // Check if the fundraiser target has been met
    //
    check_signer(contributor)?;

    // validate fundraiser PDA
    let seed = &[FUNDRAISER_SEED, maker.key().as_ref()];
    let (pda_fundraiser, f_bump) = pubkey::find_program_address(seed, &crate::ID);

    msg!("checking fundraiser PDA");
    if pda_fundraiser.ne(fundraiser.key()) {
        return Err(ProgramError::InvalidAccountOwner);
    }
    if fundraiser.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }
    if !fundraiser.is_writable() {
        return Err(ProgramError::Immutable);
    }

    if !fundraiser.is_owned_by(&crate::ID) {
        pinocchio_log::log!("fundraiser.owner() {}", fundraiser.owner());
        return Err(ProgramError::IllegalOwner);
    }

    {
        let data = &mut fundraiser.try_borrow_mut_data()?;
        let fundraiser_state = load_acc_data_mut_unchecked::<FundraiserData>(data)?;
    }

    // validate contributor_account PDA
    if contributor_account.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }
    if !contributor_account.is_writable() {
        return Err(ProgramError::Immutable);
    }

    if !contributor_account.is_owned_by(&crate::ID) {
        return Err(ProgramError::IllegalOwner);
    }

    let mut amount_to_refund: u64 = 0;
    {
        let data = &mut contributor_account.try_borrow_mut_data()?;
        let contributor_account_state = load_acc_data_mut_unchecked::<ContributorData>(data)?;
        amount_to_refund = contributor_account_state.amount();
    }

    // Validate vault
    if !vault.is_writable() {
        return Err(ProgramError::Immutable);
    }

    if !vault.is_owned_by(&token_program.key()) {
        return Err(ProgramError::IllegalOwner);
    }

    let f_seed = [f_bump.to_le()];
    let seed = [
        Seed::from(FUNDRAISER_SEED),
        Seed::from(maker.key()),
        Seed::from(&f_seed),
    ];

    let signer_seeds = Signer::from(&seed);
    pinocchio_token::instructions::Transfer {
        from: &vault,
        to: &contributor_ata,
        amount: amount_to_refund,
        authority: &fundraiser,
    }
    .invoke_signed(&[signer_seeds])?;

    msg!("Transfer successfull");

    Ok(())
}
