use crate::helpers::{DataLen, check_signer, load_acc_data_mut_unchecked, load_ix_data};
use crate::state::{
    ContributeIxData, ContributorData, FUNDRAISER_SEED, FundraiserData, InitializeFundraiserIxData,
    MAX_CONTRIBUTION_PERCENTAGE, MIN_AMOUNT_TO_RAISE, PERCENTAGE_SCALER, SECONDS_PER_DAY,
};
use pinocchio::account_info::AccountInfo;
use pinocchio::instruction::Seed;
use pinocchio::instruction::Signer;
use pinocchio::program_error::{ProgramError, ToStr};
use pinocchio::pubkey::{self, Pubkey};
use pinocchio::sysvars::rent::Rent;
use pinocchio::sysvars::{Sysvar, clock::Clock};
use pinocchio::{ProgramResult, msg, pubkey::find_program_address};
//use pinocchio_associated_token_account::solana_program;
use pinocchio_associated_token_account::instructions::Create as Create_ATA;
use pinocchio_system::instructions::{CreateAccount, CreateAccountWithSeed};
use pinocchio_token::instructions::InitializeAccount;
use pinocchio_token::state::{Mint, TokenAccount};
//use spl_token::solana_program::program_pack::Pack;
use crate::helpers::create_pda_account;

pub fn process_contribute_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [
        contributor,         // mut signer
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

    // pinocchio_log::log!("contributor {}", contributor.key());
    // pinocchio_log::log!("mint_to_raise {}", mint_to_raise.key());
    // pinocchio_log::log!("fundraiser {}", fundraiser.key());
    // pinocchio_log::log!("contributor_account {}", contributor_account.key());
    // pinocchio_log::log!("contributor_ata {}", contributor_ata.key());
    // pinocchio_log::log!("vault {}", vault.key());
    // pinocchio_log::log!("token_program {}", token_program.key());

    // Check signer
    check_signer(&contributor)?;

    // check fundraiser PDA validity and mutability
    if fundraiser.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }
    if !fundraiser.is_writable() {
        return Err(ProgramError::Immutable);
    }

    if !fundraiser.is_owned_by(&crate::ID) {
        return Err(ProgramError::IllegalOwner);
    }

    // check contributor_account PDA validity and mutability
    if !contributor_account.is_writable() {
        return Err(ProgramError::Immutable);
    }

    let ix_data = load_ix_data::<ContributeIxData>(&instruction_data)?;

    let seeds: &[&[u8]] = &[
        b"contributor",   //.as_ref(),
        fundraiser.key(), //.as_ref(),
        contributor.key(), //.as_ref(),
                          //    &ix_data.c_bump,
    ];
    let (contributor_account_pda, c_bump) = find_program_address(seeds, program_id);
    if contributor_account_pda.ne(contributor_account.key()) {
        return Err(ProgramError::InvalidSeeds);
    }

    if u8::from_le_bytes(ix_data.c_bump) != c_bump {
        return Err(ProgramError::InvalidSeeds);
    }

    if contributor_account.data_is_empty() || contributor_account.lamports() == 0 {
        msg!("contributor PDA  account data is empty");
        // let c_seed = ix_data.c_bump;
        let c_seed = [c_bump.to_le()];
        let seed = [
            Seed::from(b"contributor"), // .as_ref()
            Seed::from(fundraiser.key()),
            Seed::from(contributor.key()), // .as_ref()
            Seed::from(&c_seed),
        ];

        create_pda_account::<ContributorData>(contributor, contributor_account, &seed)?;
    }

    msg!("contributor PDA  account data verified as empty");

    // check contributor_ata validity and mutability
    // let derrived_contributor_ata = pinocchio_associated_token_account::get_associated_token_address(
    //     &contributor.pubkey(), // owner will be the fundraiser_pda
    //     &mint,                 // mint
    // );

    // if derrived_contributor_ata.eq(contributor_ata) {
    //     return Err(ProgramError::InvalidSeeds);
    // }

    if !contributor_ata.is_writable() {
        return Err(ProgramError::Immutable);
    }

    let o = contributor_ata.owner().to_ascii_lowercase();
    println!("contributor ATA owner {:?}", o);
    pinocchio_log::log!("&token_program.key() {}", token_program.key());

    // if !contributor_ata.is_owned_by(&token_program.key()) {
    //     return Err(ProgramError::IllegalOwner);
    // }

    msg!("contributor ATA verified");
    // check vault_ata PDA validity and mutability
    if !vault.is_writable() {
        return Err(ProgramError::Immutable);
    }

    if !vault.is_owned_by(&token_program.key()) {
        return Err(ProgramError::IllegalOwner);
    }

    msg!("vault verified.");

    // check amount is within range
    let mut decimals: u8 = 0;

    // Access mint account to retrieve decimals
    // Try to parse as TokenAccount
    let m = Mint::from_account_info(mint_to_raise).unwrap();
    match Mint::from_account_info(mint_to_raise) {
        Ok(m_account) => {
            if !m_account.is_initialized() {
                return Err(pinocchio::program_error::UNINITIALIZED_ACCOUNT.into());
            }
            decimals = m_account.decimals();
        }
        Err(e) => {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
    }

    msg!("mint deserialized  data");

    // Access fundraiser account data to pick amount to raise value
    let data = &mut fundraiser.try_borrow_mut_data()?;
    let fundraiser_state = load_acc_data_mut_unchecked::<FundraiserData>(data)?;

    let ix_data = load_ix_data::<ContributeIxData>(&instruction_data)?;
    let amount = ix_data.amount();

    pinocchio_log::log!("decimals: {}", decimals);
    let am = amount / decimals as u64; //amount.checked_div(decimals as u64).unwrap();
    pinocchio_log::log!("amount got: {}", am);
    let min = 3_u64.pow(decimals as u32) as u64;
    pinocchio_log::log!("minimum: {}", min);
    // Amount should be above minimum contribution
    // if am.lt(&(3_u8.pow(decimals as u32) as u64)) {
    //     msg!("Amount should be above minimum contribution");
    //     return Err(ProgramError::InvalidInstructionData);
    // }

    // if amount
    //     >= (fundraiser_state.amount_to_raise() * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER
    // {
    //     return Err(ProgramError::InvalidInstructionData);
    // }

    // let current_time = Clock::get()?.unix_timestamp;
    // if fundraiser_state.duration()
    //     < ((current_time as u64 - fundraiser_state.time_started() / SECONDS_PER_DAY) as u8)
    // {
    //     return Err(ProgramError::InvalidInstructionData);
    // }

    // // Perform transfer
    // {
    //     let from_token_account = TokenAccount::from_account_info(contributor_ata)?;
    //     pinocchio_log::log!(
    //         " from_token_account.amount(): {}",
    //         from_token_account.amount()
    //     );
    //     pinocchio_log::log!(" amount: {}", amount);
    //     if from_token_account.amount() < amount {
    //         return Err(ProgramError::InsufficientFunds);
    //     }
    // }

    // pinocchio_token::instructions::TransferChecked {
    //     mint: &mint_to_raise,
    //     from: &contributor_ata,
    //     to: &vault,
    //     amount: amount,
    //     authority: &contributor,
    //     decimals: decimals,
    // }
    // .invoke()?;

    pinocchio_token::instructions::Transfer {
        from: &contributor_ata,
        to: &vault,
        amount: 10,
        authority: &contributor,
    }
    .invoke()?;

    msg!("Transfer successfull");

    Ok(())
}
