/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

//! Parsing helpers.
//!
//! # TODO
//!
//! - make sure everything works (fails?) with UTF-8, especially with bytes/chars issues.

use std::{
    cmp::PartialOrd,
    ops::Bound,
    ops::{Range, RangeBounds},
};

use num::Num;

use crate::errors::ParameterError;

/// Builds a parsing function for a numerical value in some range.
fn parse_range<T, R>(range: R) -> impl Fn(&str) -> Result<T, ParameterError>
where
    T: Num + PartialOrd,
    R: RangeBounds<T>,
{
    move |p: &str| match T::from_str_radix(p, 10) {
        Ok(value) => {
            match range.start_bound() {
                Bound::Included(min) => {
                    if value < *min {
                        return Err(ParameterError::OutOfRange);
                    }
                }
                Bound::Excluded(min_excluded) => {
                    if value <= *min_excluded {
                        return Err(ParameterError::OutOfRange);
                    }
                }
                Bound::Unbounded => {}
            }
            match range.end_bound() {
                Bound::Included(max) => {
                    if value > *max {
                        return Err(ParameterError::OutOfRange);
                    }
                }
                Bound::Excluded(max_excluded) => {
                    if value >= *max_excluded {
                        return Err(ParameterError::OutOfRange);
                    }
                }
                Bound::Unbounded => {}
            }
            Ok(value)
        }
        _ => Err(ParameterError::UnexpectedType),
    }
}

/// Parses an 2-bit unsigned integer value.
pub(super) fn parse_const_u2(par: &str) -> Result<u8, ParameterError> {
    parse_range(0..4)(par)
}

/// Parses a 4-bit integer in `[-1, 14]`, with `-1` returned as `15`.
pub(super) fn parse_const_i4(par: &str) -> Result<u8, ParameterError> {
    parse_range(-1i8..=14)(par).map(|e| (e & 0x0F) as u8)
}
/// Parses a 4-bit unsigned integer value.
pub(super) fn parse_const_u4(par: &str) -> Result<u8, ParameterError> {
    parse_range(0u8..=15)(par)
}

/// Parses an integer in `[1, 16]` and subtracts one (result is in `[0, 15]`).
pub(super) fn parse_const_u4_plus_one(par: &str) -> Result<u8, ParameterError> {
    parse_range(1u8..=16)(par).map(|e| (e - 1) as u8)
}

/// Parses an integer in `[2, 17]` and subtracts two (result is in `[0, 15]`).
pub(super) fn parse_const_u4_plus_two(par: &str) -> Result<u8, ParameterError> {
    parse_range(2u8..=17)(par).map(|e| (e - 2) as u8)
}

/// Parses a 4-bit unsigned integer value in `[0, 14]`.
pub(super) fn parse_const_u4_14(par: &str) -> Result<u8, ParameterError> {
    parse_range(0i8..=14)(par).map(|e| e as u8)
}

/// Parses a 4-bit unsigned integer value in `[1, 14]`.
pub(super) fn parse_const_u4_1_14(par: &str) -> Result<u8, ParameterError> {
    parse_range(1i8..=14)(par).map(|e| e as u8)
}

/// Parses a 4-bit unsigned integer value in `[1, 16]`.
pub(super) fn parse_const_u4_nonzero(par: &str) -> Result<u8, ParameterError> {
    parse_range(1u8..=16)(par)
}

/// Parses a 5-bit unsigned integer value.
pub(super) fn parse_const_u5(par: &str) -> Result<u8, ParameterError> {
    parse_range(0u8..32)(par)
}

/// Parses a 10-bit unsigned integer value.
pub(super) fn parse_const_u10(par: &str) -> Result<u16, ParameterError> {
    parse_range(0..1024)(par)
}

/// Parses an 11-bit unsigned integer value.
///
/// Used by `THROW` instructions.
pub(super) fn parse_const_u11(par: &str) -> Result<u16, ParameterError> {
    parse_range(0u16..2048)(par)
}

/// Parses a 14-bit unsigned integer value.
pub(super) fn parse_const_u14(par: &str) -> Result<u16, ParameterError> {
    parse_range(0u16..16384)(par)
}

/// Parses an [`i32`] integer in `[-15, 240[` and casts it as a u8.
///
/// Used for parsing arguments for `SETCP` `-15..240`.
pub(super) fn parse_const_u8_setcp(par: &str) -> Result<u8, ParameterError> {
    parse_range(-15..240)(par).map(|z| z as u8)
}
#[test]
fn test_parse_const_u8_setcp() {
    let data = [
        ("-15", 241),
        ("-5", 251),
        ("0", 0),
        ("1", 1),
        ("42", 42),
        ("150", 150),
        ("239", 239),
    ];
    for (data, expected) in data {
        let res = parse_const_u8_setcp(data).unwrap();
        println!("{} -> {} ({})", data, res, expected);
        assert_eq!(res, expected);
    }
    assert!(parse_const_u8_setcp("240").is_err());
}

/// Parses an [`i16`] integer in `[-128, 127]` and casts it as a u8.
pub(super) fn parse_const_i8(par: &str) -> Result<u8, ParameterError> {
    parse_range(-128i16..=127)(par).map(|e| e as u8)
}
/// Parses an integer in `[1, 256]` and subtracts one (result is in `[0, 255]`).
pub(super) fn parse_const_u8_plus_one(par: &str) -> Result<u8, ParameterError> {
    parse_range(1u16..=256)(par).map(|e| (e - 1) as u8)
}

/// Parses an integer in `[0, 240[`.
pub(super) fn parse_const_u8_240(par: &str) -> Result<u8, ParameterError> {
    parse_range(0u8..240)(par)
}

/// Parses a control (`C`) register as a `u8` in `[0, 16[`.
pub(super) fn parse_control_register(par: &str) -> Result<u8, ParameterError> {
    Ok(parse_register(par, 'C', 0..16)? as u8)
}

/// Parses a register: a symbol ([`char`]) followed by a [`isize`] in some `range`.
///
/// - `symbol` is expected to be uppercase ASCII.
///
/// Fails if
///
/// - `input` has length `1` or less;
/// - `input` does not start with `symbol` or its lowercase ASCII equivalent;
/// - `input`'s tail is not a legal [`isize`] whithin `range`.
pub(super) fn parse_register(
    input: &str,
    symbol: char,
    range: Range<isize>,
) -> Result<isize, ParameterError> {
    if input.len() <= 1 {
        Err(ParameterError::UnexpectedType)
    } else if input.chars().next().unwrap().to_ascii_uppercase() != symbol {
        Err(ParameterError::UnexpectedType)
    } else {
        match isize::from_str_radix(&input[1..], 10) {
            Ok(number) => {
                if (number < range.start) || (number >= range.end) {
                    Err(ParameterError::OutOfRange)
                } else {
                    Ok(number)
                }
            }
            Err(_e) => Err(ParameterError::UnexpectedType),
        }
    }
}

///
///
/// # Fails
///
/// - if `input` has length one or less;
/// - if `input` does not start with `'X'` or `'x'`.
///
/// # Panics
///
/// - if `bits ≥ 8`, **debug-mode only**.
pub fn parse_slice(input: &str, bits: usize) -> Result<Vec<u8>, ParameterError> {
    if input.len() <= 1 {
        log::error!(target: "compile", "empty string");
        Err(ParameterError::UnexpectedType)
    } else if input.chars().next().unwrap().to_ascii_uppercase() != 'X' {
        log::error!(target: "compile", "base not set");
        Err(ParameterError::UnexpectedType)
    } else {
        parse_slice_base(&input[1..], bits, 16)
    }
}

///
///
/// # Fails
///
/// - if `input` is not a sequence of numbers optionally ending with `'_'`.
///
/// # Panics
///
/// - if `bits ≥ 8`, **debug-mode only**.
pub fn parse_slice_base(
    input: &str,
    mut bits: usize,
    base: u32,
) -> Result<Vec<u8>, ParameterError> {
    debug_assert!(bits < 8, "offset for slice parsing cannot be ≥ 8");
    let mut acc = 0u8;
    let mut data = vec![];
    let mut completion_tag = false;
    for ch in input.chars() {
        if completion_tag {
            return Err(ParameterError::UnexpectedType);
        }
        match ch.to_digit(base) {
            Some(x) => {
                if bits < 4 {
                    acc |= (x << (4 - bits)) as u8;
                    bits += 4;
                } else {
                    data.push(acc | (x as u8 >> (bits - 4)));
                    acc = (x << (12 - bits)) as u8;
                    bits -= 4;
                }
            }
            None => {
                if ch == '_' {
                    completion_tag = true
                } else {
                    return Err(ParameterError::UnexpectedType);
                }
            }
        }
    }
    if bits != 0 {
        if !completion_tag {
            acc |= 1 << (7 - bits);
        }
        if acc != 0 || data.is_empty() {
            data.push(acc);
        }
    } else if !completion_tag {
        data.push(0x80);
    }
    Ok(data)
}

pub(super) fn parse_stack_register_u4(par: &str) -> Result<u8, ParameterError> {
    Ok(parse_register(par, 'S', 0..16)? as u8)
}

pub(super) fn parse_stack_register_u4_minus_one(par: &str) -> Result<u8, ParameterError> {
    Ok((parse_register(par, 'S', -1..15)? + 1) as u8)
}

pub(super) fn parse_stack_register_u4_minus_two(par: &str) -> Result<u8, ParameterError> {
    Ok((parse_register(par, 'S', -2..14)? + 2) as u8)
}

pub(super) fn parse_plduz_parameter(par: &str) -> Result<u8, ParameterError> {
    (parse_range(32u16..=256))(par).and_then(|c| {
        if c % 32 == 0 {
            Ok(((c / 32) - 1) as u8)
        } else {
            Err(ParameterError::OutOfRange)
        }
    })
}

pub(super) fn parse_string(arg: &str) -> Vec<u8> {
    let mut string = String::from(arg);
    if string.to_ascii_uppercase().starts_with('X') {
        string.remove(0);
        let res = hex::decode(string);
        if res.is_ok() {
            return res.unwrap();
        }
    }
    Vec::from(arg)
}
