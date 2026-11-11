//! The command line: `--rows N`, with 100 rows by default.

use std::fmt;

/// The row count without `--rows`.
const DEFAULT_ROWS: usize = 100;

/// A command line that names no valid row count.
#[derive(Debug, PartialEq, Eq)]
pub enum CliError {
    /// `--rows` is the last argument.
    MissingValue,
    /// The value of `--rows` is no whole number.
    BadValue(String),
    /// An argument the program does not know.
    Unknown(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingValue => write!(f, "--rows needs a value"),
            Self::BadValue(value) => write!(f, "--rows takes a whole number, got `{value}`"),
            Self::Unknown(argument) => write!(f, "unknown argument `{argument}`"),
        }
    }
}

/// Reads the row count from the arguments after the program name.
pub fn rows_from(mut args: impl Iterator<Item = String>) -> Result<usize, CliError> {
    let mut rows = DEFAULT_ROWS;
    while let Some(argument) = args.next() {
        let value = match argument.strip_prefix("--rows=") {
            Some(value) => value.to_owned(),
            None if argument == "--rows" => args.next().ok_or(CliError::MissingValue)?,
            None => return Err(CliError::Unknown(argument)),
        };
        rows = value.parse().map_err(|_| CliError::BadValue(value))?;
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::{CliError, DEFAULT_ROWS, rows_from};

    fn parse(args: &[&str]) -> Result<usize, CliError> {
        rows_from(args.iter().map(|arg| (*arg).to_owned()))
    }

    #[test]
    fn no_arguments_give_the_default() {
        assert_eq!(parse(&[]), Ok(DEFAULT_ROWS));
    }

    #[test]
    fn both_forms_set_the_count() {
        assert_eq!(parse(&["--rows", "10000"]), Ok(10000));
        assert_eq!(parse(&["--rows=10"]), Ok(10));
    }

    #[test]
    fn bad_input_is_an_error() {
        assert_eq!(parse(&["--rows"]), Err(CliError::MissingValue));
        assert!(matches!(
            parse(&["--rows", "x"]),
            Err(CliError::BadValue(_))
        ));
        assert!(matches!(parse(&["-v"]), Err(CliError::Unknown(_))));
    }
}
