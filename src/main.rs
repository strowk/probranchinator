use clap::{arg, command, Parser};
mod analysis;
mod app;
use std::error::Error;
mod clone;
mod repo;
use crate::repo::get_repo;

/// Terminal tool to analyse conflicts between branches in a git repository
#[derive(Parser, Debug)]
#[command(name = "probranchinator", version, author)]
struct Args {
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
    remote: String,

    #[arg(long, default_value_t = 10)]
    /// Number of recent branches to analyse
    /// 
    /// If no branches are provided, the tool will analyse the most recent branches,
    /// up to the number provided by this argument.
    /// If branches are provided, `--recent` will be ignored.
    recent: usize,

    /// List of branches to analyse
    /// 
    /// If no branches are provided, the tool will analyse the most recent branches,
    /// up to the number provided by the `--recent` argument.
    branches: Vec<String>,
}

// TODO:
// - [ ] Add a way to select list of branches to analyse

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let repo = get_repo(args.remote.as_str())?;
    return app::run_app(repo, args.branches, args.recent);
}

#[cfg(test)]
mod tests {
    mod repo_test;
}
