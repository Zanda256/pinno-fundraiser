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

pub fn process_initialize_instruction(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [
        maker,
        mint_to_raise,
        fundraiser,
        vault,
        system_program,
        token_program,
        associated_token_program,
        rent_sysvar,
        remaining @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    msg!("Initialize instruction accounts deserialized successfully!!");

    // Verify maker is signer
    check_signer(&maker)?;

    //msg!("signer checked successfully!!");

    // Verify vault is not initialized
    if !vault.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    //msg!("vault checked successfully!!");

    if !fundraiser.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let ix_data = load_ix_data::<InitializeFundraiserIxData>(&instruction_data)?;

    if ix_data.duration() == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }

    //pinocchio_log::log!("Initialize instruction data {}", ix_data.duration());

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

    if !(ix_data.amount_to_raise() > MIN_AMOUNT_TO_RAISE.pow(6 as u32)) {
        return Err(ProgramError::InvalidArgument);
    }

    // msg!("amount to raise checked successfully!!");

    let seed = &[FUNDRAISER_SEED, maker.key().as_ref()];
    let (pda_fundraiser, f_bump) = pubkey::find_program_address(seed, &crate::ID);

    if pda_fundraiser.ne(fundraiser.key()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Ensure that the account to initialize is writable
    if !vault.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Ensure that thefundraiser PDA data account to initialize is writable
    if !fundraiser.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    }
    let b_seed = f_bump.to_le_bytes();

    let seed = [
        Seed::from(FUNDRAISER_SEED),
        Seed::from(maker.key().as_ref()),
        Seed::from(&b_seed),
    ];
    let signer_seeds = Signer::from(&seed);

    // pinocchio_log::log!("fundraiser.owner() {}", fundraiser.owner());
    // pinocchio_log::log!("&crate::ID {}", &crate::ID);
    {
        msg!("creating PDA account");
        CreateAccount {
            from: maker,
            to: fundraiser,
            lamports: Rent::get()?.minimum_balance(FundraiserData::LEN),
            space: FundraiserData::LEN as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&[signer_seeds.clone()])?;

        // CreateAccountWithSeed {
        //     from: maker,
        //     to: fundraiser,
        //     base: Some(&maker),
        //     seed: "fundraiser",
        //     lamports: 1u64, //Rent::get()?.minimum_balance(FundraiserData::LEN),
        //     space: FundraiserData::LEN as u64,
        //     owner: &crate::ID,
        // }
        // .invoke_signed(&[signer_seeds.clone()])?;

        // Get the Clock sysvar
        let clock = Clock::get()?;
        // Access the current slot
        // let current_slot = clock.slot;

        // Access the Unix timestamp (in seconds)
        let unix_timestamp = clock.unix_timestamp;

        let data = &mut fundraiser.try_borrow_mut_data()?;
        let fundraiser_state = load_acc_data_mut_unchecked::<FundraiserData>(data)?;

        fundraiser_state.set_maker(maker.key());
        fundraiser_state.set_mint_to_raise(mint_to_raise.key());
        fundraiser_state.set_amount_to_raise(ix_data.amount_to_raise());
        fundraiser_state.set_current_amount(0u64);
        fundraiser_state.set_time_started(unix_timestamp as u64);
        fundraiser_state.set_duration(ix_data.duration());
        fundraiser_state.set_bump(f_bump);
        fundraiser_state.add_padding();

        msg!("PDA account created");
        // drop(fundraiser_state);
        // drop(vault);
        // drop(mint_data);
    }

    Create_ATA {
        funding_account: maker,
        account: vault,
        wallet: fundraiser,
        mint: mint_to_raise,
        system_program: system_program,
        token_program: token_program,
    }
    .invoke_signed(&[signer_seeds])?;

    // Creating the instruction instance
    // let initialize_account_instruction = InitializeAccount {
    //     account: vault,
    //     mint: mint_to_raise,
    //     owner: fundraiser,
    //     rent_sysvar,
    // };

    // // Invoking the instruction
    // initialize_account_instruction.invoke_signed(&[signer_seeds])?;

    // // Ensure that the 'vault' account is writable
    // if !vault.is_writable {
    //     return Err(ProgramError::InvalidAccountData);
    // }

    // let current_time = Clock::get()?.unix_timestamp;
    // let elapsed = current_time - fundraiser.get_time_started();
    //
    // if elapsed >= ONE_WEEK {
    //     // One week has passed
    // }

    Ok(())
}
