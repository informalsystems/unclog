//! `unclog` helps you build your changelog.

use simplelog::{ColorChoice, LevelFilter, TermLogger, TerminalMode};
use std::convert::TryFrom;
use std::error::Error;
use std::path::Path;
use structopt::StructOpt;
use unclog::Changelog;

#[derive(StructOpt)]
struct Opt {
    /// Increase output logging verbosity to DEBUG level.
    #[structopt(short, long)]
    verbose: bool,

    /// The path to the '.changelog' folder you want to build.
    #[structopt(default_value = ".changelog")]
    path: String,
}

fn main() {
    let opt: Opt = Opt::from_args();
    TermLogger::init(
        if opt.verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        },
        Default::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
    .unwrap();

    if let Err(e) = build_changelog(&opt.path) {
        log::error!("Failed to build changelog: {}", e);
    }
}

fn build_changelog(path: &str) -> Result<(), Box<dyn Error>> {
    let input = unclog::ChangelogInput::load_from_dir(Path::new(path))?;
    let changelog = Changelog::try_from(input)?;
    println!("{}", changelog);
    Ok(())
}
