use crate::util::str_pack;
use solana_program::pubkey::Pubkey;
use solana_program::{
    instruction::AccountMeta, instruction::Instruction, program_error::ProgramError,
};
use std::mem::size_of;
use std::str::from_utf8;

pub enum TopicInstruction<'a> {
    CreateTopic {
        topic_name: &'a str,
        option_name: &'a str,
    },
    AddOption {
        option_name: &'a str,
    },
    VoteTopic {
        opt_idx: u8,
    },
    FinishTopic,
}

impl<'a> TopicInstruction<'a> {
    pub fn unpack(input: &'a [u8]) -> Result<Self, ProgramError> {
        use ProgramError::InvalidInstructionData;
        let (&tag, rest) = input.split_first().ok_or(InvalidInstructionData)?;

        Ok(match tag {
            0 => {
                let mut split_index: usize = 0;
                for (i, char) in rest.iter().enumerate() {
                    if *char == '|' as u8 {
                        split_index = i;
                        break;
                    }
                }
                let (topic_name, option_name) = rest.split_at(split_index);
                let (_, option_name) = option_name.split_first().ok_or(InvalidInstructionData)?;
                let topic_name = from_utf8(topic_name).unwrap();
                let option_name = from_utf8(option_name).unwrap();
                Self::CreateTopic {
                    topic_name,
                    option_name,
                }
            }
            1 => {
                let option_name = from_utf8(rest).unwrap();
                Self::AddOption { option_name }
            }
            2 => {
                let opt_idx = rest[0];
                Self::VoteTopic { opt_idx }
            }
            3 => Self::FinishTopic,
            _ => return Err(InvalidInstructionData),
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::with_capacity(size_of::<Self>());
        match *self {
            Self::CreateTopic {
                topic_name,
                option_name,
            } => {
                buf.push(0);
                buf.extend_from_slice(topic_name.as_bytes());
                buf.push('|' as u8);
                buf.extend_from_slice(option_name.as_bytes());
            }
            Self::AddOption { option_name } => {
                buf.push(1);
                buf.extend_from_slice(option_name.as_bytes());
            }
            Self::VoteTopic { opt_idx } => {
                buf.push(2);
                buf.push(opt_idx);
            }
            Self::FinishTopic => {
                buf.push(3);
            }
        }
        buf
    }
}

pub fn create_topic(
    program_id: &Pubkey,
    topic: &Pubkey,
    topic_owner: &Pubkey,
    topic_name: &str,
    option_name: &str,
) -> Result<Instruction, ProgramError> {
    let data = TopicInstruction::CreateTopic {
        topic_name,
        option_name,
    }
    .pack();
    let accounts = vec![
        AccountMeta::new(*topic, false),
        AccountMeta::new(*topic_owner, true),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

pub fn add_option(
    program_id: &Pubkey,
    topic: &Pubkey,
    option_adder: &Pubkey,
    option_name: &str,
) -> Result<Instruction, ProgramError> {
    let data = TopicInstruction::AddOption { option_name }.pack();
    let accounts = vec![
        AccountMeta::new(*topic, false),
        AccountMeta::new(*option_adder, true),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

pub fn vote_topic(
    program_id: &Pubkey,
    topic: &Pubkey,
    voter: &Pubkey,
    opt_idx: u8,
) -> Result<Instruction, ProgramError> {
    let data = TopicInstruction::VoteTopic { opt_idx }.pack();
    let accounts = vec![
        AccountMeta::new(*topic, false),
        AccountMeta::new(*voter, true),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

pub fn finish_topic(
    program_id: &Pubkey,
    topic: &Pubkey,
    topic_owner: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let data = TopicInstruction::FinishTopic.pack();
    let accounts = vec![
        AccountMeta::new(*topic, false),
        AccountMeta::new(*topic_owner, true),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}
