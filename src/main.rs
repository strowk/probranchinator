mod analysis;
mod app;
mod cli;
mod clone;
mod recent;
mod repo;
mod result;

use clap::Parser;

fn main() -> eyre::Result<()> {
    env_logger::init();
    let args = cli::Args::parse();
    let probranchinator = Probranchinator {};
    return app::run_probranchinator(
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
