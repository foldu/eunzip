use anyhow::Context;
use clap::CommandFactory;

include!("src/cli.rs");

const BIN: &str = env!("CARGO_PKG_NAME");

fn main() -> Result<(), anyhow::Error> {
    let mut opt = Opt::command();

    let outdir = match std::env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    for shell in &[
        clap_complete::Shell::Zsh,
        clap_complete::Shell::Bash,
        clap_complete::Shell::Fish,
    ] {
        clap_complete::generate_to(*shell, &mut opt, BIN, &outdir)
            .context("Failed generating completions")?;
    }

    Ok(())
}
