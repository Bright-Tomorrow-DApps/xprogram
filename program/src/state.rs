use crate::util::{str_pack, str_unpack};
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

const MAX_TOPIC_NAME: usize = 100;
const MAX_OPTION_NAME: usize = 100;

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct Topic {
    pub name: [u8; 100],
    pub options: [Option; 10],
    pub opt_current_idx: u8,
    pub owner: Pubkey,
    pub result_idx: u8,
    pub is_finished: bool,
}

impl Default for Topic {
    fn default() -> Self {
        Topic {
            name: [0; 100],
            options: [Option::default(); 10],
            opt_current_idx: 0,
            owner: Pubkey::default(),
            result_idx: 0,
            is_finished: false,
        }
    }
}

impl Topic {
    pub fn new(name: &str, owner: &Pubkey) -> Topic {
        let mut name_data: [u8; 100] = [0; 100];
        str_pack(name, &mut &mut name_data);
        Topic {
            name: name_data,
            options: [Option::default(); 10],
            opt_current_idx: 0,
            owner: *owner,
            result_idx: 0,
            is_finished: false,
        }
    }

    pub fn finalize(&mut self) -> Result<(), ProgramError> {
        self.is_finished = true;
        self.result_idx = 1;
        Ok(())
    }

    pub fn set_name(&mut self, name: &str) {
        let mut name_data: [u8; 100] = [0; 100];
        str_pack(name, &mut &mut name_data);
        self.name = name_data;
    }

    pub fn name_is_empty(&self) -> bool {
        let name_str = str_unpack(&self.name);
        name_str.is_empty()
    }

    pub fn add_option(&mut self, topic_key: &Pubkey, opt_name: &str) -> Result<(), ProgramError> {
        if self.opt_current_idx as usize == self.options.len() - 1 {
            return Err(ProgramError::InvalidArgument);
        }
        let opt = Option::new(topic_key, self.opt_current_idx, opt_name);
        self.options[self.opt_current_idx as usize] = opt;
        self.opt_current_idx += 1;
        Ok(())
    }

    pub fn vote(&mut self, opt_idx: u8, voter: &Pubkey) -> Result<(), ProgramError> {
        if self.opt_current_idx < opt_idx {
            return Err(ProgramError::InvalidArgument);
        }
        self.options[opt_idx as usize].add_voter(voter)?;
        Ok(())
    }

    pub fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref!(src, 0, 11075);
        let (name, options_bytes, opt_current_idx, owner, result_idx, is_finished) =
            array_refs![src, MAX_TOPIC_NAME, 10940, 1, 32, 1, 1];
        let mut options = [Option::default(); 10];
        for (i, option) in options.iter_mut().enumerate() {
            let (start, end) = (i * 1094, i * 1094 + 1094);
            *option = Option::unpack_from_slice(&options_bytes[start..end])?;
        }
        let opt_current_idx = opt_current_idx[0];
        let owner = Pubkey::new(owner);
        let result_idx = result_idx[0];
        let is_finished = if is_finished[0] == 1 { true } else { false };
        Ok(Topic {
            name: *name,
            options,
            opt_current_idx,
            owner,
            result_idx,
            is_finished,
        })
    }

    pub fn pack_into_slice(&self, dst: &mut [u8]) {
        let src = array_mut_ref!(dst, 0, 11075);
        let (name, options_bytes, opt_current_idx, owner, result_idx, is_finished) =
            mut_array_refs![src, MAX_TOPIC_NAME, 10940, 1, 32, 1, 1];
        name.copy_from_slice(&self.name);
        for (i, option) in self.options.iter().enumerate() {
            let (start, end) = (i * 1094, i * 1094 + 1094);
            option.pack_into_slice(&mut options_bytes[start..end]);
        }
        opt_current_idx[0] = self.opt_current_idx;
        owner.copy_from_slice(&self.owner.to_bytes());
        result_idx[0] = self.result_idx;
        if self.is_finished {
            is_finished[0] = 1;
        }
    }

    pub fn empty_bytes() -> [u8; 12168] {
        [0; 12168]
    }

    pub fn get_packed_len() -> usize {
        12168
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Option {
    belongs_to: Pubkey,
    belongs_idx: u8,
    name: [u8; 100],
    voters: [Pubkey; 30],
    current_voter_index: u8,
}

impl Default for Option {
    fn default() -> Self {
        Option {
            belongs_to: Pubkey::default(),
            belongs_idx: 0,
            name: [0; 100],
            voters: [Pubkey::default(); 30],
            current_voter_index: 0,
        }
    }
}

impl Option {
    pub fn new(belongs_to: &Pubkey, belongs_idx: u8, name: &str) -> Option {
        let mut name_data = [0; 100];
        str_pack(name, &mut name_data);
        Option {
            belongs_to: *belongs_to,
            belongs_idx,
            name: name_data,
            voters: [Pubkey::default(); 30],
            current_voter_index: 0,
        }
    }

    pub fn add_voter(&mut self, voter: &Pubkey) -> Result<(), ProgramError> {
        if self.current_voter_index as usize == self.voters.len() - 1 {
            return Err(ProgramError::InvalidArgument);
        }

        self.voters[self.current_voter_index as usize] = *voter;
        self.current_voter_index += 1;
        Ok(())
    }

    pub fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref!(src, 0, 1094);
        let (belongs_to, belongs_idx, name, voters_bytes, current_voter_index) =
            array_refs![src, 32, 1, MAX_OPTION_NAME, 960, 1];
        let belongs_to = Pubkey::new(belongs_to);
        let belongs_idx = belongs_idx[0];
        let mut voters = [Pubkey::new_unique(); 30];
        for (i, voter) in voters.iter_mut().enumerate() {
            let (start, end) = (i * 32, i * 32 + 32);
            *voter = Pubkey::new(&voters_bytes[start..end]);
        }
        let current_voter_index = current_voter_index[0];

        Ok(Option {
            belongs_to,
            belongs_idx,
            name: *name,
            voters,
            current_voter_index,
        })
    }

    pub fn pack_into_slice(&self, dst: &mut [u8]) {
        let src = array_mut_ref![dst, 0, 1094];
        let (belongs_to, belongs_idx, name, voters_bytes, current_voter_index) =
            mut_array_refs![src, 32, 1, MAX_OPTION_NAME, 960, 1];
        belongs_to.copy_from_slice(&self.belongs_to.to_bytes());
        name.copy_from_slice(&self.name);
        belongs_idx[0] = self.belongs_idx;
        for (i, pubkey) in self.voters.iter().enumerate() {
            let (start, end) = (i * 32, i * 32 + 32);
            voters_bytes[start..end].copy_from_slice(&pubkey.to_bytes());
        }
        current_voter_index[0] = self.current_voter_index;
    }

    pub fn empty_bytes() -> [u8; 1094] {
        [0; 1094]
    }

    pub fn get_packed_len() -> usize {
        1094
    }
}

#[cfg(test)]
mod tests {
    use crate::state::{Option, Topic};
    use solana_program::pubkey::Pubkey;

    #[test]
    fn test_option_pack_unpack() {
        let pkey = Pubkey::default();
        let mut opt = Option::new(&pkey, 0, "test_option");
        let mut data = Option::empty_bytes();
        opt.add_voter(&Pubkey::new_unique()).unwrap();
        opt.pack_into_slice(&mut data[..]);

        let opt2 = Option::unpack_from_slice(&data).unwrap();
        assert_eq!(opt, opt2);
    }

    #[test]
    fn test_topic_pack_unpack() {
        let pk = Pubkey::new_unique();
        let mut topic = Topic::new("test_topic", &pk);
        topic.add_option(&pk, "option_name").unwrap();
        let mut data = Topic::empty_bytes();
        topic.pack_into_slice(&mut data);

        let topic2 = Topic::unpack_from_slice(&mut data).unwrap();
        assert_eq!(topic, topic2);
    }
}
