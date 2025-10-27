use pinocchio::{account_info::AccountInfo, program_error::ProgramError};
use bytemuck::{try_from_bytes, Pod, Zeroable};

pub trait DataLen {
    const LEN: usize;
}

pub trait Initialized {
    fn is_initialized(&self) -> bool;
}


#[inline(always)]
pub unsafe fn load_acc<T: DataLen + Initialized>(bytes: &[u8]) -> Result<&T, ProgramError> {
    load_acc_unchecked::<T>(bytes).and_then(|acc| {
        if acc.is_initialized() {
            Ok(acc)
        } else {
            Err(ProgramError::UninitializedAccount)
        }
    })
}

#[inline(always)]
pub unsafe fn load_acc_unchecked<T: DataLen>(bytes: &[u8]) -> Result<&T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(&*(bytes.as_ptr() as *const T))
}

#[inline(always)]
pub fn load_ix_data<T>(bytes: &[u8]) -> Result<&T, ProgramError> 
where T: DataLen + Pod + Zeroable
{
    pinocchio_log::log!("load_ix_data : bytes.len(): {} - T::LEN: {}", bytes.len(), T::LEN);

    if bytes.len() != T::LEN {
        return Err(ProgramError::InvalidInstructionData.into());
    }

    try_from_bytes(bytes).map_err(|_| ProgramError::InvalidInstructionData)
}

