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

    use crate::instructions;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_sdk_ids::sysvar::rent;
    use solana_signer::Signer;
    use solana_transaction::Transaction;

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
}
