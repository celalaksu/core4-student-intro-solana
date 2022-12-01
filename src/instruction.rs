use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

pub enum StudentIntroInstruction {
    AddStudentIntro { name: String, message: String },
    UpdateStudentIntro { name: String, message: String },
    AddComment { comment: String },
    InitializeMint,
}

impl StudentIntroInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;
        
        Ok(match variant {
            0 => 
            {
                let payload = StudentIntroPayload::try_from_slice(rest).unwrap();
                Self::AddStudentIntro {
                name: payload.name,
                message: payload.message,
                }
            },
            1 =>
            {
                let payload = StudentIntroPayload::try_from_slice(rest).unwrap();
                Self::UpdateStudentIntro {
                name:payload.name,
                message:payload.message,
                }
            },
            2 => 
            {
                let payload = StudentIntroCommentPayload::try_from_slice(rest).unwrap(); 
                Self::AddComment {
                    comment: payload.comment,
                }  
            },
            3 => Self::InitializeMint,
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}

#[derive(BorshDeserialize)]
struct StudentIntroPayload {
    name: String,
    message: String,
}

#[derive(BorshDeserialize)]
struct StudentIntroCommentPayload {
    comment: String,
}
