use pinocchio::program_error::ProgramError;

pub mod initialize_with_ext_life_cycle;

pub use initialize_with_ext_life_cycle::*;

#[repr(u8)]
pub enum MyProgramInstruction {
    InitializeState,
}

impl TryFrom<&u8> for MyProgramInstruction {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(MyProgramInstruction::InitializeState),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

mod idl_gen {
    use super::InitializeMyStateIxDataWithExtensions;

    #[derive(shank::ShankInstruction)]
    enum _MyProgramInstruction {
        #[account(0, writable, signer, name = "payer_acc", desc = "Fee payer account")]
        #[account(1, writable, name = "state_acc", desc = "New State account")]
        #[account(2, name = "sysvar_rent_acc", desc = "Sysvar rent account")]
        #[account(3, name = "system_program_acc", desc = "System program account")]
        InitializeState(InitializeMyStateIxDataWithExtensions),
    }
}
