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

pub fn process_check_contributions_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [
        maker,         // mut signer
        mint_to_raise, // mint
        fundraiser,    // mut close = maker. seeds = [b"fundraiser".as_ref(), maker.key().as_ref()],
        vault,         // mut
        maker_ata,     // mut ata init_if_needed
        token_program,
        system_program,
        associated_token_program,
        rent_sysvar,
        remaining @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(maker)?;

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

    // Access fundraiser account data to pick amount to raise value
    let data = &mut fundraiser.try_borrow_mut_data()?;
    let fundraiser_state = load_acc_data_mut_unchecked::<FundraiserData>(data)?;

    let vault_token_acc = TokenAccount::from_account_info(vault).unwrap();
    if vault_token_acc.amount() < fundraiser_state.amount_to_raise() {
        return Err(ProgramError::Custom(0));
    }

    let f_bump_seed = [f_bump.to_le()];
    let s_seed = [
        Seed::from(FUNDRAISER_SEED),
        Seed::from(maker.key()),
        Seed::from(&f_bump_seed),
    ];

    let signer_seeds = Signer::from(&s_seed);
    pinocchio_token::instructions::Transfer {
        from: &vault,
        to: &maker_ata,
        amount: vault_token_acc.amount(),
        authority: &fundraiser,
    }
    .invoke_signed(&[signer_seeds])?;

    // let s = &[FUNDRAISER_SEED.as_ref(), maker.key()];
    // pinocchio_system::instructions::TransferWithSeed {
    //     from: fundraiser,
    //     base: fundraiser,
    //     to: maker,
    //     lamports: fundraiser.lamports(),
    //     seed: seed.as_ptr(),
    //     owner: &crate::ID,
    // }
    // .invoke_signed(&[signer_seeds])?;

    let signer_seeds1 = Signer::from(&s_seed);
    pinocchio_system::instructions::Transfer {
        from: fundraiser,
        to: maker,
        lamports: fundraiser.lamports(),
    }
    .invoke_signed(&[signer_seeds1])?;

    fundraiser.close()?;

    Ok(())
}
