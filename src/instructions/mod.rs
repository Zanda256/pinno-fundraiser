mod initialize;

pub use initialize::*;

pub enum FundraiserInstructions {
    Initialize = 0,
    // Take = 1,
    // Cancel = 2,
    // MakeV2 = 3,
}

impl TryFrom<&u8> for FundraiserInstructions {
    type Error = pinocchio::program_error::ProgramError;
    
    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        pinocchio_log::log!("try from : discriminator: {}", value.to_owned());
        match value {
            0 => Ok(FundraiserInstructions::Initialize),
            // 1 => Ok(FundraiserInstrctions::Take),
            // 2 => Ok(FundraiserInstrctions::Cancel),
            // 3 => Ok(FundraiserInstrctions::MakeV2),
            _ => Err(pinocchio::program_error::ProgramError::InvalidInstructionData),
        }
    }
}