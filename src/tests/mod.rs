#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use litesvm::LiteSVM;
    use litesvm_token::{
        CreateAssociatedTokenAccount, CreateMint, MintTo,
        spl_token::{
            self,
            solana_program::{msg, rent::Rent, sysvar::SysvarId},
        },
    };
    use solana_account::ReadableAccount;

    use crate::helpers::*;
    use crate::instructions;
    use crate::state::*;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_sdk_ids::sysvar::rent;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    use spl_associated_token_account::solana_program::program_pack::Pack;

    const PROGRAM_ID: &str = "HAV1KKoQW1ckwgvUP8fCXRfjZ4gGfHeu7VhfMej8Bw8i";
    const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
    const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

    fn program_id() -> Pubkey {
        Pubkey::from(crate::ID)
    }

    fn setup() -> (LiteSVM, Keypair) {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();

        svm.airdrop(&payer.pubkey(), 20 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        // Load program SO file
        //msg!("The path is!! {}", env!("CARGO_MANIFEST_DIR"));
        // let project_path = env!("CARGO_MANIFEST_DIR").to_owned();
        // let so_path = PathBuf::from(project_path.push_str("/target/sbf-solana-solana/release/escrow.so") );

        let so_path = PathBuf::from(
            "/Users/hamzahussein/Developer/solana/pinno-fundraiser/target/deploy/pinno_fundraiser.so",
        );
        //  let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        //      .join("../../target/deploy/escrow.so");

        msg!("The path is!! {:?}", so_path);

        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

        svm.add_program(program_id(), &program_data);

        (svm, payer)
    }

    #[test]
    pub fn test_initialize_instruction() {
        let (mut svm, payer) = setup();

        let program_id = program_id();

        assert_eq!(program_id.to_string(), PROGRAM_ID);

        let mint = CreateMint::new(&mut svm, &payer)
            .decimals(6)
            .authority(&payer.pubkey())
            .send()
            .unwrap();
        msg!("Mint A: {}", mint);

        // Derive the PDA for the fundraiser account using the maker's public key and a seed value
        let fundraiser_pda = Pubkey::find_program_address(
            &[b"fundraiser".as_ref(), payer.pubkey().as_ref()],
            &PROGRAM_ID.parse().unwrap(),
        );
        msg!("Fundraiser PDA: {}\n", fundraiser_pda.0);

        // Derive the PDA for the vault associated token account using the fundraiser_pda and Mint
        let vault = spl_associated_token_account::get_associated_token_address(
            &fundraiser_pda.0, // owner will be the fundraiser_pda
            &mint,             // mint
        );
        msg!("Vault PDA: {}\n", vault);

        // Define program IDs for associated token program, token program, and system program
        let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = solana_sdk_ids::system_program::ID;
        let rent_sys_var = rent::ID;

        let amount_to_raise: u64 = 500000000; // 500 tokens with 6 decimal places
        let f_bump: u8 = fundraiser_pda.1;
        let duration: u8 = 7; // 1 week
        let padding: Vec<u8> = vec![0; 6];

        // Create the "Initialize" instruction to initialize the fundraiser
        let init_data = [
            vec![0u8], // Discriminator for "Initialize" instruction
            amount_to_raise.to_le_bytes().to_vec(),
            duration.to_le_bytes().to_vec(),
            f_bump.to_le_bytes().to_vec(),
            padding,
        ]
        .concat();

        let init_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true), // maker - signer only
                AccountMeta::new_readonly(mint, false), // mint
                AccountMeta::new(fundraiser_pda.0, false), // escrow - writable
                AccountMeta::new(vault, false),         // vault - writable
                AccountMeta::new_readonly(system_program, false), // system_program
                AccountMeta::new_readonly(token_program, false), // token_program
                AccountMeta::new_readonly(associated_token_program, false),
                AccountMeta::new_readonly(Rent::id(), false),
            ],

            data: init_data,
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[init_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let res = svm.send_transaction(transaction);

        let mut ok = false;
        match res {
            Ok(tx) => {
                // Log transaction details
                msg!("\n\ntest_initialize transaction successful");
                msg!("CUs Consumed: {}", tx.compute_units_consumed);
                msg!("Tx Signature: {}", tx.signature);
                msg!("Tx Logs: {:?}", tx.logs);
                ok = true;
            }

            Err(err) => {
                msg!("\n\ntest_initialize transaction failed with {:?}", err);
            }
        }

        assert!(ok);
    }

    #[test]
    pub fn test_contribute_instruction() {
        let (mut svm, payer) = setup();

        let program_id = program_id();

        assert_eq!(program_id.to_string(), PROGRAM_ID);

        let mint = CreateMint::new(&mut svm, &payer)
            .decimals(6)
            .authority(&payer.pubkey())
            .send()
            .unwrap();

        // Derive the PDA for the fundraiser account using the maker's public key and a seed value
        let fundraiser_pda = Pubkey::find_program_address(
            &[b"fundraiser".as_ref(), payer.pubkey().as_ref()],
            &PROGRAM_ID.parse().unwrap(),
        );

        // Derive the PDA for the vault associated token account using the fundraiser_pda and Mint
        let vault = spl_associated_token_account::get_associated_token_address(
            &fundraiser_pda.0, // owner will be the fundraiser_pda
            &mint,             // mint
        );

        // Define program IDs for associated token program, token program, and system program
        let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = solana_sdk_ids::system_program::ID;
        let rent_sys_var = rent::ID;

        let amount_to_raise: u64 = 500000000; // 500 tokens with 6 decimal places
        let f_bump: u8 = fundraiser_pda.1;
        let duration: u8 = 7; // 1 week
        let padding: Vec<u8> = vec![0; 6];
        {
            // Create the "Initialize" instruction to initialize the fundraiser
            let init_data = [
                vec![0u8], // Discriminator for "Initialize" instruction
                amount_to_raise.to_le_bytes().to_vec(),
                duration.to_le_bytes().to_vec(),
                f_bump.to_le_bytes().to_vec(),
                padding,
            ]
            .concat();

            let init_ix = Instruction {
                program_id: program_id,
                accounts: vec![
                    AccountMeta::new(payer.pubkey(), true), // maker - signer only
                    AccountMeta::new_readonly(mint, false), // mint
                    AccountMeta::new(fundraiser_pda.0, false), // escrow - writable
                    AccountMeta::new(vault, false),         // vault - writable
                    AccountMeta::new_readonly(system_program, false), // system_program
                    AccountMeta::new_readonly(token_program, false), // token_program
                    AccountMeta::new_readonly(associated_token_program, false),
                    AccountMeta::new_readonly(Rent::id(), false),
                ],

                data: init_data,
            };

            // Create and send the transaction containing the "Make" instruction
            let message = Message::new(&[init_ix], Some(&payer.pubkey()));
            let recent_blockhash = svm.latest_blockhash();

            let transaction = Transaction::new(&[&payer], message, recent_blockhash);

            // Send the transaction and capture the result
            let res = svm.send_transaction(transaction);

            let mut ok = false;
            match res {
                Ok(tx) => {
                    // Log transaction details
                    msg!("\n\ntest_contribute: initialize transaction successful");
                    msg!("CUs Consumed: {}", tx.compute_units_consumed);
                    msg!("Tx Signature: {}", tx.signature);
                    msg!("Tx Logs: {:?}", tx.logs);
                    ok = true;
                }

                Err(err) => {
                    msg!("\n\ntest_contribute: transaction failed with {:?}", err);
                }
            }

            assert!(ok);
        }

        //----------------------------------------------------------------------------------------
        let contributor = Keypair::new();
        svm.airdrop(&contributor.pubkey(), 2 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        let contibutor_PDA = Pubkey::find_program_address(
            &[
                b"contributor".as_ref(),
                fundraiser_pda.0.as_ref(),
                contributor.pubkey().as_ref(),
            ],
            &PROGRAM_ID.parse().unwrap(),
        );

        let contributor_ata = {
            spl_associated_token_account::get_associated_token_address(
                &contributor.pubkey(), // owner will be the contributor
                &mint,                 // mint
            )
        };

        let mut create_contributor_ata_ix =
            CreateAssociatedTokenAccount::new(&mut svm, &contributor, &mint);
        // {
        //     svm:svm,
        //     payer:contributor,
        //     mint: mint,
        //     token_program_id:Some(token_program),
        //     owner: Some(contributor.pubkey())
        // };

        let c_pubkey = &contributor.pubkey();
        create_contributor_ata_ix = create_contributor_ata_ix.owner(&c_pubkey);
        create_contributor_ata_ix = create_contributor_ata_ix.token_program_id(&token_program);
        let _sig = create_contributor_ata_ix.send();

        let mut mint_to_ix = MintTo::new(
            &mut svm,
            &payer,
            &mint,
            &contributor_ata,
            100 * 10_u64.pow(6u32),
        );
        mint_to_ix = mint_to_ix.owner(&payer);
        let _sig = mint_to_ix.send().unwrap();

        let amount_to_contribute: u64 = 4 * 10_u64.pow(6u32);
        // let amount_to_contribute: u64 = 2000000; //000000; // 2 tokens with 6 decimal places

        let c_bump: u8 = contibutor_PDA.1;
        let padding: Vec<u8> = vec![0; 6];
        let contribute_ix_discriminator: u8 = 1;

        let contribute_data = [
            vec![contribute_ix_discriminator], // Discriminator for "Initialize" instruction
            amount_to_contribute.to_le_bytes().to_vec(),
            c_bump.to_le_bytes().to_vec(),
            f_bump.to_be_bytes().to_vec(),
            padding,
        ]
        .concat();

        println!("contributor {}", contributor.pubkey());
        println!("mint_to_raise {}", mint);
        println!("fundraiser {}", fundraiser_pda.0);
        println!("contributor_account {}", contibutor_PDA.0);
        println!("contributor_ata {}", contributor_ata);
        println!("vault {}", vault);
        println!("token_program {}", token_program);

        let contribute_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(contributor.pubkey(), true), // maker - signer only
                AccountMeta::new(mint, false),                // mint
                AccountMeta::new(fundraiser_pda.0, false),    // fundraiser - writable
                AccountMeta::new(contibutor_PDA.0, false),
                AccountMeta::new(contributor_ata, false),
                AccountMeta::new(vault, false), // vault - writable
                AccountMeta::new(token_program, false), // token_program
                AccountMeta::new(system_program, false), // system_program
                AccountMeta::new(associated_token_program, false),
                AccountMeta::new(Rent::id(), false),
            ],

            data: contribute_data,
        };

        // Create and send the transaction containing the "Contibute" instruction
        let message1 = Message::new(&[contribute_ix], Some(&contributor.pubkey()));
        let recent_blockhash1 = svm.latest_blockhash();

        let transaction1 = Transaction::new(&[&contributor], message1, recent_blockhash1);

        // Send the transaction and capture the result
        let res1 = svm.send_transaction(transaction1);

        let mut ok1 = false;
        match res1 {
            Ok(tx) => {
                // Log transaction details
                msg!("\n\ncontribute transaction successful");
                msg!("CUs Consumed: {}", tx.compute_units_consumed);
                msg!("Tx Signature: {}", tx.signature);
                msg!("Tx Logs: {:?}", tx.logs);
                ok1 = true;
            }

            Err(err) => {
                msg!("\n\ntest_contribute transaction failed with {:?}", err);
            }
        }

        assert!(ok1);

        let vault_account = svm.get_account(&vault).expect("Vault ATA should exist");
        let vault_data = spl_token::state::Account::unpack(&vault_account.data).unwrap();

        msg!("\n\nvault token amount: {:?}", vault_data.amount);
        assert_eq!(vault_data.amount, amount_to_contribute);

        // Check balance in contributor PDA is correct
        let contributor_account = svm
            .get_account(&contibutor_PDA.0)
            .expect("Contributor PDA account should exist");

        let mut c_pda_amount: u64 = 0;
        let amt_bytes = contributor_account.data.clone();
        let amt_arr: [u8; 8] = amt_bytes.try_into().unwrap();
        c_pda_amount = u64::from_le_bytes(amt_arr);

        assert_eq!(c_pda_amount, amount_to_contribute);

        // Check balance in fundraiser PDA is correct
        let fundraiser_account = svm
            .get_account(&fundraiser_pda.0)
            .expect("Fundraiser PDA account should exist");

        let mut f_pda_amount: u64 = 0;
        let mut buf: [u8; 8] = [0; 8];
        buf.copy_from_slice(&fundraiser_account.data[72..80]);
        f_pda_amount = u64::from_le_bytes(buf);

        assert_eq!(f_pda_amount, amount_to_contribute);
    }

    #[test]
    pub fn test_refund_instruction() {
        let (mut svm, payer) = setup();

        let program_id = program_id();

        assert_eq!(program_id.to_string(), PROGRAM_ID);

        let mint = CreateMint::new(&mut svm, &payer)
            .decimals(6)
            .authority(&payer.pubkey())
            .send()
            .unwrap();

        // Derive the PDA for the fundraiser account using the maker's public key and a seed value
        let fundraiser_pda = Pubkey::find_program_address(
            &[b"fundraiser".as_ref(), payer.pubkey().as_ref()],
            &PROGRAM_ID.parse().unwrap(),
        );

        // Derive the PDA for the vault associated token account using the fundraiser_pda and Mint
        let vault = spl_associated_token_account::get_associated_token_address(
            &fundraiser_pda.0, // owner will be the fundraiser_pda
            &mint,             // mint
        );

        // Define program IDs for associated token program, token program, and system program
        let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = solana_sdk_ids::system_program::ID;
        let rent_sys_var = rent::ID;

        let amount_to_raise: u64 = 500000000; // 500 tokens with 6 decimal places
        let f_bump: u8 = fundraiser_pda.1;
        let duration: u8 = 7; // 1 week
        let padding: Vec<u8> = vec![0; 6];
        {
            // Create the "Initialize" instruction to initialize the fundraiser
            let init_data = [
                vec![0u8], // Discriminator for "Initialize" instruction
                amount_to_raise.to_le_bytes().to_vec(),
                duration.to_le_bytes().to_vec(),
                f_bump.to_le_bytes().to_vec(),
                padding,
            ]
            .concat();

            let init_ix = Instruction {
                program_id: program_id,
                accounts: vec![
                    AccountMeta::new(payer.pubkey(), true), // maker - signer only
                    AccountMeta::new_readonly(mint, false), // mint
                    AccountMeta::new(fundraiser_pda.0, false), // escrow - writable
                    AccountMeta::new(vault, false),         // vault - writable
                    AccountMeta::new_readonly(system_program, false), // system_program
                    AccountMeta::new_readonly(token_program, false), // token_program
                    AccountMeta::new_readonly(associated_token_program, false),
                    AccountMeta::new_readonly(Rent::id(), false),
                ],

                data: init_data,
            };

            // Create and send the transaction containing the "Make" instruction
            let message = Message::new(&[init_ix], Some(&payer.pubkey()));
            let recent_blockhash = svm.latest_blockhash();

            let transaction = Transaction::new(&[&payer], message, recent_blockhash);

            // Send the transaction and capture the result
            let res = svm.send_transaction(transaction);

            let mut ok = false;
            match res {
                Ok(tx) => {
                    // Log transaction details
                    msg!("\n\ntest_contribute: initialize transaction successful");
                    msg!("CUs Consumed: {}", tx.compute_units_consumed);
                    msg!("Tx Signature: {}", tx.signature);
                    msg!("Tx Logs: {:?}", tx.logs);
                    ok = true;
                }

                Err(err) => {
                    msg!("\n\ntest_contribute: transaction failed with {:?}", err);
                }
            }

            assert!(ok);
        }

        //----------------------------------------------------------------------------------------
        let contributor = Keypair::new();
        svm.airdrop(&contributor.pubkey(), 2 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        let contibutor_PDA = Pubkey::find_program_address(
            &[
                b"contributor".as_ref(),
                fundraiser_pda.0.as_ref(),
                contributor.pubkey().as_ref(),
            ],
            &PROGRAM_ID.parse().unwrap(),
        );

        let contributor_ata = {
            spl_associated_token_account::get_associated_token_address(
                &contributor.pubkey(), // owner will be the contributor
                &mint,                 // mint
            )
        };

        let mut create_contributor_ata_ix =
            CreateAssociatedTokenAccount::new(&mut svm, &contributor, &mint);
        // {
        //     svm:svm,
        //     payer:contributor,
        //     mint: mint,
        //     token_program_id:Some(token_program),
        //     owner: Some(contributor.pubkey())
        // };

        let c_pubkey = &contributor.pubkey();
        create_contributor_ata_ix = create_contributor_ata_ix.owner(&c_pubkey);
        create_contributor_ata_ix = create_contributor_ata_ix.token_program_id(&token_program);
        let _sig = create_contributor_ata_ix.send();

        let mut mint_to_ix = MintTo::new(&mut svm, &payer, &mint, &contributor_ata, 100);
        mint_to_ix = mint_to_ix.owner(&payer);
        let _sig = mint_to_ix.send().unwrap();

        let amount_to_contribute: u64 = 12; //00000; // 500 tokens with 6 decimal places
        let c_bump: u8 = contibutor_PDA.1;
        let padding: Vec<u8> = vec![0; 6];
        let contribute_ix_discriminator: u8 = 1;

        let contribute_data = [
            vec![contribute_ix_discriminator], // Discriminator for "Initialize" instruction
            amount_to_contribute.to_le_bytes().to_vec(),
            c_bump.to_le_bytes().to_vec(),
            f_bump.to_be_bytes().to_vec(),
            padding,
        ]
        .concat();

        println!("contributor {}", contributor.pubkey());
        println!("mint_to_raise {}", mint);
        println!("fundraiser {}", fundraiser_pda.0);
        println!("contributor_account {}", contibutor_PDA.0);
        println!("contributor_ata {}", contributor_ata);
        println!("vault {}", vault);
        println!("token_program {}", token_program);

        let contribute_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(contributor.pubkey(), true), // maker - signer only
                AccountMeta::new(mint, false),                // mint
                AccountMeta::new(fundraiser_pda.0, false),    // fundraiser - writable
                AccountMeta::new(contibutor_PDA.0, false),
                AccountMeta::new(contributor_ata, false),
                AccountMeta::new(vault, false), // vault - writable
                AccountMeta::new(token_program, false), // token_program
                AccountMeta::new(system_program, false), // system_program
                AccountMeta::new(associated_token_program, false),
                AccountMeta::new(Rent::id(), false),
            ],

            data: contribute_data,
        };

        // Create and send the transaction containing the "Contibute" instruction
        let message1 = Message::new(&[contribute_ix], Some(&contributor.pubkey()));
        let recent_blockhash1 = svm.latest_blockhash();

        let transaction1 = Transaction::new(&[&contributor], message1, recent_blockhash1);

        // Send the transaction and capture the result
        let res1 = svm.send_transaction(transaction1);

        let mut ok1 = false;
        match res1 {
            Ok(tx) => {
                // Log transaction details
                msg!("\n\ncontribute transaction successful");
                msg!("CUs Consumed: {}", tx.compute_units_consumed);
                msg!("Tx Signature: {}", tx.signature);
                msg!("Tx Logs: {:?}", tx.logs);
                ok1 = true;
            }

            Err(err) => {
                msg!("\n\ntest_contribute transaction failed with {:?}", err);
            }
        }

        assert!(ok1);

        //------------------------------------------------------------------------------------
        let refund_ix_discriminator: u8 = 2;
        let refund_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(contributor.pubkey(), true), // contributor - signer only
                AccountMeta::new(payer.pubkey(), false),
                AccountMeta::new(mint, false),             // mint
                AccountMeta::new(fundraiser_pda.0, false), // fundraiser - writable
                AccountMeta::new(contibutor_PDA.0, false),
                AccountMeta::new(contributor_ata, false),
                AccountMeta::new(vault, false), // vault - writable
                AccountMeta::new(token_program, false), // token_program
                AccountMeta::new(system_program, false), // system_program
                AccountMeta::new(associated_token_program, false),
                AccountMeta::new(Rent::id(), false),
            ],

            data: vec![refund_ix_discriminator],
        };

        // Create and send the transaction containing the "Contibute" instruction
        let message2 = Message::new(&[refund_ix], Some(&contributor.pubkey()));
        let recent_blockhash2 = svm.latest_blockhash();

        let transaction2 = Transaction::new(&[&contributor], message2, recent_blockhash2);

        // Send the transaction and capture the result
        let res2 = svm.send_transaction(transaction2);

        let mut ok2 = false;
        match res2 {
            Ok(tx) => {
                // Log transaction details
                msg!("\n\nrefund transaction successful");
                msg!("CUs Consumed: {}", tx.compute_units_consumed);
                msg!("Tx Signature: {}", tx.signature);
                msg!("Tx Logs: {:?}", tx.logs);
                ok2 = true;
            }

            Err(err) => {
                msg!("\n\ntest_refund transaction failed with {:?}", err);
            }
        }

        assert!(ok2);
    }

    #[test]
    pub fn test_check_contribution_instruction() {
        let (mut svm, payer) = setup();

        let program_id = program_id();

        assert_eq!(program_id.to_string(), PROGRAM_ID);

        let mint = CreateMint::new(&mut svm, &payer)
            .decimals(6)
            .authority(&payer.pubkey())
            .send()
            .unwrap();

        // Derive the PDA for the fundraiser account using the maker's public key and a seed value
        let fundraiser_pda = Pubkey::find_program_address(
            &[b"fundraiser".as_ref(), payer.pubkey().as_ref()],
            &PROGRAM_ID.parse().unwrap(),
        );

        // Derive the PDA for the vault associated token account using the fundraiser_pda and Mint
        let vault = spl_associated_token_account::get_associated_token_address(
            &fundraiser_pda.0, // owner will be the fundraiser_pda
            &mint,             // mint
        );

        // Define program IDs for associated token program, token program, and system program
        let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = solana_sdk_ids::system_program::ID;
        let rent_sys_var = rent::ID;

        let amount_to_raise: u64 = 500000000; // 500 tokens with 6 decimal places
        let f_bump: u8 = fundraiser_pda.1;
        let duration: u8 = 7; // 1 week
        let padding: Vec<u8> = vec![0; 6];
        {
            // Create the "Initialize" instruction to initialize the fundraiser
            let init_data = [
                vec![0u8], // Discriminator for "Initialize" instruction
                amount_to_raise.to_le_bytes().to_vec(),
                duration.to_le_bytes().to_vec(),
                f_bump.to_le_bytes().to_vec(),
                padding,
            ]
            .concat();

            let init_ix = Instruction {
                program_id: program_id,
                accounts: vec![
                    AccountMeta::new(payer.pubkey(), true), // maker - signer only
                    AccountMeta::new_readonly(mint, false), // mint
                    AccountMeta::new(fundraiser_pda.0.clone(), false), // escrow - writable
                    AccountMeta::new(vault, false),         // vault - writable
                    AccountMeta::new_readonly(system_program, false), // system_program
                    AccountMeta::new_readonly(token_program, false), // token_program
                    AccountMeta::new_readonly(associated_token_program, false),
                    AccountMeta::new_readonly(Rent::id(), false),
                ],

                data: init_data,
            };

            // Create and send the transaction containing the "Make" instruction
            let message = Message::new(&[init_ix], Some(&payer.pubkey()));
            let recent_blockhash = svm.latest_blockhash();

            let transaction = Transaction::new(&[&payer], message, recent_blockhash);

            // Send the transaction and capture the result
            let res = svm.send_transaction(transaction);

            let mut ok = false;
            match res {
                Ok(tx) => {
                    // Log transaction details
                    msg!("\n\ntest_contribute: initialize transaction successful");
                    msg!("CUs Consumed: {}", tx.compute_units_consumed);
                    msg!("Tx Signature: {}", tx.signature);
                    msg!("Tx Logs: {:?}", tx.logs);
                    ok = true;
                }

                Err(err) => {
                    msg!("\n\ntest_contribute: transaction failed with {:?}", err);
                }
            }

            assert!(ok);
        }

        //----------------------------------------------------------------------------------------
        for i in 0..=9 {
            let contributor = Keypair::new();
            svm.airdrop(&contributor.pubkey(), 2 * LAMPORTS_PER_SOL)
                .expect("Airdrop failed");

            let contibutor_PDA = Pubkey::find_program_address(
                &[
                    b"contributor".as_ref(),
                    fundraiser_pda.0.as_ref(),
                    contributor.pubkey().as_ref(),
                ],
                &PROGRAM_ID.parse().unwrap(),
            );

            let contributor_ata = {
                spl_associated_token_account::get_associated_token_address(
                    &contributor.pubkey(), // owner will be the contributor
                    &mint,                 // mint
                )
            };

            let mut create_contributor_ata_ix =
                CreateAssociatedTokenAccount::new(&mut svm, &contributor, &mint);

            let c_pubkey = &contributor.pubkey();
            create_contributor_ata_ix = create_contributor_ata_ix.owner(&c_pubkey);
            create_contributor_ata_ix = create_contributor_ata_ix.token_program_id(&token_program);
            let _sig = create_contributor_ata_ix.send().unwrap();

            let mut mint_to_ix = MintTo::new(&mut svm, &payer, &mint, &contributor_ata, 500000000);
            mint_to_ix = mint_to_ix.owner(&payer);
            let _sig = mint_to_ix.send().unwrap();

            let amount_to_contribute: u64 = 50000000; //00000; // 500 tokens with 6 decimal places
            let c_bump: u8 = contibutor_PDA.1;
            let padding: Vec<u8> = vec![0; 6];
            let contribute_ix_discriminator: u8 = 1;

            let contribute_data = [
                vec![contribute_ix_discriminator], // Discriminator for "Initialize" instruction
                amount_to_contribute.to_le_bytes().to_vec(),
                c_bump.to_le_bytes().to_vec(),
                f_bump.to_be_bytes().to_vec(),
                padding,
            ]
            .concat();

            if i == 0 {
                println!("mint_to_raise {}", mint);
                println!("fundraiser {}", fundraiser_pda.0);
                println!("vault {}", vault);
                println!("token_program {}", token_program);
            }
            println!("-------------------------------------------------------");
            println!("contributor {}", contributor.pubkey());
            println!("contributor_account {}", contibutor_PDA.0);
            println!("contributor_ata {}", contributor_ata);
            println!("-------------------------------------------------------");

            let contribute_ix = Instruction {
                program_id: program_id,
                accounts: vec![
                    AccountMeta::new(contributor.pubkey(), true), // maker - signer only
                    AccountMeta::new(mint, false),                // mint
                    AccountMeta::new(fundraiser_pda.0.clone(), false), // fundraiser - writable
                    AccountMeta::new(contibutor_PDA.0, false),
                    AccountMeta::new(contributor_ata, false),
                    AccountMeta::new(vault, false), // vault - writable
                    AccountMeta::new(token_program, false), // token_program
                    AccountMeta::new(system_program, false), // system_program
                    AccountMeta::new(associated_token_program, false),
                    AccountMeta::new(Rent::id(), false),
                ],

                data: contribute_data,
            };

            // Create and send the transaction containing the "Contibute" instruction
            let message1 = Message::new(&[contribute_ix], Some(&contributor.pubkey()));
            let recent_blockhash1 = svm.latest_blockhash();

            let transaction1 = Transaction::new(&[&contributor], message1, recent_blockhash1);

            // Send the transaction and capture the result
            let res1 = svm.send_transaction(transaction1);

            let mut ok1 = false;
            match res1 {
                Ok(tx) => {
                    // Log transaction details
                    msg!("\n\ncontribute transaction successful");
                    msg!("CUs Consumed: {}", tx.compute_units_consumed);
                    msg!("Tx Signature: {}", tx.signature);
                    msg!("Tx Logs: {:?}", tx.logs);
                    ok1 = true;
                }

                Err(err) => {
                    msg!("\n\ntest_contribute transaction failed with {:?}", err);
                }
            }

            assert!(ok1);
        }

        //----------------------------------------------------------------------------------------

        let maker_ata = {
            spl_associated_token_account::get_associated_token_address(
                &payer.pubkey(), // owner will be the payer
                &mint,           // mint
            )
        };

        let mut create_maker_ata_ix = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint);

        let m_pubkey = &payer.pubkey();
        create_maker_ata_ix = create_maker_ata_ix.owner(&m_pubkey);
        create_maker_ata_ix = create_maker_ata_ix.token_program_id(&token_program);
        let _sig = create_maker_ata_ix.send().unwrap();

        // Create the "Check" instruction to claim raised funds in the fundraiser vault
        let check_data = [
            vec![3u8], // Discriminator for "Check" instruction
        ]
        .concat();

        let check_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true), // maker - signer only
                AccountMeta::new_readonly(mint, false), // mint
                AccountMeta::new(fundraiser_pda.0.clone(), false), // escrow - writable
                AccountMeta::new(vault, false),         // vault - writable
                AccountMeta::new(maker_ata, false),     // maker's ata - writable
                AccountMeta::new_readonly(token_program, false), // token_program
                AccountMeta::new_readonly(system_program, false), // system_program
                AccountMeta::new_readonly(associated_token_program, false),
                AccountMeta::new_readonly(Rent::id(), false),
            ],

            data: check_data,
        };

        // Create and send the transaction containing the "Check" instruction
        let message = Message::new(&[check_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let res2 = svm.send_transaction(transaction);

        let mut ok2 = false;
        match res2 {
            Ok(tx) => {
                // Log transaction details
                msg!("\n\nCheck transaction successful");
                msg!("CUs Consumed: {}", tx.compute_units_consumed);
                msg!("Tx Signature: {}", tx.signature);
                msg!("Tx Logs: {:?}", tx.logs);
                ok2 = true;
            }

            Err(err) => {
                msg!(
                    "\n\ntest_check_contributions transaction failed with {:?}",
                    err
                );
            }
        }

        assert!(ok2);
    }
}
