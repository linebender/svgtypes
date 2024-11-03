use std::str::FromStr;

use crate::{Color, Error, Stream};

/// Representation of the fallback part of the [`<paint>`] type.
///
/// Used by the [`Paint`](enum.Paint.html) type.
///
/// [`<paint>`]: https://www.w3.org/TR/SVG2/painting.html#SpecifyingPaint
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PaintFallback<'a> {
    /// An empty value.
    /// 
    /// Rule should be treated as invalid.
    Empty,
    /// The `none` value.
    None,
    /// The `currentColor` value.
    CurrentColor,
    /// [`<color>`] value.
    ///
    /// [`<color>`]: https://www.w3.org/TR/css-color-3/
    Color(Color),
    /// A `var()` function fallback.
    /// 
    /// See [`VariableFunction`][crate::variable::VariableFunction] for details.
    Variable(&'a str, Option<Box<PaintFallback<'a>>>),
}

impl<'a> PaintFallback<'a> {
    /// Parses a `PaintFallback` from a string.
    ///
    /// We can't use the `FromStr` trait because it requires
    /// an owned value as a return type.
    #[allow(clippy::should_implement_trait)]
    fn from_str(text: &'a str) -> Result<Self, Error> {
        let text = text.trim();
        match text {
            "" => return Ok(PaintFallback::Empty),
            "none" => return Ok(PaintFallback::None),
            "currentColor" =>return Ok( PaintFallback::CurrentColor),
            _ => {}
        }

        let mut s = Stream::from(text);
        if s.starts_with(b"var(") {
            let (variable, fallback) = s.parse_var_func()?;
            let fallback = match fallback {
                None => None,
                Some("") => Some(Box::new(PaintFallback::Empty)),
                Some(other) => Some(Box::new(PaintFallback::from_str(other)?))
            };
            return Ok(PaintFallback::Variable(variable, fallback));
        }

        Color::from_str(text).map(PaintFallback::Color)
    }
}

/// Representation of the [`<paint>`] type.
///
/// Doesn't own the data. Use only for parsing.
///
/// `<icccolor>` isn't supported.
///
/// [`<paint>`]: https://www.w3.org/TR/SVG2/painting.html#SpecifyingPaint
///
/// # Examples
///
/// ```
/// use svgtypes::{Paint, PaintFallback, Color};
///
/// let paint = Paint::from_str("url(#gradient) red").unwrap();
/// assert_eq!(paint, Paint::FuncIRI("gradient",
///                                  Some(PaintFallback::Color(Color::red()))));
///
/// let paint = Paint::from_str("inherit").unwrap();
/// assert_eq!(paint, Paint::Inherit);
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Paint<'a> {
    /// The `none` value.
    None,
    /// The `inherit` value.
    Inherit,
    /// The `currentColor` value.
    CurrentColor,
    /// [`<color>`] value.
    ///
    /// [`<color>`]: https://www.w3.org/TR/css-color-3/
    Color(Color),
    /// [`<FuncIRI>`] value with an optional fallback.
    ///
    /// [`<FuncIRI>`]: https://www.w3.org/TR/SVG11/types.html#DataTypeFuncIRI
    FuncIRI(&'a str, Option<PaintFallback<'a>>),
    /// The `context-fill` value.
    ContextFill,
    /// The `context-stroke` value.
    ContextStroke,
    /// A `var()` function.
    /// 
    /// See [`VariableFunction`][crate::variable::VariableFunction] for details.
    Variable(&'a str, Option<PaintFallback<'a>>),
}

impl<'a> Paint<'a> {
    /// Parses a `Paint` from a string.
    ///
    /// We can't use the `FromStr` trait because it requires
    /// an owned value as a return type.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(text: &'a str) -> Result<Self, Error> {
        let text = text.trim();
        match text {
            "none" => return Ok(Paint::None),
            "inherit" => return Ok(Paint::Inherit),
            "currentColor" => return Ok(Paint::CurrentColor),
            "context-fill" => return Ok(Paint::ContextFill),
            "context-stroke" => return Ok(Paint::ContextStroke),
            _ => {}
        }
        let mut s = Stream::from(text);
        if s.starts_with(b"url(") {
            let link = s.parse_func_iri()?;
            s.skip_spaces();
            // get fallback
            if !s.at_end() {
                let fallback = s.slice_tail();
                let fallback = PaintFallback::from_str(fallback)?;
                return Ok(Paint::FuncIRI(link, Some(fallback)));
            } else {
                return Ok(Paint::FuncIRI(link, None));
            }
        }

        if s.starts_with(b"var(") {
            let (variable, fallback) = s.parse_var_func()?;
            let fallback = match fallback {
                None => None,
                Some("") => Some(PaintFallback::Empty),
                Some(other) => Some(PaintFallback::from_str(other)?)
            };
            return Ok(Paint::Variable(variable, fallback));
        }
        
        match Color::from_str(text) {
            Ok(c) => Ok(Paint::Color(c)),
            Err(_) => Err(Error::InvalidValue),
        }
    }
}

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test {
        ($name:ident, $text:expr, $result:expr) => (
            #[test]
            fn $name() {
                assert_eq!(Paint::from_str($text).unwrap(), $result);
            }
        )
    }

    test!(parse_1, "none", Paint::None);
    test!(parse_2, "  none   ", Paint::None);
    test!(parse_3, " inherit ", Paint::Inherit);
    test!(parse_4, " currentColor ", Paint::CurrentColor);
    test!(parse_5, " red ", Paint::Color(Color::red()));
    test!(parse_6, " url(#qwe) ", Paint::FuncIRI("qwe", None));
    test!(parse_7, " url(#qwe) none ", Paint::FuncIRI("qwe", Some(PaintFallback::None)));
    test!(parse_8, " url(#qwe) currentColor ", Paint::FuncIRI("qwe", Some(PaintFallback::CurrentColor)));
    test!(parse_9, " url(#qwe) red ", Paint::FuncIRI("qwe", Some(PaintFallback::Color(Color::red()))));

    macro_rules! test_err {
        ($name:ident, $text:expr, $result:expr) => (
            #[test]
            fn $name() {
                assert_eq!(Paint::from_str($text).unwrap_err().to_string(), $result);
            }
        )
    }

    test_err!(parse_err_1, "qwe", "invalid value");
    test_err!(parse_err_2, "red icc-color(acmecmyk, 0.11, 0.48, 0.83, 0.00)", "invalid value");
    // TODO: this
//    test_err!(parse_err_3, "url(#qwe) red icc-color(acmecmyk, 0.11, 0.48, 0.83, 0.00)", "invalid color at 1:15");
}
