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

    if pda_fundraiser.ne(fundraiser.key()) {
        msg!("fundraiser PDA check failed");
        return Err(ProgramError::InvalidAccountOwner);
    }
    pinocchio_log::log!("fundraiser.data_len() {}", fundraiser.data_len());
    pinocchio_log::log!("fundraiser.owner() {}", fundraiser.owner());
    let mut empty: bool = fundraiser.data_is_empty();
    pinocchio_log::log!("empty {}", empty);
    if empty {
        return Err(ProgramError::UninitializedAccount);
    }

    let mutable = fundraiser.is_writable();
    pinocchio_log::log!("checking fundraiser mutability {}", mutable);
    if !mutable {
        return Err(ProgramError::Immutable);
    }

    let is_owned = fundraiser.is_owned_by(&crate::ID);
    pinocchio_log::log!("checking fundraiser owner is our program {}", is_owned);
    if !is_owned {
        pinocchio_log::log!("fundraiser.owner() {}", fundraiser.owner());
        return Err(ProgramError::IllegalOwner);
    }

    let mut amt_to_raise: u64 = 0;
    let mut vault_token_amt: u64 = 0;
    // Access fundraiser account data to pick amount to raise value
    {
        let data = &mut fundraiser.try_borrow_mut_data()?;
        pinocchio_log::log!("fundraiser state data borrowed mutably");
        let fundraiser_state = load_acc_data_mut_unchecked::<FundraiserData>(data)?;
        pinocchio_log::log!("fetched fundraiser state {}", is_owned);
        amt_to_raise = fundraiser_state.amount_to_raise();
        pinocchio_log::log!("amt_to_raise : {}", amt_to_raise);

        let vault_token_acc = TokenAccount::from_account_info(vault).unwrap();
        pinocchio_log::log!("deserialized vault_token_acc");
        let amt_in_vault = vault_token_acc.amount();
        pinocchio_log::log!("amt_in_vault : {}", amt_in_vault);
        if amt_in_vault < amt_to_raise {
            return Err(ProgramError::Custom(0));
        }
        vault_token_amt = vault_token_acc.amount()
    }

    pinocchio_log::log!("verified amount in vault_token_acc");
    let f_bump_seed = [f_bump.to_le()];
    let s_seed = [
        Seed::from(FUNDRAISER_SEED),
        Seed::from(maker.key()),
        Seed::from(&f_bump_seed),
    ];

    pinocchio_log::log!("Transferring tokens from vault_token_acc");
    let signer_seeds = Signer::from(&s_seed);
    pinocchio_token::instructions::Transfer {
        from: &vault,
        to: &maker_ata,
        amount: vault_token_amt,
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

    // let signer_seeds1 = Signer::from(&s_seed);
    // pinocchio_system::instructions::Transfer {
    //     from: fundraiser,
    //     to: maker,
    //     lamports: fundraiser.lamports(),
    // }
    // .invoke_signed(&[signer_seeds1])?;
    //
    pinocchio_log::log!("closing fundraiser PDA account");
    crate::helpers::close_account(fundraiser, maker)?;

    pinocchio_log::log!("Check contributions finished successfully");
    //fundraiser.close()?;

    Ok(())
}
