use crate::{
    variable::{VariableFallback, VariableFunction},
    Error, Stream,
};

/// List of all SVG angle units.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(missing_docs)]
pub enum AngleUnit {
    Degrees,
    Gradians,
    Radians,
    Turns,
}

/// Representation of the [`<angle>`] type.
///
/// [`<angle>`]: https://www.w3.org/TR/css-values-3/#angles
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum Angle<'a> {
    Discrete { number: f64, unit: AngleUnit },
    Variable(VariableFunction<'a>),
}

impl<'a> Angle<'a> {
    /// Constructs a new angle.
    #[inline]
    pub fn new(number: f64, unit: AngleUnit) -> Self {
        Self::Discrete { number, unit }
    }

    /// Constructs a new angle variable.
    #[inline]
    pub fn new_var(variable: VariableFunction<'a>) -> Self {
        Self::Variable(variable)
    }

    /// Converts discrete angle values to degrees.
    ///
    /// If variable value is used, it will be returned instead as `Err`.
    #[inline]
    pub fn to_degrees(&self) -> Result<f64, VariableFunction<'a>> {
        match self {
            Self::Discrete { number, unit } => Ok(match unit {
                AngleUnit::Degrees => *number,
                AngleUnit::Gradians => number * 180.0 / 200.0,
                AngleUnit::Radians => number.to_degrees(),
                AngleUnit::Turns => number * 360.0,
            }),
            Self::Variable(variable) => Err(*variable),
        }
    }

    /// Parsers an `Angle` from a string.
    ///
    /// We can't use the `FromStr` trait because it requires
    /// an owned value as a return type.
    #[inline]
    fn from_str(text: &'a str) -> Result<Self, Error> {
        let mut s = Stream::from(text);
        let l = s.parse_angle()?;

        if !s.at_end() {
            return Err(Error::UnexpectedData(s.calc_char_pos()));
        }

        Ok(l)
    }
}

impl<'a> Stream<'a> {
    /// Parses angle from the stream.
    ///
    /// <https://www.w3.org/TR/SVG2/types.html#InterfaceSVGAngle>
    ///
    /// # Notes
    ///
    /// - Suffix must be lowercase, otherwise it will be an error.
    pub fn parse_angle(&mut self) -> Result<Angle<'a>, Error> {
        self.skip_spaces();

        if self.starts_with(b"var(") {
            let (variable, fallback) = self.parse_var_func()?;
            return Ok(Angle::Variable(VariableFunction {
                variable,
                fallback: VariableFallback::from(fallback),
            }));
        }

        let n = self.parse_number()?;

        if self.at_end() {
            return Ok(Angle::new(n, AngleUnit::Degrees));
        }

        let u = if self.starts_with(b"deg") {
            self.advance(3);
            AngleUnit::Degrees
        } else if self.starts_with(b"grad") {
            self.advance(4);
            AngleUnit::Gradians
        } else if self.starts_with(b"rad") {
            self.advance(3);
            AngleUnit::Radians
        } else if self.starts_with(b"turn") {
            self.advance(4);
            AngleUnit::Turns
        } else {
            AngleUnit::Degrees
        };

        Ok(Angle::new(n, u))
    }
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    macro_rules! test_p {
        ($name:ident, $text:expr, $result:expr) => (
            #[test]
            fn $name() {
                assert_eq!(Angle::from_str($text).unwrap(), $result);
            }
        )
    }

    test_p!(parse_1,  "1",   Angle::new(1.0, AngleUnit::Degrees));
    test_p!(parse_2,  "1deg", Angle::new(1.0, AngleUnit::Degrees));
    test_p!(parse_3,  "1grad", Angle::new(1.0, AngleUnit::Gradians));
    test_p!(parse_4,  "1rad", Angle::new(1.0, AngleUnit::Radians));
    test_p!(parse_5,  "1turn", Angle::new(1.0, AngleUnit::Turns));

    #[test]
    fn err_1() {
        let mut s = Stream::from("1q");
        assert_eq!(s.parse_angle().unwrap(), Angle::new(1.0, AngleUnit::Degrees));
        assert_eq!(s.parse_angle().unwrap_err().to_string(),
                   "invalid number at position 2");
    }

    #[test]
    fn err_2() {
        assert_eq!(Angle::from_str("1degq").unwrap_err().to_string(),
                   "unexpected data at position 5");
    }
}
