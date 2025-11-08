mod check_contribution;
mod contribute;
mod initialize;
mod refund;

pub use check_contribution::*;
pub use contribute::*;
pub use initialize::*;
pub use refund::*;

pub enum FundraiserInstructions {
    Initialize = 0,
    Contribute = 1,
    Refund = 2,
    Check = 3,
}

impl TryFrom<&u8> for FundraiserInstructions {
    type Error = pinocchio::program_error::ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        pinocchio_log::log!("try from : discriminator: {}", value.to_owned());
        match value {
            0 => Ok(FundraiserInstructions::Initialize),
            1 => Ok(FundraiserInstructions::Contribute),
            2 => Ok(FundraiserInstructions::Refund),
            // 3 => Ok(FundraiserInstrctions::MakeV2),
            _ => Err(pinocchio::program_error::ProgramError::InvalidInstructionData),
        }
    }
}
