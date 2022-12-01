use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_pack::{IsInitialized, Sealed}, pubkey::Pubkey};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct StudentIntroState {
    pub discriminator: String,
    pub reviewer: Pubkey,
    pub is_initialized: bool,
    pub name: String,
    pub message: String,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct StudentIntroCommentCounter {
    pub discriminator: String,
    pub is_intialized: bool,
    pub counter: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct StudentIntroComment {
    pub discriminator: String,
    pub is_initialized: bool,
    pub review: Pubkey,
    pub commenter: Pubkey,
    pub comment: String,
    pub count: u64,
}

impl Sealed for StudentIntroState {}

impl Sealed for StudentIntroCommentCounter {}

impl IsInitialized for StudentIntroState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl IsInitialized for StudentIntroCommentCounter {
    fn is_initialized(&self) -> bool {
        self.is_intialized
    }
}

impl IsInitialized for StudentIntroComment {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl StudentIntroState {
    pub const DISCRIMINATOR: &'static str = "studentintro";

    pub fn get_account_size(name: String, message: String) -> usize {
                // 4 bytes to store the size of the subsequent dynamic data (string)
        return (4 + StudentIntroState::DISCRIMINATOR.len())  
            + 1 // 1 byte for is_initialized (boolean)
            + (4 + name.len()) // 4 bytes to store the size of the subsequent dynamic data (string)
            + (4 + message.len()); // Same as above
    }
}

impl StudentIntroComment {
    pub const DISCRIMINATOR: &'static str = "comment";

    pub fn get_account_size(comment: String) -> usize {
        return (4 + StudentIntroComment::DISCRIMINATOR.len()) 
        + 1  // 1 byte for is_initialized (boolean)
        + 32 // 32 bytes for the movie review account key 
        + 32 // 32 bytes for the commenter key size
        + (4 + comment.len()) // 4 bytes to store the size of the subsequent dynamic data (string)
        + 8; // 8 bytes for the count (u64)
    }
}

impl StudentIntroCommentCounter {
    pub const DISCRIMINATOR: &'static str = "counter";
    pub const SIZE: usize = (4 + StudentIntroCommentCounter::DISCRIMINATOR.len()) + 1 + 8;
}
