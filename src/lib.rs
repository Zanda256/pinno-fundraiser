use pinocchio::{ProgramResult, account_info::AccountInfo, entrypoint, msg, pubkey::Pubkey};

mod errors;
mod helpers;
mod instructions;
mod state;
mod tests;
use instructions::*;

pinocchio_pubkey::declare_id!("HAV1KKoQW1ckwgvUP8fCXRfjZ4gGfHeu7VhfMej8Bw8i");

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    assert_eq!(program_id, &ID);

    pinocchio_log::log!("instruction data length: {}", instruction_data.len());

    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(pinocchio::program_error::ProgramError::InvalidInstructionData)?;

    pinocchio_log::log!(
        "discriminator: {}\ndata length: {}",
        discriminator.to_owned(),
        data.len()
    );

    match FundraiserInstructions::try_from(discriminator)? {
        FundraiserInstructions::Initialize => {
            instructions::process_initialize_instruction(accounts, data)?;
        }
        FundraiserInstructions::Contribute => {
            instructions::process_contribute_instruction(program_id, accounts, data)?;
        }
        FundraiserInstructions::Refund => {
            instructions::process_refund_instruction(accounts, data)?;
        }
        _ => {
            pinocchio_log::log!(
                "unknown instruction discriminator: {}",
                discriminator.to_owned(),
            );
            return Err(pinocchio::program_error::ProgramError::InvalidInstructionData);
        }
    }

    Ok(())
}
