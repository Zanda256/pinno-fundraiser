use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError
};
use bytemuck::{ Zeroable, Pod};
use pinocchio::pubkey::Pubkey;
use crate::helpers::DataLen;


#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct FundraiserData {
    pub maker: [u8; 32],
    pub mint_to_raise: [u8; 32],
    pub amount_to_raise: [u8; 8],
    pub current_amount: [u8; 8],
    pub time_started: [u8; 8],
    pub duration: [u8; 8],
    pub bump: [u8; 1],
    _padding: [u8; 7],
}


impl DataLen for FundraiserData {
    const LEN: usize = core::mem::size_of::<FundraiserData>();
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct InitializeFundraiserIxData {
    pub amount_to_raise: [u8; 8],
    pub duration: [u8; 8],
    pub bump: [u8; 1],
    _padding: [u8; 7],
}

impl DataLen for InitializeFundraiserIxData {
    const LEN: usize = core::mem::size_of::<InitializeFundraiserIxData>();
}

impl InitializeFundraiserIxData {
    pub fn amount_to_raise(&self) -> u64 {
        u64::from_le_bytes(self.amount_to_raise)
    }

    pub fn set_amount_to_raise(&mut self, amount: u64) {
        self.amount_to_raise = amount.to_le_bytes();
    }

    pub fn duration(&self) -> u64 {
        u64::from_le_bytes(self.duration)
    }

    pub fn set_duration(&mut self, amount: u64) {
        self.amount_to_raise = amount.to_le_bytes();
    }

    pub fn set_padding(&mut self) {
        self._padding = [0u8; 7];
    }
}