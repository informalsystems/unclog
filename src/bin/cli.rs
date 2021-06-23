//! `unclog` helps you build your changelog.

use simplelog::{ColorChoice, LevelFilter, TermLogger, TerminalMode};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use unclog::{
    Changelog, Error, ProjectType, Result, RustProject, CHANGE_SET_SUMMARY_FILENAME,
    UNRELEASED_FOLDER,
};

const RELEASE_SUMMARY_TEMPLATE: &str = r#"<!--
    Add a summary for the release here.

    If you don't change this message, or if this file is empty, the release
    will not be created.
-->
"#;

const ADD_CHANGE_TEMPLATE: &str = r#"<!--
    Add your entry's details here (in Markdown format).

    If you don't change this message, or if this file is empty, the entry will
    not be created.
-->
"#;

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
    /// Create and initialize a fresh .changelog folder.
    Init {
        /// An optional epilogue to add to the new changelog.
        #[structopt(short, long)]
        epilogue_path: Option<PathBuf>,

        /// The path to the changelog folder to initialize.
        #[structopt(default_value = ".changelog")]
        path: PathBuf,
    },
    /// Add a change to the unreleased set of changes.
    Add {
        /// The path to the editor to use to edit the details of the change.
        #[structopt(long, env = "EDITOR")]
        editor: PathBuf,

        /// The component to which this entry should be added
        #[structopt(short, long)]
        component: Option<String>,

        /// The ID of the section to which the change must be added (e.g.
        /// "breaking-changes").
        section: String,

        /// The ID of the change to add, which should include the number of the
        /// issue or PR to which the change applies (e.g. "820-change-api").
        id: String,

        /// The path to the changelog folder to build.
        #[structopt(default_value = ".changelog")]
        path: PathBuf,
    },
    /// Build the changelog from the input path and write the output to stdout.
    Build {
        /// The path to the changelog folder to build.
        #[structopt(default_value = ".changelog")]
        path: PathBuf,

        /// Only render unreleased changes.
        #[structopt(short, long)]
        unreleased: bool,

        /// The type of project this is. If not supplied, unclog will attempt
        /// to autodetect it.
        #[structopt(short = "t", long)]
        project_type: Option<ProjectType>,
    },
    /// Release any unreleased features.
    Release {
        /// The path to the editor to use to edit the release summary.
        #[structopt(long, env = "EDITOR")]
        editor: PathBuf,

        /// The version string to use for the new release (e.g. "v0.1.0").
        version: String,

        /// The path to the changelog folder.
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
        Command::Build {
            path,
            unreleased,
            project_type,
        } => build_changelog(&path, unreleased, project_type),
        Command::Add {
            editor,
            component,
            section,
            id,
            path,
        } => add_unreleased_entry(&editor, &path, &section, component, &id),
        Command::Init {
            epilogue_path,
            path,
        } => Changelog::init_dir(path, epilogue_path),
        Command::Release {
            editor,
            version,
            path,
        } => prepare_release(&editor, &path, &version),
    };
    if let Err(e) = result {
        log::error!("Failed: {}", e);
        std::process::exit(1);
    }
}

fn build_changelog(
    path: &Path,
    unreleased: bool,
    maybe_project_type: Option<ProjectType>,
) -> Result<()> {
    let project_type = match maybe_project_type {
        Some(pt) => pt,
        None => ProjectType::autodetect(std::fs::canonicalize(path)?.parent().unwrap())?,
    };
    log::info!("Project type: {}", project_type);
    let project = match project_type {
        ProjectType::Rust => RustProject::new(path),
    };
    let changelog = project.read_changelog()?;
    log::info!("Success!");
    if unreleased {
        println!("{}", changelog.render_unreleased()?);
    } else {
        println!("{}", changelog.render_full());
    }
    Ok(())
}

fn add_unreleased_entry(
    editor: &Path,
    path: &Path,
    section: &str,
    component: Option<String>,
    id: &str,
) -> Result<()> {
    let entry_path =
        Changelog::get_entry_path(path, UNRELEASED_FOLDER, section, component.clone(), id);
    if std::fs::metadata(&entry_path).is_ok() {
        return Err(Error::FileExists(entry_path.display().to_string()));
    }

    let tmpdir = tempfile::tempdir()?;
    let tmpfile_path = tmpdir.path().join("entry.md");
    std::fs::write(&tmpfile_path, ADD_CHANGE_TEMPLATE)?;

    // Run the user's editor and wait for the process to exit
    let _ = std::process::Command::new(editor)
        .arg(&tmpfile_path)
        .status()?;

    // Check if the temporary file's content's changed, and that it's not empty
    let tmpfile_content = std::fs::read_to_string(&tmpfile_path)?;
    if tmpfile_content.is_empty() || tmpfile_content == ADD_CHANGE_TEMPLATE {
        log::info!("No changes to entry - not adding new entry to changelog");
        return Ok(());
    }

    Changelog::add_unreleased_entry(path, section, component, id, &tmpfile_content)
}

fn prepare_release(editor: &Path, path: &Path, version: &str) -> Result<()> {
    // Add the summary to the unreleased folder, since we'll be moving it to
    // the new release folder
    let summary_path = path
        .join(UNRELEASED_FOLDER)
        .join(CHANGE_SET_SUMMARY_FILENAME);
    // If the summary doesn't exist, try to create it
    if std::fs::metadata(&summary_path).is_err() {
        std::fs::write(&summary_path, RELEASE_SUMMARY_TEMPLATE)?;
    }

    // Run the user's editor and wait for the process to exit
    let _ = std::process::Command::new(editor)
        .arg(&summary_path)
        .status()?;

    // Check if the file's contents have changed - if not, don't continue with
    // the release
    let summary_content = std::fs::read_to_string(&summary_path)?;
    if summary_content.is_empty() || summary_content == RELEASE_SUMMARY_TEMPLATE {
        log::info!("No changes to release summary - not creating a new release");
        return Ok(());
    }

    Changelog::prepare_release_dir(path, version)
}
