use super::utils::{DataLen, Initialized};
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    ProgramResult,
};

use solana_state_extensions::{Extension, ExtensionEnum, StateExtension};

use crate::{
    error::MyProgramError, instruction::InitializeMyStateIxDataWithExtensions,
    state::try_from_account_info_mut,
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, shank::ShankType)]
pub enum State {
    Uninitialized,
    Initialized,
    Updated,
}

#[repr(C)] //keeps the struct layout the same across different architectures
#[derive(Clone, Copy, Debug, PartialEq, shank::ShankAccount)]
pub struct MyState {
    pub is_initialized: u8,
    pub owner: Pubkey,
    pub state: State,
    pub data: [u8; 32],
    pub update_count: u32,
    pub bump: u8,
}

impl DataLen for MyState {
    const LEN: usize = core::mem::size_of::<MyState>();
}

impl Initialized for MyState {
    fn is_initialized(&self) -> bool {
        self.is_initialized > 0
    }
}

impl MyState {
    pub const SEED: &'static str = "mystate";

    pub fn validate_pda(bump: u8, pda: &Pubkey, owner: &Pubkey) -> Result<(), ProgramError> {
        let seed_with_bump = &[Self::SEED.as_bytes(), owner, &[bump]];
        let derived = pubkey::create_program_address(seed_with_bump, &crate::ID)?;
        if derived != *pda {
            return Err(MyProgramError::PdaMismatch.into());
        }
        Ok(())
    }

    pub fn initialize_with_extensions(
        my_stata_acc: &AccountInfo,
        fee_payer: &AccountInfo,
        rent: &AccountInfo,
        ix_data: &InitializeMyStateIxDataWithExtensions,
    ) -> ProgramResult {
        {
            let my_state = unsafe { try_from_account_info_mut::<MyState>(my_stata_acc) }?;

            my_state.owner = ix_data.owner;
            my_state.state = State::Initialized;
            my_state.data = ix_data.data;
            my_state.update_count = 0;
            my_state.bump = ix_data.bump;
            my_state.is_initialized = 1;
        }

        unsafe {
            MyState::add_extension(
                my_stata_acc,
                fee_payer,
                rent,
                &MyExt1 {
                    id: 255,
                    data: [4; 32],
                },
            )?
        };

        unsafe {
            MyState::add_extension(
                my_stata_acc,
                fee_payer,
                rent,
                &MyExt2 {
                    id: 10,
                    check: true,
                    owner: Pubkey::default(),
                    data: [9; 32],
                },
            )?
        };

        unsafe {
            MyState::add_extension(
                my_stata_acc,
                fee_payer,
                rent,
                &MyExt3 {
                    id: 50,
                    payer: Pubkey::default(),
                    authority: Pubkey::default(),
                    data: [9; 32],
                },
            )?
        };

        unsafe {
            MyState::update_extension(
                my_stata_acc,
                ExtEnum::Ext1,
                &MyExt1 {
                    id: 1,
                    data: [7; 32],
                },
            )?;
        }

        unsafe {
            MyState::zero_out_extension_data::<MyExt1>(my_stata_acc, ExtEnum::Ext1)?;
        }

        Ok(())
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct MyExt1 {
    id: u8,
    data: [u8; 32],
}

#[repr(C)]
#[derive(Debug)]
pub struct MyExt2 {
    id: u8,
    data: [u8; 32],
    owner: Pubkey,
    check: bool,
}

#[repr(C)]
#[derive(Debug)]
pub struct MyExt3 {
    id: u8,
    data: [u8; 32],
    payer: Pubkey,
    authority: Pubkey,
}

impl StateExtension for MyState {
    const OWNER_PROGRAM: Pubkey = crate::ID;

    const MAX_EXTENSIONS: u8 = 5;

    const EXT_START_MARKER: [u8; 8] = [167, 97, 34, 56, 78, 90, 102, 46];

    const BASE_STATE_LEN: usize = 76;
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtEnum {
    Ext1,
    Ext2,
    Ext3,
}

impl ExtensionEnum for ExtEnum {
    fn from_u8(ext_type: u8) -> Option<Self> {
        match ext_type {
            0 => Some(ExtEnum::Ext1),
            1 => Some(ExtEnum::Ext2),
            2 => Some(ExtEnum::Ext3),
            _ => None,
        }
    }

    fn as_u8(&self) -> u8 {
        match self {
            ExtEnum::Ext1 => 0,
            ExtEnum::Ext2 => 1,
            ExtEnum::Ext3 => 2,
        }
    }
}

impl Extension for MyExt1 {
    const LEN: u16 = 33;

    type ExtensionEnum = ExtEnum;

    fn ext_type() -> u8 {
        ExtEnum::Ext1 as u8
    }
}

impl Extension for MyExt2 {
    const LEN: u16 = 66;

    type ExtensionEnum = ExtEnum;
    fn ext_type() -> u8 {
        ExtEnum::Ext2 as u8
    }
}

impl Extension for MyExt3 {
    const LEN: u16 = 97;

    type ExtensionEnum = ExtEnum;

    fn ext_type() -> u8 {
        ExtEnum::Ext3 as u8
    }
}
