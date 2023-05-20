mod analysis;
mod app;
mod cli;
mod clone;
mod recent;
mod repo;
mod result;

use clap::Parser;

fn main() -> eyre::Result<()> {
    let args = cli::Args::parse();
    return app::run_app(args);
}

#[cfg(test)]
mod tests {
    mod analysis_test;
    mod recent_test;
    mod repo_test;
    mod support;
}
