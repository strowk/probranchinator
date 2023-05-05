use std::path::PathBuf;

use fehler::throws;

#[throws(eyre::Error)]
pub(crate) fn clone_repo(repo_url: &str, dst: &PathBuf) {
    println!("Cloning repository");
    let mut cmd = std::process::Command::new("git");
    cmd.arg("clone");
    cmd.arg(repo_url);
    cmd.arg(dst);
    let output = cmd.output()?;
    if !output.status.success() {
        eyre::bail!(format!(
            "Failed to clone repository: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
}
