use std::path::PathBuf;

use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey,
    pubkey::Pubkey,
    signer::Signer,
};
use solana_state_ext_program::{
    instruction::InitializeMyStateIxDataWithExtensions,
    state::{to_bytes, ExtEnum, MyExt1, MyExt2, MyExt3, MyState},
    ID,
};
use solana_state_extensions::StateExtension;
use solana_transaction::Transaction;

pub const RENT: Pubkey = pubkey!("SysvarRent111111111111111111111111111111111");

pub const PROGRAM: Pubkey = Pubkey::new_from_array(ID);

fn read_program() -> Vec<u8> {
    let mut so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    so_path.push("target/deploy/solana_state_ext_program.so");
    std::fs::read(so_path).unwrap()
}

#[test]
fn test_program_with_state_ext_lifecycle() {
    let mut svm = LiteSVM::new().with_blockhash_check(false);
    let payer_kp = Keypair::new();
    let payer = payer_kp.pubkey();
    let program_id = pubkey!("ENrRns55VechXJiq4bMbdx7idzQh7tvaEJoYeWxRNe7Y");

    svm.add_program(program_id, &read_program());
    svm.airdrop(&payer, 1000000000).unwrap();

    let (mystate_pda, bump) =
        Pubkey::find_program_address(&[MyState::SEED.as_bytes(), &payer.to_bytes()], &PROGRAM);

    let ix_accounts = vec![
        AccountMeta::new(payer, true),
        AccountMeta::new(mystate_pda, false),
        AccountMeta::new_readonly(RENT, false),
        AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
    ];

    let ix_data = InitializeMyStateIxDataWithExtensions {
        owner: *payer.as_array(),
        data: [1; 32],
        bump,
    };

    // Ix discriminator = 0
    let mut ser_ix_data = vec![0];

    // Serialize the instruction data
    ser_ix_data.extend_from_slice(unsafe { to_bytes(&ix_data) });

    // Create instruction
    let ixn = Instruction::new_with_bytes(PROGRAM, &ser_ix_data, ix_accounts);

    let txn = Transaction::new_signed_with_payer(
        &[ixn],
        Some(&payer),
        &[payer_kp],
        svm.latest_blockhash(),
    );

    match svm.send_transaction(txn) {
        Ok(res) => {
            println!("SUCCESS:");
            for log in res.logs {
                println!("    {log}")
            }
        }
        Err(e) => {
            println!("ERROR:");
            for log in e.meta.logs {
                println!("    {log}")
            }
        }
    }

    println!();

    if let Some(my_state) = svm.get_account(&mystate_pda) {
        let extensions =
            MyState::get_extension_variants_from_acc_data_uncheked::<ExtEnum>(&my_state.data)
                .unwrap();

        println!("extensions : {:?}", extensions);

        let extension1 =
            MyState::get_extension_from_acc_data_unchecked::<MyExt1>(&my_state.data, ExtEnum::Ext1)
                .unwrap();

        println!("extension 1 : {:?}", extension1);

        let extension2 =
            MyState::get_extension_from_acc_data_unchecked::<MyExt2>(&my_state.data, ExtEnum::Ext2)
                .unwrap();

        println!("extension 2: {:?}", extension2);

        let extension3 =
            MyState::get_extension_from_acc_data_unchecked::<MyExt3>(&my_state.data, ExtEnum::Ext3)
                .unwrap();

        println!("extension 3: {:?}", extension3);
    }
}
