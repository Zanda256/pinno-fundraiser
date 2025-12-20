use {
    num_derive::FromPrimitive,
    pinocchio::program_error::{ProgramError, ToStr},
    thiserror::Error,
};

#[derive(Clone, Debug, Eq, Error, PartialEq, FromPrimitive)]
pub enum FundraiserError {
    #[error("Maximum contributor percentage reached")]
    ContributorMaxPercentage,
    #[error("Lamport balance below rent-exempt threshold")]
    NotRentExempt,
}

impl From<FundraiserError> for ProgramError {
    fn from(e: FundraiserError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl ToStr for FundraiserError {
    fn to_str<E>(&self) -> &'static str {
        match self {
            FundraiserError::ContributorMaxPercentage => {
                "Error: maximum individual contributor percentage reached"
            }
            FundraiserError::NotRentExempt => "Error: Lamport balance below rent-exempt threshold",
        }
    }
}
