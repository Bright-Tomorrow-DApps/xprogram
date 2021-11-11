use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{instruction::TopicInstruction, state::Topic};

pub struct Processor {}
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let i = TopicInstruction::unpack(input)?;

        Ok(match i {
            TopicInstruction::CreateTopic {
                topic_name,
                option_name,
            } => {
                msg!("process create topic");
                Processor::process_create_topic(program_id, accounts, topic_name, option_name)?
            }
            TopicInstruction::AddOption { option_name } => {
                msg!("process add option");
                Processor::process_add_option(program_id, accounts, option_name)?
            }
            TopicInstruction::VoteTopic { opt_idx } => {
                msg!("process vote topic");
                Processor::process_vote(program_id, accounts, opt_idx)?
            }
            TopicInstruction::FinishTopic => {
                msg!("process finish topic");
                Processor::process_finish(program_id, accounts)?
            }
        })
    }

    pub fn process_create_topic(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        topic_name: &str,
        option_name: &str,
    ) -> ProgramResult {
        let accs_iter = &mut accounts.iter();
        let topic_account = next_account_info(accs_iter)?;
        let topic_owner = next_account_info(accs_iter)?;

        if topic_account.owner != program_id {
            return Err(ProgramError::IllegalOwner);
        }
        if !topic_owner.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut topic = Topic::unpack_from_slice(&topic_account.data.borrow())?;
        if !topic.name_is_empty() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        topic.set_name(topic_name);
        topic.owner = *topic_owner.key;
        topic.add_option(topic_account.key, option_name)?;
        topic.pack_into_slice(&mut topic_account.data.borrow_mut());
        Ok(())
    }

    pub fn process_add_option(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        option_name: &str,
    ) -> ProgramResult {
        let accs_iter = &mut accounts.iter();
        let topic_account = next_account_info(accs_iter)?;
        let option_adder = next_account_info(accs_iter)?;

        if topic_account.owner != program_id {
            return Err(ProgramError::IllegalOwner);
        }
        if !option_adder.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut topic = Topic::unpack_from_slice(&topic_account.data.borrow())?;
        if topic.name.is_empty() || topic.is_finished {
            return Err(ProgramError::InvalidAccountData);
        }
        topic.add_option(topic_account.key, option_name)?;
        topic.pack_into_slice(&mut topic_account.data.borrow_mut());
        Ok(())
    }

    pub fn process_vote(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        opt_idx: u8,
    ) -> ProgramResult {
        let accs_iter = &mut accounts.iter();
        let topic_account = next_account_info(accs_iter)?;
        let voter = next_account_info(accs_iter)?;

        if topic_account.owner != program_id {
            return Err(ProgramError::IllegalOwner);
        }
        if !voter.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut topic = Topic::unpack_from_slice(&topic_account.data.borrow())?;
        if topic.name.is_empty() || topic.is_finished {
            return Err(ProgramError::InvalidAccountData);
        }
        topic.vote(opt_idx, voter.key)?;
        topic.pack_into_slice(&mut topic_account.data.borrow_mut());
        Ok(())
    }

    pub fn process_finish(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accs_iter = &mut accounts.iter();
        let topic_account = next_account_info(accs_iter)?;
        let topic_owner = next_account_info(accs_iter)?;

        if topic_account.owner != program_id {
            return Err(ProgramError::IllegalOwner);
        }
        if !topic_owner.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let mut topic = Topic::unpack_from_slice(&topic_account.data.borrow())?;
        if topic.name.is_empty() || topic.is_finished {
            return Err(ProgramError::InvalidAccountData);
        }
        if topic.owner != *topic_owner.key {
            return Err(ProgramError::IllegalOwner);
        }
        topic.finalize()?;
        topic.pack_into_slice(&mut topic_account.data.borrow_mut());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::{add_option, create_topic, finish_topic, vote_topic};
    use solana_program::{instruction::Instruction, system_program};
    use solana_sdk::account::{create_is_signer_account_infos, Account as SolanaAccount};

    fn do_process_instruction(
        instruction: Instruction,
        accounts: Vec<&mut SolanaAccount>,
    ) -> ProgramResult {
        let mut meta = instruction
            .accounts
            .iter()
            .zip(accounts)
            .map(|(account_meta, account)| (&account_meta.pubkey, account_meta.is_signer, account))
            .collect::<Vec<_>>();

        let account_infos = create_is_signer_account_infos(&mut meta);
        Processor::process(&instruction.program_id, &account_infos, &instruction.data)
    }

    struct TestSuite {
        program_id: Pubkey,
        topic_key: (Pubkey, SolanaAccount),
        topic_owner: (Pubkey, SolanaAccount),
    }

    impl TestSuite {
        fn new() -> TestSuite {
            let pid = Pubkey::default();
            TestSuite {
                program_id: pid,
                topic_key: Self::get_key_account(&pid, Topic::get_packed_len()),
                topic_owner: Self::get_key_account(&system_program::ID, Topic::get_packed_len()),
            }
        }

        fn topic_eq(&self, expect_topic: &Topic) -> Result<bool, ProgramError> {
            let topic = Topic::unpack_from_slice(&self.topic_key.1.data)?;
            Ok(topic.eq(expect_topic))
        }

        fn get_key_account(owner: &Pubkey, space: usize) -> (Pubkey, SolanaAccount) {
            (
                Pubkey::new_unique(),
                SolanaAccount::new(10000, space, owner),
            )
        }

        fn process_create_topic(&mut self, topic_name: &str, option_name: &str) -> ProgramResult {
            let i = create_topic(
                &self.program_id,
                &self.topic_key.0,
                &self.topic_owner.0,
                topic_name,
                option_name,
            )?;
            do_process_instruction(i, vec![&mut self.topic_key.1, &mut self.topic_owner.1])
        }

        fn process_add_option(&mut self, option_name: &str) -> ProgramResult {
            let i = add_option(
                &self.program_id,
                &self.topic_key.0,
                &self.topic_owner.0,
                option_name,
            )?;
            do_process_instruction(i, vec![&mut self.topic_key.1, &mut self.topic_owner.1])
        }

        fn process_vote(
            &mut self,
            option_idx: u8,
            voter: &mut (Pubkey, SolanaAccount),
        ) -> ProgramResult {
            let i = vote_topic(&self.program_id, &self.topic_key.0, &voter.0, option_idx)?;
            do_process_instruction(i, vec![&mut self.topic_key.1, &mut voter.1])
        }

        fn process_finish(&mut self) -> ProgramResult {
            let i = finish_topic(&self.program_id, &self.topic_key.0, &self.topic_owner.0)?;
            do_process_instruction(i, vec![&mut self.topic_key.1, &mut self.topic_owner.1])
        }

        fn process_init_topic(
            &mut self,
            topic_name: &str,
            first_opt: &str,
            opts: Vec<&str>,
        ) -> ProgramResult {
            self.process_create_topic(topic_name, first_opt)?;
            for opt in opts {
                self.process_add_option(opt)?;
            }
            Ok(())
        }
    }

    #[test]
    fn test_create_topic() {
        let mut ts = TestSuite::new();
        let topic_name = "test_topic";
        let opt_name = "test_option";
        ts.process_create_topic(topic_name, opt_name).unwrap();
        let mut expect = Topic::new("test_topic", &ts.topic_owner.0);
        expect.add_option(&ts.topic_key.0, opt_name).unwrap();
        assert_eq!(Ok(true), ts.topic_eq(&expect));

        assert_eq!(
            Err(ProgramError::AccountAlreadyInitialized),
            ts.process_create_topic("test_topic_2", "test_option")
        )
    }

    #[test]
    fn test_add_option() {
        let mut ts = TestSuite::new();
        let topic_name = "test_topic";
        let opt_name = "test_option";
        ts.process_create_topic(topic_name, opt_name).unwrap();
        let mut expect_topic = Topic::new(topic_name, &ts.topic_owner.0);
        expect_topic.add_option(&ts.topic_key.0, opt_name).unwrap();
        let opt_name = "test_option2";
        ts.process_add_option(opt_name).unwrap();
        expect_topic.add_option(&ts.topic_key.0, opt_name).unwrap();

        assert_eq!(Ok(true), ts.topic_eq(&expect_topic))
    }

    #[test]
    fn test_vote() {
        let mut ts = TestSuite::new();
        let topic_name = "test_topic";
        let opt_name = "test_option";
        let opt_name2 = "test_option2";
        ts.process_init_topic(topic_name, opt_name, vec![opt_name2])
            .unwrap();
        let mut expect_topic = Topic::new(topic_name, &ts.topic_owner.0);
        expect_topic.add_option(&ts.topic_key.0, opt_name).unwrap();
        expect_topic.add_option(&ts.topic_key.0, opt_name2).unwrap();

        let mut key_acc = TestSuite::get_key_account(&system_program::ID, 100);
        ts.process_vote(0, &mut key_acc).unwrap();
        expect_topic.options[0].add_voter(&key_acc.0).unwrap();
        assert_eq!(Ok(true), ts.topic_eq(&expect_topic))
    }

    #[test]
    fn test_finish_topic() {
        let mut ts = TestSuite::new();
        let topic_name = "test_topic";
        let opt_name = "test_option";
        let opt_name2 = "test_option2";
        ts.process_init_topic(topic_name, opt_name, vec![opt_name2])
            .unwrap();
        let mut expect_topic = Topic::new(topic_name, &ts.topic_owner.0);
        expect_topic.add_option(&ts.topic_key.0, opt_name).unwrap();
        expect_topic.add_option(&ts.topic_key.0, opt_name2).unwrap();
        ts.process_finish().unwrap();
        expect_topic.is_finished = true;
        expect_topic.result_idx = 1;
        assert_eq!(Ok(true), ts.topic_eq(&expect_topic))
    }
}
