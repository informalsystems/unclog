//! `unclog` helps you build your changelog.

use simplelog::{ColorChoice, LevelFilter, TermLogger, TerminalMode};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use unclog::{Changelog, Result};

#[derive(StructOpt)]
struct Opt {
    /// Increase output logging verbosity to DEBUG level.
    #[structopt(short, long)]
    verbose: bool,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    /// Build the changelog from the input path and write the output to stdout.
    Build {
        /// The path to the changelog folder to build.
        #[structopt(default_value = ".changelog")]
        path: PathBuf,
    },
    /// Create and initialize a fresh .changelog folder.
    Init {
        /// An optional epilogue to add to the new changelog.
        #[structopt(short, long)]
        epilogue_path: Option<PathBuf>,

        /// The path to the changelog folder to initialize.
        #[structopt(default_value = ".changelog")]
        path: PathBuf,
    },
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

    let result = match opt.cmd {
        Command::Build { path } => build_changelog(path),
        Command::Init {
            epilogue_path,
            path,
        } => init_changelog(path, epilogue_path),
    };
    if let Err(e) = result {
        log::error!("Failed with error: {}", e);
    }
}

fn build_changelog<P: AsRef<Path>>(path: P) -> Result<()> {
    let changelog = Changelog::read_from_dir(path)?;
    println!("{}", changelog);
    log::info!("Success!");
    Ok(())
}

fn init_changelog<P: AsRef<Path>, E: AsRef<Path>>(path: P, epilogue_path: Option<E>) -> Result<()> {
    Changelog::init(path, epilogue_path)
}
