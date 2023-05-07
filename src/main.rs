mod analysis;
mod app;
use std::error::Error;
mod cli;
mod clone;
mod recent;
mod repo;

use clap::Parser;

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Args::parse();
    return app::run_app(args);
}

#[cfg(test)]
mod tests {
    mod repo_test;
}
