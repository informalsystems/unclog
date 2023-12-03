# CHANGELOG

## v0.7.0

*Dec 3, 2023*

This release includes a few workflow enhancements that will hopefully make
users' lives a little easier and cater for a few more variations of how
changelogs are generated.

### FEATURES

- Add configuration file section `[change_set_sections]` with parameter
  `sort_entries_by` to sort entries in change set sections either by issue/PR
  number (`id`; default), or alphabetically (`entry-text`)
  ([\#147](https://github.com/informalsystems/unclog/pull/147))
- Add ability to optionally sort releases by date
  to configuration - see the README for details
  ([\#148](https://github.com/informalsystems/unclog/pull/148))
- Add CLI subcommand `find-duplicates` to assist in finding changelog entries
  that are duplicated across releases - see the README for more details
  ([\#149](https://github.com/informalsystems/unclog/pull/149))

## v0.6.0

*Mar 10, 2023*

This release introduces a few CLI-breaking changes in order to improve user
experience, so please review those breaking changes below carefully. In terms of
non-breaking changes, unclog v0.6.0 now supports the insertion of an arbitrary
prologue at the beginning of the changelog, in case you want some form of
preamble to your changelog.

Internally, the `structopt` package has been replaced by the latest version of
`clap` to build unclog's CLI, since it appears to have a better support
trajectory.

Also, special thanks to @thaligar for adding support for GitLab projects!

### BREAKING CHANGES

- When calling `unclog build`, unclog now only builds
  _released_ entries into the changelog. To build _all_ entries
  (including unreleased ones), please use `unclog build --all`
  ([\#51](https://github.com/informalsystems/unclog/issues/51))
- When calling `unclog release`, the `--version` flag has been removed and
  has become a mandatory positional argument, e.g. `unclog release v0.1.0`
  ([\#60](https://github.com/informalsystems/unclog/pull/60))

### DEPENDENCIES

- Switch from structopt to clap to remove dependency on now unmaintained
  `ansi_term` package, and update other dependencies where possible
  ([\#49](https://github.com/informalsystems/unclog/pull/49))

### FEATURES

- Add support for GitLab repositories.
  ([#37](https://github.com/informalsystems/unclog/pull/37))
- Add support for a prologue to be inserted at the beginning of the changelog
  ([\#47](https://github.com/informalsystems/unclog/issues/47))

## v0.5.1

*27 January 2023*

A minor bug fix release with a small improvement to the way new entries are
added from the CLI.

### BUG FIXES

- Escape \# in issue or PR number.
  ([\#38](https://github.com/informalsystems/unclog/issues/38))

## v0.5.0

*23 June 2022*

This release includes a minor footgun guardrail and some minor improvements to
the way I/O errors are reported.

### BREAKING CHANGES

- It is now required to add components to your `config.toml`
  file prior to creating entries referencing those components
  ([#23](https://github.com/informalsystems/unclog/issues/23))

## v0.4.1

Just one minor bug fix relating to component rendering.

### BUG FIXES

- Fixed component name rendering
  ([#19](https://github.com/informalsystems/unclog/issues/19))

## v0.4.0

This version is a pretty major breaking change from the previous one. Some of
the highlights:

1. Entries can now be automatically generated from the CLI. This is only
   available, however, for projects hosted on GitHub at the moment, since links
   to issues/pull requests need to be automatically generated.
2. A configuration file (`.changelog/config.toml`) can now be specified that
   allows you to override many of the default settings. See the `README.md` file
   for more details.
3. Components/submodules are no longer automatically detected and must be
   specified through the configuration file. This allows the greatest level of
   flexibility for all kinds of projects instead of limiting `unclog` to just
   Rust projects and implementing per-project-type component detection.

### BREAKING CHANGES

- All positional CLI arguments have now been replaced with flagged ones. See
  `unclog --help` and the project `README.md` for more details.
  ([#12](https://github.com/informalsystems/unclog/issues/12))
- Unreleased entries can now automatically be added to changelogs from the CLI.
  This necessarily introduces configuration to be able to specify the project's
  GitHub URL ([#13](https://github.com/informalsystems/unclog/issues/13))

## v0.3.0

This is a minor breaking release that now favours the use of hyphens (`-`) in
bulleted Markdown lists over asterisks (`*`). In future this will probably be
configurable.

### BREAKING CHANGES

- Replace all asterisks with hyphens for Markdown-based bulleted lists (related
  to [#2](https://github.com/informalsystems/unclog/issues/2))

## v0.2.1

*23 July 2021*

A minor release to augment the `add` command's functionality.

### FEATURES

* Added the `--component` flag to the `add` command so that you can now specify
  a component when adding a new entry.
  ([#6](https://github.com/informalsystems/unclog/issues/6))

## v0.2.0

*22 June 2021*

This release refactors some of the internals to provide support for grouping
entries by way of their respective **components**. A "component" is effectively
a module or sub-package within a project. More concretely, in a Rust project
with multiple crates, a "component" is one of those crates.

Right now, only Rust projects are really supported for this feature. If this
would be useful to other types of projects, let us know and we'll look at adding
such support.

Having per-language support works around the need for a configuration file,
letting the directory structures pack in as much meaning as possible. We could
always, of course, simply add support for a configuration file in future, which
could provide generic component support for any kind of project.

Another useful feature provided in this release is the ability to only render
unreleased changes. You can do so by running:

```bash
unclog build --unreleased

# Or
unclog build -u
```

### BREAKING CHANGES

* Add support for grouping entries by way of their **component**. This refactors
  the interface for loading changelogs such that you first need to construct a
  `Project`, and then use the `Project` instance to read the changelog.
  **NOTE**: This interface is unstable and will most likely change.
  ([#2](https://github.com/informalsystems/unclog/issues/2))

### FEATURES

* Added a `-u` or `--unreleased` flag to the `build` command to allow for only
  building the unreleased portion of the changelog
  ([#4](https://github.com/informalsystems/unclog/pull/4))

## v0.1.1

A minor release that just focuses on improving output formatting.

### IMPROVEMENTS

* Fix the formatting of the rendered changelog to make the behaviour of joining
  paragraphs more predictable
  ([#1](https://github.com/informalsystems/unclog/pull/1)).

## v0.1.0

The first release of `unclog`!

Basic features include:

* Building changelogs
* Initialization of empty `.changelog` directories
* Adding entries to the `unreleased` directory
* Automating the process of releasing unreleased features

See [README.md](README.md) for more details.

