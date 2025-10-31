use crate::helpers::DataLen;
use bytemuck::{Pod, Zeroable};
use pinocchio::pubkey::Pubkey;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct ContributorData {
    pub amount: [u8; 8],
}

impl ContributorData {
    pub fn set_amount(&mut self, amount: u64) {
        self.amount = amount.to_le_bytes();
    }

    pub fn amount(&self) -> u64 {
        u64::from_le_bytes(self.amount)
    }
}

impl DataLen for ContributorData {
    const LEN: usize = core::mem::size_of::<ContributorData>();
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct ContributeIxData {
    pub amount: [u8; 8],
    pub c_bump: [u8; 1],
    pub f_bump: [u8; 1],
    _padding: [u8; 6],
}

impl DataLen for ContributeIxData {
    const LEN: usize = core::mem::size_of::<ContributeIxData>();
}

impl ContributeIxData {
    pub fn amount(&self) -> u64 {
        u64::from_le_bytes(self.amount)
    }

    pub fn c_bump(&self) -> u8 {
        u8::from_le_bytes(self.c_bump)
    }

    pub fn f_bump(&self) -> u8 {
        u8::from_le_bytes(self.f_bump)
    }

    pub fn add_padding(&mut self) {
        self._padding = [0; 6];
    }
}
