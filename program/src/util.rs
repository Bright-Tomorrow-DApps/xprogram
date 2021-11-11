use std::{convert::TryInto, str::from_utf8};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError};

pub fn str_unpack<'a>(src: &'a [u8; 100]) -> &'a str {
    let mut split_index: usize = 0;
    for (i, char) in src.iter().enumerate() {
        if *char == '|' as u8 {
            split_index = i;
            break;
        }
    }
    if split_index == 0 {
        return ""
    }

    let (str_body, _) = src.split_at(split_index);

    let str = from_utf8(str_body).unwrap();
    str
}

pub fn str_pack(str: &str, dst: &mut [u8; 100]) {
    let str_bytes = str.as_bytes();
    for (i, char) in str_bytes.iter().enumerate() {
        dst[i] = *char
    }
    dst[str_bytes.len()] = '|' as u8;
}

#[cfg(test)]
mod tests {
    use crate::util::*;

    #[test]
    fn test_str_pack_unpack() {
        let str_test = "tests";
        let mut str_bytes: [u8; 100] = [0; 100];
        str_pack(str_test, &mut str_bytes);

        let unpack_str = str_unpack(&str_bytes);
        assert_eq!(str_test, unpack_str);
    }
}
