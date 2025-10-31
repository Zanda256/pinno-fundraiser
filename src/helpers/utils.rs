use bytemuck::{Pod, Zeroable, try_from_bytes, try_from_bytes_mut};
use pinocchio::instruction::Seed;
use pinocchio::instruction::Signer;
use pinocchio::sysvars::Sysvar;
use pinocchio::sysvars::rent::Rent;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError};
use pinocchio_system::instructions::CreateAccount;

pub trait DataLen {
    const LEN: usize;
}

pub trait Initialized {
    fn is_initialized(&self) -> bool;
}

#[inline(always)]
pub fn load_acc_data_mut<T: DataLen + Initialized>(bytes: &mut [u8]) -> Result<&mut T, ProgramError>
where
    T: DataLen + Pod + Zeroable,
{
    load_acc_data_mut_unchecked::<T>(bytes).and_then(|acc| {
        if acc.is_initialized() {
            Ok(acc)
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    })
}

#[inline(always)]
pub fn load_acc_data_mut_unchecked<T>(bytes: &mut [u8]) -> Result<&mut T, ProgramError>
where
    T: DataLen + Pod + Zeroable,
{
    if bytes.len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    bytemuck::try_from_bytes_mut::<T>(bytes).map_err(|_| ProgramError::InvalidAccountData)
}

// #[inline(always)]
// pub unsafe fn load_acc<T: DataLen + Initialized>(bytes: &[u8]) -> Result<&T, ProgramError> {
//     load_acc_unchecked::<T>(bytes).and_then(|acc| {
//         if acc.is_initialized() {
//             Ok(acc)
//         } else {
//             Err(ProgramError::UninitializedAccount)
//         }
//     })
// }

// #[inline(always)]
// pub unsafe fn load_acc_unchecked<T: DataLen>(bytes: &[u8]) -> Result<&T, ProgramError> {
//     if bytes.len() != T::LEN {
//         return Err(ProgramError::InvalidAccountData);
//     }
//     try_from_bytes(bytes).map_err(|_| ProgramError::InvalidAccountData)
// }

#[inline(always)]
pub fn load_ix_data<T>(bytes: &[u8]) -> Result<&T, ProgramError>
where
    T: DataLen + Pod + Zeroable,
{
    // pinocchio_log::log!(
    //     "load_ix_data : bytes.len(): {} - T::LEN: {}",
    //     bytes.len(),
    //     T::LEN
    // );

    if bytes.len() != T::LEN {
        pinocchio_log::log!(
            "Unexpected data length load_ix_data : bytes.len(): {} - T::LEN: {}",
            bytes.len(),
            T::LEN
        );
        return Err(ProgramError::InvalidInstructionData.into());
    }

    try_from_bytes(bytes).map_err(|_| ProgramError::InvalidInstructionData)
}

#[inline(always)]
pub fn create_pda_account<S>(
    payer: &AccountInfo,
    account: &AccountInfo,
    signer_seeds: &[Seed],
) -> Result<(), ProgramError>
where
    S: DataLen,
{
    let signer_seeds = Signer::from(signer_seeds);
    CreateAccount {
        from: payer,
        to: account,
        lamports: Rent::get()?.minimum_balance(S::LEN),
        space: S::LEN as u64,
        owner: &crate::ID,
    }
    .invoke_signed(&[signer_seeds])?;

    Ok(())
}
