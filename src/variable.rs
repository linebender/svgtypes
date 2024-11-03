//! Parsing for [CSS Variables Level 1].
//!
//! [CSS Variables Level 1]: https://www.w3.org/TR/css-variables-1

use crate::stream::Stream;
use crate::Error;
use std::marker::PhantomData;

/// Fallback values for [variable] functions.
/// 
/// [variable]: https://www.w3.org/TR/css-variables-1/#using-variables
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableFallback<'a> {
    /// No fallback specified.
    /// 
    /// Discard the rule on substitution failure.
    None,
    /// Empty fallback is specified.
    /// 
    /// Treat as empty value on substitution failure.
    Empty(PhantomData<&'a str>),
    /// Fallback is specified.
    /// 
    /// Use the fallback on substitution failure.
    Some(&'a str),
}

impl<'a> From<Option<&'a str>> for VariableFallback<'a> {
    fn from(value: Option<&'a str>) -> Self {
        match value {
            Some("") => VariableFallback::Empty(PhantomData),
            Some(not_empty) => VariableFallback::Some(not_empty),
            None => VariableFallback::None,
        }
    }
}

/// Arbitrary substitution function with optional fallback ([`var()`]).
/// 
/// [`var()`]: https://drafts.csswg.org/css-variables/#funcdef-var
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VariableFunction<'a> {
    pub variable: &'a str,
    pub fallback: VariableFallback<'a>
}

impl<'a> VariableFunction<'a> {
    /// Parsers a `Variable` from a string.
    ///
    /// We can't use the `FromStr` trait because it requires
    /// an owned value as a return type.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(text: &'a str) -> Result<Self, Error> {
        let mut s = Stream::from(text);
        let (variable, fallback) = s.parse_var_func()?;
        Ok(VariableFunction {
            variable,
            fallback: VariableFallback::from(fallback)
        })
    }
}

impl<'a> Stream<'a> {
    /// Parses a single [custom property] identifier.
    ///
    /// It's expected to start with double dash ("--").
    ///
    /// [custom property]: https://www.w3.org/TR/css-variables-1/#defining-variables
    pub fn parse_custom_property(&mut self) -> Result<&'a str, Error> {
        self.skip_spaces();
        if !self.starts_with(b"--") {
            return Err(Error::InvalidIdent);
        }
        let name = self.consume_bytes(|_, c| !c.is_ascii_whitespace());
        if name.is_empty() {
            // only "--" is reserved by the spec for future use
            Err(Error::InvalidValue)
        } else {
            Ok(name)
        }
    }

    pub fn parse_var_func(&mut self) -> Result<(&'a str, Option<&'a str>), Error> {
        // https://www.w3.org/TR/css-variables-1/#syntax
        // var() = var( <custom-property-name> , <declaration-value>? )

        self.skip_spaces();
        self.consume_string(b"var(")?;
        self.skip_spaces();

        let custom_property_name = self.parse_custom_property()?;

        self.skip_spaces();
        if self.curr_byte()? != b',' {
            self.consume_byte(b')')?;
            return Ok((custom_property_name, None));
        } else {
            self.advance(1);
            self.skip_spaces();
        };

        let mut declaration_value = Some(self.curr_empty());
        if self.curr_byte()? == b')' {
            // https://www.w3.org/TR/css-variables-1/#using-variables
            // var(--a,) is a valid function, specifying that if the --a custom
            // property is invalid or missing, the var() should be replaced with
            // nothing
            return Ok((custom_property_name, declaration_value));
        }

        // The <declaration-value> production matches any sequence of one or
        // more tokens, so long as the sequence does not contain
        // <bad-string-token>, <bad-url-token>, unmatched <)-token>, <]-token>,
        // or <}-token>, or top-level <semicolon-token> tokens or <delim-token>
        // tokens with a value of "!".

        let mut wrap = Vec::new();
        loop {
            match self.next_byte()? {
                b'(' => wrap.push(b')'),
                b'[' => wrap.push(b']'),
                b'{' => wrap.push(b']'),
                b'"' | b'\'' => {
                    // check for <bad-string-token>
                    self.parse_quoted_string()?;
                },
                b'u' if self.starts_with(b"url(") => {
                    // check for <bad-url-token>
                    self.parse_func_iri()?;
                }
                b')' => match wrap.last() {
                    Some(b')') => {
                        wrap.pop();
                    }
                    // unmatched <)-token>
                    Some(_) => return Err(Error::InvalidValue),
                    // <declaration-value> termination
                    None => {
                        declaration_value =
                            declaration_value.map(|start| self.terminate_start(start).trim_end());
                        break;
                    }
                },
                b']' => match wrap.last() {
                    Some(b']') => {
                        wrap.pop();
                    }
                    // unmatched <]-token>
                    Some(_) | None => return Err(Error::InvalidValue),
                },
                b'}' => match wrap.last() {
                    Some(b'}') => {
                        wrap.pop();
                    }
                    // unmatched <}-token>
                    Some(_) | None => return Err(Error::InvalidValue),
                },
                // top-level <semicolon-token> token
                b';' if wrap.is_empty() => {
                    return Err(Error::InvalidValue)
                }
                // top-level <delim-token> token with value '!'
                b'!' if wrap.is_empty() => {
                    return Err(Error::InvalidValue)
                }
                _ => {}
            }
            self.advance(1);
        }

        Ok((custom_property_name, declaration_value))
    }
}

#[cfg(test)]
mod tests {
    // TODO: Add tests
}
