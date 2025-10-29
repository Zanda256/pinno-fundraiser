use crate::helpers::DataLen;
use bytemuck::{Pod, Zeroable};
use pinocchio::pubkey::Pubkey;
use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct FundraiserData {
    pub maker: [u8; 32],
    pub mint_to_raise: [u8; 32],
    pub amount_to_raise: [u8; 8],
    pub current_amount: [u8; 8],
    pub time_started: [u8; 8],
    pub duration: [u8; 1],
    pub bump: [u8; 1],
    _padding: [u8; 7],
}

impl FundraiserData {
    pub fn set_maker(&mut self, maker: &Pubkey) {
        self.maker.copy_from_slice(maker.as_ref());
    }

    pub fn maker(&self) -> Pubkey {
        Pubkey::from(self.maker)
    }

    pub fn set_mint_to_raise(&mut self, mint: &Pubkey) {
        self.mint_to_raise.copy_from_slice(mint.as_ref());
    }

    pub fn mint_to_raise(&self) -> Pubkey {
        Pubkey::from(self.mint_to_raise)
    }

    pub fn set_amount_to_raise(&mut self, amount: u64) {
        self.amount_to_raise = amount.to_le_bytes();
    }

    pub fn amount_to_raise(&self) -> u64 {
        u64::from_le_bytes(self.amount_to_raise)
    }

    pub fn set_current_amount(&mut self, amount: u64) {
        self.current_amount = amount.to_le_bytes();
    }

    pub fn current_amount(&self) -> u64 {
        u64::from_le_bytes(self.current_amount)
    }

    pub fn set_time_started(&mut self, amount: u64) {
        self.time_started = amount.to_le_bytes();
    }

    pub fn time_started(&self) -> u64 {
        u64::from_le_bytes(self.time_started)
    }

    pub fn set_duration(&mut self, days: u8) {
        self.duration = days.to_le_bytes();
    }

    pub fn duration(&self) -> u8 {
        u8::from_le_bytes(self.duration)
    }

    pub fn set_bump(&mut self, bump: u8) {
        self.bump = bump.to_le_bytes();
    }

    pub fn bump(&self) -> u8 {
        u8::from_le_bytes(self.bump)
    }

    pub fn add_padding(&mut self) {
        self._padding = [0; 7];
    }
}

impl DataLen for FundraiserData {
    const LEN: usize = core::mem::size_of::<FundraiserData>();
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct InitializeFundraiserIxData {
    pub amount_to_raise: [u8; 8],
    pub duration: [u8; 1],
    pub bump: [u8; 1],
    _padding: [u8; 6],
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

    pub fn duration(&self) -> u8 {
        u8::from_le_bytes(self.duration)
    }

    pub fn set_duration(&mut self, amount: u8) {
        self.duration = amount.to_le_bytes();
    }

    pub fn set_padding(&mut self) {
        self._padding = [0u8; 6];
    }
}
