use clap::{arg, command, Parser};
mod analysis;
mod app;
use std::error::Error;
mod repo;
mod clone;
use crate::repo::get_repo;

/// Terminal tool to analyse conflicts between branches in a git repository
#[derive(Parser, Debug)]
#[command(name = "probranchinator", version, author)]
struct Args {
    #[arg(short, long)]
    remote: String,

    branches: Vec<String>,
}

// TODO:
// - [ ] Add a way to select list of branches to analyse

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let repo = get_repo(args.remote.as_str())?;
    return app::run_app(repo, args.branches);
}

#[cfg(test)]
mod tests {
    mod repo_test;
}
