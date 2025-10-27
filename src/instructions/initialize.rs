use pinocchio::account_info::AccountInfo;
use pinocchio::program_error::ProgramError;
use pinocchio::{msg, pubkey, ProgramResult};
use pinocchio::pubkey::Pubkey;
use pinocchio::{
    instruction::Signer,
};
use pinocchio::instruction::Seed;
use pinocchio::sysvars::rent::Rent;
use pinocchio::sysvars::Sysvar;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::InitializeAccount;
use spl_associated_token_account::solana_program;
use crate::helpers::{check_signer, load_ix_data, DataLen};
use crate::state::{FundraiserData, InitializeFundraiserIxData, FUNDRAISER_SEED, MIN_AMOUNT_TO_RAISE};

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
        rent_sysvar @ ..
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    msg!("Initialize instruction accounts deserialized successfully!!");

    check_signer(&maker)?;

    msg!("signer checked successfully!!");
    
    if !vault.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    msg!("vault checked successfully!!");
    
    let ix_data = load_ix_data::<InitializeFundraiserIxData>(&instruction_data)?;
    
    pinocchio_log::log!("Initialize instruction data {}", ix_data.duration());

    if !(ix_data.amount_to_raise > MIN_AMOUNT_TO_RAISE.pow(self.mint_to_raise.decimals as u32)){
        return Err(ProgramError::InvalidArgument);
    }
    
    let seed = &[FUNDRAISER_SEED, maker.key().as_ref()];
    let (pda_fundraiser, f_bump) = pubkey::find_program_address(seed, &crate::ID);

    if pda_fundraiser.ne(fundraiser.key()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Ensure that thefundraiser PDA data account to initialize is writable
    if !fundraiser.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    }

    let seed = [Seed::from(FUNDRAISER_SEED), Seed::from(maker.key().as_ref()), Seed::from(&f_bump)];
    let signer_seeds = Signer::from(&seed);

    {
        let b = data[0];
        if fundraiser.owner() != &crate::ID {
            CreateAccount {
                from: maker,
                to: fundraiser,
                lamports: Rent::get()?.minimum_balance(FundraiserData::LEN),
                space: FundraiserData::LEN as u64,
                owner: &crate::ID,
            }.invoke_signed(&[signer_seeds.clone()])?;

            {
                let fundraiser_state = FundraiserData::from_account_info(fundraiser)?;

                escrow_state.set_maker(maker.key());
                escrow_state.set_mint_a(mint_a.key());
                escrow_state.set_mint_b(mint_b.key());
                escrow_state.set_amount_to_receive(amount_to_receive);
                escrow_state.set_amount_to_give(amount_to_give);
                escrow_state.bump = [b];
            }
        }
        else {
            return Err(pinocchio::program_error::ProgramError::IllegalOwner);
        }
    }

    // Ensure that the account to initialize is writable
    if !vault.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Ensure the rent sysvar is valid (you might need additional checks here)
    if rent_sysvar.key() != &solana_program::sysvar::rent::ID {
        return Err(ProgramError::InvalidAccountData);
    }

    // Creating the instruction instance
    let initialize_account_instruction = InitializeAccount {
        account: vault,
        mint: mint_to_raise,
        owner: pda_fundraiser,
        rent_sysvar,
    };

    // Invoking the instruction
    initialize_account_instruction.invoke_signed(signers)?;
    // 
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