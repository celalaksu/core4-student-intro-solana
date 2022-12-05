use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar, rent::ID as RENT_PROGRAM_ID},
    program_pack::IsInitialized,
    system_program::ID as SYSTEM_PROGRAM_ID,
    native_token::LAMPORTS_PER_SOL,
};
use borsh::BorshSerialize;
use crate::{error::ReviewError, state::StudentIntroCommentCounter, state::StudentIntroComment};
use crate::instruction::StudentIntroInstruction;
use std::convert::TryInto;
use crate::state::StudentIntroState;
use spl_token::{ instruction::{ initialize_mint, mint_to }, ID as TOKEN_PROGRAM_ID };
use spl_associated_token_account::get_associated_token_address;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = StudentIntroInstruction::unpack(instruction_data)?;

    match instruction {
        StudentIntroInstruction::AddStudentIntro { 
            name, 
            message } => {
            add_student_intro(program_id, accounts, name, message)
        }
        StudentIntroInstruction::UpdateStudentIntro { 
            name, 
            message } => {
            update_student_intro(program_id, accounts, name, message)
        }
        StudentIntroInstruction::AddComment { comment } => {
            add_student_intro_comment(program_id, accounts, comment)
        }
        StudentIntroInstruction::InitializeMint => 
            initialize_token_mint(program_id, accounts),
        
    }
}

pub fn add_student_intro(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    message: String,
) -> ProgramResult {

    msg!("Adding student intro..");
    msg!("Name: {}", name);
    msg!("Message: {}", message);
    
   // Get Account iterator
   let account_info_iter = &mut accounts.iter();

   // Get accounts
   let initializer = next_account_info(account_info_iter)?;
   let pda_account = next_account_info(account_info_iter)?;
   let pda_counter = next_account_info(account_info_iter)?;

    let token_mint = next_account_info(account_info_iter)?;
    let mint_auth = next_account_info(account_info_iter)?;
    let user_ata = next_account_info(account_info_iter)?;

   let system_program = next_account_info(account_info_iter)?;

   let token_program = next_account_info(account_info_iter)?;

   msg!("Deriving mint authority");
    let (mint_pda, _mint_bump) = Pubkey::find_program_address(
        &[b"token_mint"], program_id);
    let (mint_auth_pda, mint_auth_bump) = Pubkey::find_program_address(
        &[b"token_auth"], program_id
    );
    if *mint_auth.key != mint_auth_pda {
        msg!("Mint passed in add mint derived do not match");
        return Err(ReviewError::InvalidPDA.into());
    }
    if *user_ata.key != get_associated_token_address(initializer.key, token_mint.key) {
        msg!("Incorrect token mint");
        return Err(ReviewError::IncorrectAccountError.into());
    }
    if *token_program.key != TOKEN_PROGRAM_ID {
        msg!("Incorrect token program");
        return Err(ReviewError::IncorrectAccountError.into());
    }


    if *token_mint.key != mint_pda {
        msg!("Incorrect token mint");
        return Err(ReviewError::IncorrectAccountError.into());
    }


   if !initializer.is_signer {
    msg!("Missing required signature");
    return Err(ProgramError::MissingRequiredSignature);
    }

   let (pda, bump_seed) = Pubkey::find_program_address(
       &[initializer.key.as_ref(), name.as_bytes().as_ref()],
       program_id,
   );

   if pda != *pda_account.key {
    msg!("Invalid seeds for PDA");
    return Err(ProgramError::InvalidArgument);
   }

   // Calculate account size required
   // let account_len = 1 + (4 + name.len()) + (4 + message.len());
   let account_len: usize = 1000;

   // let total_len: usize = 1 + 1 + (4 + name.len()) + (4 + message.len());
   if StudentIntroState::get_account_size(name.clone(), message.clone()) > 1000 {
       msg!("Data length is larger than 1000 bytes");
       return Err(ReviewError::InvalidDataLength.into());
   }

   // Calculate rent required
   let rent = Rent::get()?;
   let rent_lamports = rent.minimum_balance(account_len);

   // Create the account
   invoke_signed(
       &system_instruction::create_account(
           initializer.key,
           pda_account.key,
           rent_lamports,
           account_len.try_into().unwrap(),
           program_id,
       ),
       &[
           initializer.clone(),
           pda_account.clone(),
           system_program.clone(),
       ],
       &[&[
           initializer.key.as_ref(),
           name.as_bytes().as_ref(),
           &[bump_seed],
       ]],
   )?;

   msg!("PDA created: {}", pda);

   msg!("unpacking state account");
   let mut account_data =
       try_from_slice_unchecked::<StudentIntroState>(&pda_account.data.borrow()).unwrap();
   msg!("borrowed account data");

   account_data.discriminator = StudentIntroState::DISCRIMINATOR.to_string();
   account_data.reviewer = *initializer.key;
   account_data.name = name;
   account_data.message = message;
   account_data.is_initialized = true;

   msg!("serializing account");
   account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;
   msg!("state account serialized");

   msg!("Creating comment counter");
   let rent = Rent::get()?;
   let counter_rent_lamports = rent.minimum_balance(StudentIntroCommentCounter::SIZE);

   let (counter, counter_bump) = Pubkey::find_program_address(
        &[pda.as_ref(), "comment".as_ref()], program_id
   );
   if counter != *pda_counter.key {
    msg!("Invalid seeds for PDA");
    return Err(ProgramError::InvalidArgument);
   }

   invoke_signed(
    &system_instruction::create_account(
        initializer.key,
        pda_counter.key, 
        counter_rent_lamports, 
        StudentIntroCommentCounter::SIZE.try_into().unwrap(), 
        program_id), 
        &[
            initializer.clone(),
            pda_counter.clone(),
            system_program.clone(),
        ], 
    &[&[pda.as_ref(), "comment".as_ref(), &[counter_bump]]],)?;
    
    msg!("Comment counter created");

    let mut counter_data = try_from_slice_unchecked::<StudentIntroCommentCounter>(
        &pda_counter.data.borrow()).unwrap();

    msg!("Checking if counter account is already initialized.");
    if counter_data.is_initialized() {
        msg!("Account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    counter_data.discriminator = StudentIntroCommentCounter::DISCRIMINATOR.to_string();
    counter_data.counter = 0;
    counter_data.is_intialized = true;
    msg!("Comment count: {}", counter_data.counter);
    
    counter_data.serialize(&mut &mut pda_counter.data.borrow_mut()[..])?;

    msg!("Comment counter initialized");

    msg!("Minting 10 tokens to User associated token account");
    invoke_signed(
        // Instruction
        &mint_to(
            token_program.key,
            token_mint.key,
            user_ata.key,
            mint_auth.key,
            &[],
            10*LAMPORTS_PER_SOL,
        )?, // ? unwraps and returns the error if there is one
        // Account_infos
        &[token_mint.clone(), user_ata.clone(), mint_auth.clone()],
        // Seeds
        &[&[b"token_auth", &[mint_auth_bump]]],
    )?;


   Ok(())
}

pub fn update_student_intro(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    message: String,
) -> ProgramResult {
    msg!("Updating student intro...");

    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let pda_account = next_account_info(account_info_iter)?;

    if pda_account.owner != program_id {
        return  Err(ProgramError::IllegalOwner);
    }

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    msg!("Unpacking state student");
    let mut account_data = try_from_slice_unchecked::<StudentIntroState>(&pda_account.data.borrow()).unwrap();
    msg!("borrowed account data");

    let (pda, _bump_seed) = Pubkey::find_program_address(&[
        initializer.key.as_ref(),
        account_data.name.as_bytes().as_ref(),
    ], program_id);

    if pda != *pda_account.key {
        msg!("Invalid seeds for PDA");
        return Err(ReviewError::InvalidPDA.into());
    }

    if !account_data.is_initialized {
        msg!("Account is not initialized");
        return Err(ReviewError::UninitializedAccount.into());
    }

    let total_len: usize = 1 + (4 + account_data.name.len()) + (4 + message.len());
    if total_len > 1000 {
        msg!("Data length is larger than 1000 bytes");
        return Err(ReviewError::InvalidDataLength.into());
    }

    account_data.name = name;
    account_data.message = message;

    account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;

    Ok(())
}

pub fn add_student_intro_comment(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    comment: String
) -> ProgramResult {
    msg!("Adding comment ...");
    msg!("Comment: {}",comment);

    let account_info_iter = &mut accounts.iter();

    let commenter = next_account_info(account_info_iter)?;
    let pda_review = next_account_info(account_info_iter)?;
    let pda_counter = next_account_info(account_info_iter)?;
    let pda_comment = next_account_info(account_info_iter)?;

    let token_mint = next_account_info(account_info_iter)?;
    let mint_auth = next_account_info(account_info_iter)?;
    let user_ata = next_account_info(account_info_iter)?;

    let system_program = next_account_info(account_info_iter)?;

    let token_program = next_account_info(account_info_iter)?;


    let mut counter_data = try_from_slice_unchecked::<StudentIntroCommentCounter>(
        &pda_counter.data.borrow()).unwrap();
    
    let account_len = StudentIntroComment::get_account_size(comment.clone());
    
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_len);

    let (pda, bump_seed) = Pubkey::find_program_address(
        &[pda_review.key.as_ref(),
        counter_data.counter.to_be_bytes().as_ref()],
        program_id
    );
    if pda != *pda_comment.key {
        msg!("Invalid seeds for PDA");
        return Err(ReviewError::InvalidPDA.into());
    }

    invoke_signed(
        &system_instruction::create_account(
        commenter.key, 
        pda_comment.key, 
        rent_lamports, 
        account_len.try_into().unwrap(), 
        program_id,
        ),
        &[commenter.clone(),
        pda_comment.clone(),
        system_program.clone()],
    &[&[pda_review.key.as_ref(),
    counter_data.counter.to_be_bytes().as_ref(),
    &[bump_seed]]],
    )?;

    msg!("Created comment account.");

    let mut comment_data = try_from_slice_unchecked::<StudentIntroComment>(
        &pda_comment.data.borrow()).unwrap();

    msg!("Checking if comment account is already initialized.");
    if comment_data.is_initialized() {
        msg!("Account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    comment_data.discriminator = StudentIntroComment::DISCRIMINATOR.to_string();
    comment_data.review = *pda_review.key;
    comment_data.commenter = *commenter.key;
    comment_data.comment = comment;
    comment_data.is_initialized = true;
    comment_data.serialize(&mut &mut pda_comment.data.borrow_mut()[..])?;

    msg!("Comment count: {}", counter_data.counter);

    counter_data.counter += 1;
    counter_data.serialize(&mut &mut pda_counter.data.borrow_mut()[..])?;

    // Mint tokens here
    msg!("deriving mint authority");
    let (mint_pda, _mint_bump) = Pubkey::find_program_address(&[b"token_mint"], program_id);
    let (mint_auth_pda, mint_auth_bump) =
        Pubkey::find_program_address(&[b"token_auth"], program_id);

    if *token_mint.key != mint_pda {
        msg!("Incorrect token mint");
        return Err(ReviewError::IncorrectAccountError.into());
    }

    if *mint_auth.key != mint_auth_pda {
        msg!("Mint passed in and mint derived do not match");
        return Err(ReviewError::InvalidPDA.into());
    }

    if *user_ata.key != get_associated_token_address(commenter.key, token_mint.key) {
        msg!("Incorrect token mint");
        return Err(ReviewError::IncorrectAccountError.into());
    }

    if *token_program.key != TOKEN_PROGRAM_ID {
        msg!("Incorrect token program");
        return Err(ReviewError::IncorrectAccountError.into());
    }

    msg!("Minting 5 tokens to User associated token account");
    invoke_signed(
        // Instruction
        &mint_to(
            token_program.key,
            token_mint.key,
            user_ata.key,
            mint_auth.key,
            &[],
            5 * LAMPORTS_PER_SOL,
        )?,
        // Account_infos
        &[token_mint.clone(), user_ata.clone(), mint_auth.clone()],
        // Seeds
        &[&[b"token_auth", &[mint_auth_bump]]],
    )?;



    Ok(())
}

pub fn initialize_token_mint(
    program_id: &Pubkey,
    accounts: &[AccountInfo]
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let token_mint = next_account_info(account_info_iter)?;
    let mint_auth = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let sysvar_rent = next_account_info(account_info_iter)?;

    let (mint_pda, mint_bump) = Pubkey::find_program_address(
        &[b"token_mint"], program_id
    );
    
    let (mint_auth_pda, mint_auth_bump) = Pubkey::find_program_address(
        &[b"token_auth"], program_id
    );

    msg!("Token mint: {:?}", mint_pda);
    msg!("Mint authority: {:?}", mint_auth_pda);

    if mint_pda != *token_mint.key {
        msg!("Incorrect token mint account");
        return Err(ReviewError::IncorrectAccountError.into());
    }
    if mint_auth_pda != *mint_auth.key {
        msg!("Incorrect mint auth account");
        return Err(ReviewError::IncorrectAccountError.into());
    }
    if *token_program.key != TOKEN_PROGRAM_ID {
        msg!("Incorrect token program");
        return Err(ReviewError::IncorrectAccountError.into());
    }
    if *system_program.key != SYSTEM_PROGRAM_ID {
        msg!("Incorrect system program");
        return  Err(ReviewError::IncorrectAccountError.into());
    }
    if *sysvar_rent.key != RENT_PROGRAM_ID {
        msg!("Incorrect rent program");
        return Err(ReviewError::IncorrectAccountError.into());
    }

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(82);

    // Create the token mint PDA
    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            token_mint.key,
            rent_lamports,
            82, // Size of the token mint account
            token_program.key,
        ),
        // Accounts we're reading from or writing to
        &[
            initializer.clone(),
            token_mint.clone(),
            system_program.clone(),
        ],
        // Seeds for out token mint account
    &[&[b"token_mint", &[mint_bump]]],
    )?;

    msg!("Created token mint account");

    // Initialize the mint account
    invoke_signed(
        &initialize_mint(
            token_program.key,
            token_mint.key,
            mint_auth.key,
            Option::None, // Freeze authority - we don't want anyone to be able to freeze!
            9, // Number of decimals
        )?,
        // Which accounts we're reading from or writing to
        &[token_mint.clone(), sysvar_rent.clone(), mint_auth.clone()],
        // The seeds for out token mint PDA
        &[&[b"token_mint", &[mint_bump]]],
    )?;   

    Ok(())
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        assert_matches::*,
        solana_program::{
            instruction::{AccountMeta, Instruction},
            system_program::ID as SYSTEM_PROGRAM_ID,
        },
        solana_program_test::*,
        solana_sdk::{
            signature::Signer,
            transaction::Transaction,
            sysvar::rent::ID as SYSVAR_RENT_ID    
        },
        spl_associated_token_account::{
            get_associated_token_address,
            instruction::create_associated_token_account,
        },
        spl_token::ID as TOKEN_PROGRAM_ID
    };

    // Inside the tests modules - helper function
    fn create_init_mint_ix (payer:Pubkey, program_id: Pubkey) -> 
    (Pubkey, Pubkey, Instruction) {
        // Derive PDA for token mint authority
        let (mint, _bump_seed) = Pubkey::find_program_address(&[b"token_mint"], &program_id);
        let (mint_auth, _bump_seed) = Pubkey::find_program_address(&[b"token_auth"], &program_id);

        let init_mint_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new_readonly(payer, true),
                AccountMeta::new(mint, false),
                AccountMeta::new(mint_auth, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(SYSVAR_RENT_ID, false)
            ],
            data: vec![3]
        };

        (mint, mint_auth, init_mint_ix)
    }

    // First unit test
    #[tokio::test]
    async fn test_initialize_mint_instruction(){
        let program_id = Pubkey::new_unique();
        let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
            "dapp3_student_intro_comment_with_token",
            program_id,
            processor!(process_instruction),
        )
        .start()
        .await;

        // Call helper function
        let (_mint, _mint_auth, init_mint_ix) = create_init_mint_ix(payer.pubkey(), program_id);

        // Create transaction object with instructions, accounts, and input data
        let mut transaction = Transaction::new_with_payer(
            &[init_mint_ix],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer], recent_blockhash);

        // Process transaction and compare the result
        assert_matches!(banks_client.process_transaction(transaction).await, Ok(_));
    }

    // Second unit test
    #[tokio::test]
    async fn test_add_student_intro_instruction() {
        let program_id = Pubkey::new_unique();
        let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
            "dapp3_student_intro_comment_with_token",
            program_id,
            processor!(process_instruction),
        )
        .start()
        .await;

        // Call helper function
        let (mint, mint_auth, init_mint_ix) = create_init_mint_ix(payer.pubkey(), program_id);

        // Create review PDA
        let name: String = "Celal Melal".to_owned();
        // const RATING: u8 = 3;
        let message: String = "Here to learn".to_owned();
        let (review_pda, bump_seed) = Pubkey::find_program_address(
            &[payer.pubkey().as_ref(), name.as_bytes()],
            &program_id
        );

        // Create comment PDA
        let (comment_pda, _bump_seed) = Pubkey::find_program_address(
            &[review_pda.as_ref(), b"comment"],
            &program_id
        );

        // Create user associate token account of token mint
        let init_ata_ix: Instruction = create_associated_token_account(
            &payer.pubkey(),
            &payer.pubkey(),
            &mint
        );

        let user_ata: Pubkey = get_associated_token_address(&payer.pubkey(), &mint);

        // Concat data to single buffer
        let mut data_vec = vec![0];
        data_vec.append(
            &mut (TryInto::<u32>::try_into(name.len()).unwrap().to_le_bytes())
            .try_into()
            .unwrap(),
        );
        data_vec.append(&mut name.into_bytes());
        // data_vec.push(RATING);
        data_vec.append(
            &mut (TryInto::<u32>::try_into(message.len()).unwrap().to_le_bytes())
            .try_into()
            .unwrap(),
        );
        data_vec.append(&mut message.into_bytes());

        // Create transaction object with instructions, accounts and input data
        let mut transaction = Transaction::new_with_payer(
            &[
                init_mint_ix,
                init_ata_ix,
                Instruction {
                    program_id: program_id,
                    accounts: vec![
                        AccountMeta::new_readonly(payer.pubkey(), true),
                        AccountMeta::new(review_pda, false),
                        AccountMeta::new(comment_pda, false),
                        AccountMeta::new(mint, false),
                        AccountMeta::new_readonly(mint_auth, false),
                        AccountMeta::new(user_ata, false),
                        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                    ],
                    data: data_vec,
                },
            ],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer], recent_blockhash);

        // Process transaction and compare the result
        assert_matches!(banks_client.process_transaction(transaction).await, Ok(_));

    }
}