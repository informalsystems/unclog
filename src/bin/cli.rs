//! `unclog` helps you build your changelog.

use log::error;
use simplelog::{ColorChoice, LevelFilter, TermLogger, TerminalMode};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use unclog::{Changelog, Config, Error, PlatformId, ProjectType, Result, RustProject};

const RELEASE_SUMMARY_TEMPLATE: &str = r#"<!--
    Add a summary for the release here.

    If you don't change this message, or if this file is empty, the release
    will not be created. -->
"#;

const ADD_CHANGE_TEMPLATE: &str = r#"<!--
    Add your entry's details here (in Markdown format).

    If you don't change this message, or if this file is empty, the entry will
    not be created. -->
"#;

const DEFAULT_CHANGELOG_DIR: &str = ".changelog";
const DEFAULT_CONFIG_FILENAME: &str = "config.toml";

#[derive(StructOpt)]
struct Opt {
    /// The path to the changelog folder.
    #[structopt(short, long, default_value = DEFAULT_CHANGELOG_DIR)]
    path: PathBuf,

    /// The path to the changelog configuration file. If a relative path is
    /// provided, it is assumed this is relative to the `path` parameter. If no
    /// configuration file exists, defaults will be used for all parameters.
    #[structopt(short, long, default_value = DEFAULT_CONFIG_FILENAME)]
    config_file: PathBuf,

    /// Increase output logging verbosity to DEBUG level.
    #[structopt(short, long)]
    verbose: bool,

    /// Suppress all output logging (overrides `--verbose`).
    #[structopt(short, long)]
    quiet: bool,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    /// Create and initialize a fresh .changelog folder.
    Init {
        /// The path to an epilogue to optionally append to the new changelog.
        #[structopt(name = "epilogue", short, long)]
        maybe_epilogue_path: Option<PathBuf>,
    },
    /// Add a change to the unreleased set of changes.
    Add {
        /// The path to the editor to use to edit the details of the change.
        #[structopt(long, env = "EDITOR")]
        editor: PathBuf,

        /// The component to which this entry should be added.
        #[structopt(name = "component", short, long)]
        maybe_component: Option<String>,

        /// The ID of the section to which the change must be added (e.g.
        /// "breaking-changes").
        #[structopt(short, long)]
        section: String,

        /// The ID of the change to add, which should include the number of the
        /// issue or PR to which the change applies (e.g. "820-change-api").
        #[structopt(short, long)]
        id: String,

        /// The issue number associated with this change, if any. Only relevant
        /// if the `--message` flag is also provided. Only one of the
        /// `--issue-no` or `--pull-request` flags can be specified at a time.
        #[structopt(name = "issue_no", short = "n", long)]
        maybe_issue_no: Option<u32>,

        /// The number of the pull request associated with this change, if any.
        /// Only relevant if the `--message` flag is also provided. Only one of
        /// the `--issue-no` or `--pull-request` flags can be specified at a
        /// time.
        #[structopt(name = "pull_request", short, long)]
        maybe_pull_request: Option<u32>,

        /// If specified, the change will automatically be generated from the
        /// default change template. Requires a project URL to be specified in
        /// the changelog configuration file.
        #[structopt(name = "message", short, long)]
        maybe_message: Option<String>,
    },
    /// Build the changelog from the input path and write the output to stdout.
    Build {
        /// Only render unreleased changes.
        #[structopt(short, long)]
        unreleased: bool,

        /// The type of project this is. Overrides the project type specified in
        /// the configuration file. If not specified, unclog will attempt to
        /// autodetect the project type.
        #[structopt(name = "type", short, long)]
        maybe_project_type: Option<ProjectType>,
    },
    /// Release any unreleased features.
    Release {
        /// The path to the editor to use to edit the release summary.
        #[structopt(long, env = "EDITOR")]
        editor: PathBuf,

        /// The version string to use for the new release (e.g. "v0.1.0").
        #[structopt(long)]
        version: String,
    },
}

fn main() {
    let opt: Opt = Opt::from_args();
    TermLogger::init(
        if opt.quiet {
            LevelFilter::Off
        } else if opt.verbose {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        },
        Default::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
    .unwrap();

    let config_path = if opt.config_file.is_relative() {
        opt.path.join(opt.config_file)
    } else {
        opt.config_file
    };
    let config = Config::read_from_file(config_path).unwrap();

    let result = match opt.cmd {
        Command::Init {
            maybe_epilogue_path,
        } => Changelog::init_dir(&config, opt.path, maybe_epilogue_path),
        Command::Build {
            unreleased,
            maybe_project_type,
        } => build_changelog(&config, &opt.path, unreleased, maybe_project_type),
        Command::Add {
            editor,
            maybe_component,
            section,
            id,
            maybe_issue_no,
            maybe_pull_request,
            maybe_message,
        } => match maybe_message {
            Some(message) => match maybe_issue_no {
                Some(issue_no) => match maybe_pull_request {
                    Some(_) => Err(Error::EitherIssueNoOrPullRequest),
                    None => Changelog::add_unreleased_entry_from_template(
                        &config,
                        &opt.path,
                        &section,
                        maybe_component,
                        &id,
                        PlatformId::Issue(issue_no),
                        &message,
                    ),
                },
                None => match maybe_pull_request {
                    Some(pull_request) => Changelog::add_unreleased_entry_from_template(
                        &config,
                        &opt.path,
                        &section,
                        maybe_component,
                        &id,
                        PlatformId::PullRequest(pull_request),
                        &message,
                    ),
                    None => Err(Error::MissingIssueNoOrPullRequest),
                },
            },
            None => add_unreleased_entry_with_editor(
                &config,
                &editor,
                &opt.path,
                &section,
                maybe_component,
                &id,
            ),
        },
        Command::Release { editor, version } => {
            prepare_release(&config, &editor, &opt.path, &version)
        }
    };
    if let Err(e) = result {
        error!("Failed: {}", e);
        std::process::exit(1);
    }
}

fn build_changelog(
    config: &Config,
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
    let changelog = project.read_changelog(config)?;
    log::info!("Success!");
    if unreleased {
        println!("{}", changelog.render_unreleased(config)?);
    } else {
        println!("{}", changelog.render(config));
    }
    Ok(())
}

fn add_unreleased_entry_with_editor(
    config: &Config,
    editor: &Path,
    path: &Path,
    section: &str,
    component: Option<String>,
    id: &str,
) -> Result<()> {
    let entry_path = Changelog::get_entry_path(
        config,
        path,
        &config.unreleased.folder,
        section,
        component.clone(),
        id,
    );
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

    Changelog::add_unreleased_entry(config, path, section, component, id, &tmpfile_content)
}

fn prepare_release(config: &Config, editor: &Path, path: &Path, version: &str) -> Result<()> {
    // Add the summary to the unreleased folder, since we'll be moving it to
    // the new release folder
    let summary_path = path
        .join(&config.unreleased.folder)
        .join(&config.change_sets.summary_filename);
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

    Changelog::prepare_release_dir(config, path, version)
}
