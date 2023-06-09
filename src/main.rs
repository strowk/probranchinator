mod analysis;
mod cli;
mod clone;
mod interactive;
mod probranchinator;
mod recent;
mod repo;
mod result;

use clap::Parser;

fn main() -> eyre::Result<()> {
    env_logger::init();
    let args = cli::Args::parse();
    let probranchinator = Probranchinator {};
    return probranchinator::run_probranchinator(
        args,
        &mut std::io::stdout(),
        &probranchinator,
        &probranchinator,
    );
}

pub(crate) struct Probranchinator {}

#[cfg(test)]
mod tests {
    mod analysis_test;
    mod recent_test;
    mod repo_test;
    mod support;
}
