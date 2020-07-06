mod args;

use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{Clap, IntoApp};

use args::Args;

/// This is where we perform our validation of the arguments.
fn validate_arguments(mut args: Args) -> Result<Args> {
    // Check we have a pattern.
    if args.pattern.is_none() && args.patterns.is_empty() {
        return Err(anyhow!("No pattern was provided!"));
    }

    // If a positional pattern was passed _and_ patterns via flags were passed, then
    // assume that the positional pattern is a path.
    if args.pattern.is_some() && !args.patterns.is_empty() {
        args.paths.push(PathBuf::from(args.pattern.take().unwrap()));
    }

    // We don't support binary searches.
    if args.unrestricted > 2 {
        eprintln!("Binary file searching is not supported. Changing -uuu to -uu");
        args.unrestricted = 2;
    }

    Ok(args)
}

/// Prints the help generated by clap.
pub fn print_help() {
    Args::into_app().print_help().unwrap();
}

// Parses arguments from the environment (argv, etc).
pub fn parse_arguments() -> Result<Args> {
    validate_arguments(Args::parse())
}

/// Parses arguments from a list. (used in tests.)
#[allow(unused)]
pub fn parse_arguments_from<I, T>(itr: I) -> Result<Args>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    validate_arguments(Args::parse_from(itr))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use pretty_assertions::assert_eq;

    use crate::cli::parse_arguments_from;

    #[test]
    fn checks_if_no_pattern_was_passed() {
        let args = parse_arguments_from(&["rgr", "-E", "utf8", "-A1", "-B", "10"]);
        assert!(args.is_err());
        assert_eq!(
            format!("{}", args.unwrap_err()),
            String::from("No pattern was provided!")
        );
    }

    #[test]
    fn reads_pattern_as_path_if_pattern_flag_given() {
        let args = parse_arguments_from(&["rgr", "-e", "pattern-flag", "pattern-pos", "path-pos"])
            .unwrap();

        assert_eq!(args.pattern, None);
        assert_eq!(args.patterns, vec!["pattern-flag".to_owned()]);
        assert_eq!(
            args.paths,
            vec![PathBuf::from("path-pos"), PathBuf::from("pattern-pos")]
        )
    }

    #[test]
    fn does_not_allow_unrestricted_above_two() {
        let args = parse_arguments_from(&["rgr", "-uuu", "pattern-pos"]).unwrap();
        assert_eq!(args.unrestricted, 2);
    }

    #[test]
    fn returns_rg_patterns() {
        let args = parse_arguments_from(&[
            "rgr",
            "-e",
            "pattern-flag",
            "--regexp",
            "pattern-flag-long",
            "path-pos",
        ])
        .unwrap();

        assert_eq!(
            args.rg_patterns(),
            vec!["pattern-flag", "pattern-flag-long"]
        );
    }
}