// Copyright 2018 the SVG Types Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

use alloc::string::String;

/// List of all errors.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// An input data ended earlier than expected.
    ///
    /// Should only appear on invalid input data.
    /// Errors in a valid XML should be handled by errors below.
    UnexpectedEndOfStream,

    /// An input text contains unknown data.
    UnexpectedData(usize),

    /// A provided string doesn't have a valid data.
    ///
    /// For example, if we try to parse a color form `zzz`
    /// string - we will get this error.
    /// But if we try to parse a number list like `1.2 zzz`,
    /// then we will get `InvalidNumber`, because at least some data is valid.
    InvalidValue,

    /// An invalid ident.
    ///
    /// CSS idents have certain rules with regard to the characters they may contain.
    /// For example, they may not start with a number. If an invalid ident is encountered,
    /// this error will be returned.
    InvalidIdent,

    /// An invalid/unexpected character.
    ///
    /// The first byte is an actual one, others - expected.
    ///
    /// We are using a single value to reduce the struct size.
    InvalidChar([u8; 2], usize),

    /// An unexpected character instead of an XML space.
    ///
    /// The first string is an actual one, others - expected.
    ///
    /// We are using a single value to reduce the struct size.
    InvalidString([String; 2], usize),

    /// An invalid number.
    InvalidNumber(usize),
}

macro_rules! invalid_string_choices {
    ($($s:expr),* $(,)?) => {
        [$($s),*].join("', '")
    };
}

pub(crate) use invalid_string_choices;

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::UnexpectedEndOfStream => {
                write!(f, "unexpected end of stream")
            }
            Self::UnexpectedData(pos) => {
                write!(f, "unexpected data at position {pos}")
            }
            Self::InvalidValue => {
                write!(f, "invalid value")
            }
            Self::InvalidIdent => {
                write!(f, "invalid ident")
            }
            Self::InvalidChar(ref chars, pos) => {
                write!(
                    f,
                    "expected '{}' not '{}' at position {}",
                    chars[1] as char, chars[0] as char, pos
                )
            }
            Self::InvalidString(ref strings, pos) => {
                write!(
                    f,
                    "expected '{}' not '{}' at position {}",
                    strings[1], strings[0], pos
                )
            }
            Self::InvalidNumber(pos) => {
                write!(f, "invalid number at position {pos}")
            }
        }
    }
}

impl core::error::Error for Error {
    fn description(&self) -> &str {
        "an SVG data parsing error"
    }
}
