mod analysis;
mod app;
use std::error::Error;
mod cli;
mod clone;
mod recent;
mod repo;

use clap::Parser;

use crate::repo::get_repo;
fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Args::parse();
    let repo = get_repo(args.remote.as_str())?;
    return app::run_app(repo, args.branches, args.recent);
}

#[cfg(test)]
mod tests {
    mod repo_test;
}
