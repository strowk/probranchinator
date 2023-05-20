use std::fmt::Display;

use clap::{arg, command, Parser, ValueEnum};

#[derive(Clone, Debug, ValueEnum)]
pub(crate) enum OutputType {
    Table,
    Simple,
    Markdown,
    Json,
    Interactive,
}

impl Display for OutputType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OutputType::Table => write!(f, "table"),
            OutputType::Markdown => write!(f, "markdown"),
            OutputType::Simple => write!(f, "simple"),
            OutputType::Json => write!(f, "json"),
            OutputType::Interactive => write!(f, "interactive"),
        }
    }
}

// custom boolean to allow for --pretty to be true by default
#[derive(Clone, Debug, ValueEnum, PartialEq, Eq)]
pub(crate) enum BooleanCLI {
    True,
    False,
}

impl Display for BooleanCLI {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BooleanCLI::True => write!(f, "true"),
            BooleanCLI::False => write!(f, "false"),
        }
    }
}

/// Terminal tool to analyse conflicts between branches in a git repository
#[derive(Parser, Debug)]
#[command(name = "probranchinator", version, author)]
pub(crate) struct Args {
    #[arg(short, long)]
    /// Remote repository to analyse
    ///
    /// This can be a https/ssh URL or file:// path to a local repository.
    /// Tool would clone the repository in a temporary directory in case
    /// if it was not cloned before, in which case it would only fetch
    /// the latest changes.
    ///
    /// This is done to avoid any changes to the working repository, as
    /// the tool would checkout branches to analyse them in case if it
    /// needs to detect conflicts.
    pub remote: String,

    #[arg(long, default_value_t = 10)]
    /// Number of recent branches to analyse
    ///
    /// If no branches are provided, the tool will analyse the most recent branches,
    /// up to the number provided by this argument.
    /// If branches are provided, `--recent` will be ignored.
    pub recent: usize,

    /// List of branches to analyse
    ///
    /// If no branches are provided, the tool will analyse the most recent branches,
    /// up to the number provided by the `--recent` argument.
    pub branches: Vec<String>,

    #[arg(short, long, default_value_t = OutputType::Interactive)]
    /// How to output the results
    ///
    /// Choices are `table`, `json` and `interactive`.
    ///
    /// - table - outputs a table with the results
    /// 
    /// - json - outputs results in JSON format
    /// 
    /// - interactive - outputs results in terminal UI
    pub output: OutputType,

    #[arg(short, long, default_value_t = BooleanCLI::True)]
    /// If output should be prettified
    ///
    /// Only applicable to `json` output type.
    pub pretty: BooleanCLI,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_type_display() {
        assert_eq!(OutputType::Table.to_string(), "table");
        assert_eq!(OutputType::Markdown.to_string(), "markdown");
        assert_eq!(OutputType::Simple.to_string(), "simple");
        assert_eq!(OutputType::Json.to_string(), "json");
        assert_eq!(OutputType::Interactive.to_string(), "interactive");
    }

    #[test]
    fn test_boolean_cli_display() {
        assert_eq!(BooleanCLI::True.to_string(), "true");
        assert_eq!(BooleanCLI::False.to_string(), "false");
    }
}