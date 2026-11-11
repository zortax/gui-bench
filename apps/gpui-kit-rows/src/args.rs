//! Command line parsing: `--rows N`, default 100.

const DEFAULT_ROWS: usize = 100;

/// Returns the row count from the command line.
pub fn parse() -> Result<usize, String> {
    let mut rows = DEFAULT_ROWS;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let value = match arg.as_str() {
            "--rows" => args.next().ok_or("--rows needs a value")?,
            other => match other.strip_prefix("--rows=") {
                Some(value) => value.to_owned(),
                None => return Err(format!("unknown argument `{other}`")),
            },
        };
        rows = value
            .parse()
            .map_err(|_| format!("invalid row count `{value}`"))?;
    }
    Ok(rows)
}
