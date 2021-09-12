# unclog

[![Crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
[![Build Status][build-image]][build-link]
[![Apache 2.0 Licensed][license-image]][license-link]
![Rust Stable][rustc-image]

**Unclog your changelog**

Build your changelog from a structured collection of independent files in your
project's `.changelog` folder. This helps avoid annoying merge conflicts when
working on multiple PRs simultaneously.

It's assumed your changelog will be output in **Markdown** format.

### Why not just use the Git commit history?

Many other tools that provide similar functionality focus on extracting
changelog entries from the project's Git commit history. Why don't we just do
this?

We find value in targeting different audiences with each kind of content, and
being able to tailor content to each audience: Git commit histories for our
*developers*, and changelogs for our *users*.

## Requirements

* Rust v1.54+ with `cargo`

## Installation

```bash
# Install to ~/.cargo/bin/
cargo install unclog
```

Or you can build from source:

```bash
git clone https://github.com/informalsystems/unclog
cd unclog

# Install to ~/.cargo/bin/
cargo install --path .
```

## Usage

### Example `.changelog` folder

An example layout for a project's `.changelog` folder is as follows:

```
.changelog/                   - The project's .changelog folder, in the root of the repo.
|__ unreleased/               - Changes to be released in the next version.
|   |__ breaking-changes/     - "BREAKING CHANGES" section entries.
|   |   |__ 890-block.md      - An entry in the "BREAKING CHANGES" section.
|   |
|   |__ bug-fixes/            - "BUG FIXES" section entries.
|   |   |__ module1/          - "BUG FIXES" section entries specific to "module1".
|   |       |__ 745-rename.md - An entry in the "BUG FIXES" section under "module1".
|   |__ features/             - "FEATURES" section entries.
|   |
|   |__ summary.md            - A summary of the next release.
|
|__ v0.1.0/                   - Changes released historically in v0.1.0.
|   |__ breaking-changes/     - "BREAKING CHANGES" section entries for v0.1.0.
|   |   |__ 467-api.md        - An entry in the "BREAKING CHANGES" section for v0.1.0.
|   |   |__ 479-rpc.md        - Another entry in the "BREAKING CHANGES" section for v0.1.0.
|   |
|   |__ bug-fixes/            - "BUG FIXES" section entries for v0.1.0.
|   |
|   |__ summary.md            - A summary of release v0.1.0.
|
|__ epilogue.md               - Any content to be added to the end of the generated CHANGELOG.
```

For a more detailed example, see the [`tests/full`](./tests/full) folder for
the primary integration test that uses the most features/functionality. The
file [`tests/full/expected.md`](./tests/full/expected.md) is the expected
output when building the files in `tests/full`.

### CLI

```bash
# Detailed information regarding usage.
unclog -h
```

#### Initializing a changelog

```bash
# Creates a ".changelog" folder in the current directory.
unclog init

# Creates a ".changelog" folder in the current directory, and also copies your
# existing CHANGELOG.md into it as an epilogue (to be appended at the end of
# the final changelog built by unclog).
unclog init -e CHANGELOG.md

# Automatically generate a `config.toml` file for your changelog, inferring as
# many settings as possible from the environment. (Right now this mainly infers
# your GitHub project URL, if it's a GitHub project)
unclog init -g
```

#### Adding a new unreleased entry

There are two ways of adding a new entry:

1. Entirely through the CLI
2. By way of your default `$EDITOR`

To add an entry entirely through the CLI:

```bash
# First ensure your config.toml file contains the project URL:
echo 'project_url = "https://github.com/org/project"' >> .changelog/config.toml

# Add a new entry whose associated GitHub issue number is 23.
# Word wrapping will automatically be applied at the boundary specified in your
# `config.toml` file.
unclog add --id some-new-feature \
  --issue 23 \
  --section breaking-changes \
  --message "Some *new* feature"

# Same as above, but with shortened parameters
unclog add -i some-new-feature \
  -n 23 \
  -s breaking-changes \
  -m "Some *new* feature"

# If your project uses components/sub-modules
unclog add -i some-new-feature \
  -n 23 \
  -c submodule \
  -s breaking-changes \
  -m "Some *new* feature"
```

To add an entry with your favourite `$EDITOR`:

```bash
# First ensure that your $EDITOR environment variable is configured, or you can
# manually specify an editor binary path via the --editor flag.
#
# This will launch your configured editor and, if you add any content to the
# feature file it will be added to
# ".changelog/unreleased/features/23-some-new-feature.md".
#
# The convention is that you *must* prepend the issue/PR number to which the
# change refers to the entry ID (i.e. 23-some-new-feature relates to issue 23).
unclog add --section features --id 23-some-new-feature

# Add another feature in a different section
unclog add -s breaking-changes -i 24-break-the-api
```

The format of an entry is currently recommended as the following (in Markdown):

```markdown
- A user-oriented description of the change ([#123](https://github.com/someone/someproject/issues/123))
```

The `#123` and its corresponding link is ideally a link to the issue being
resolved. If there's no issue, then reference the PR.

#### Building a changelog

```bash
# Run from your project's directory to build your '.changelog' folder.
# Builds your CHANGELOG.md and writes it to stdout.
unclog build

# Only render unreleased changes (returns an error if none)
unclog build --unreleased

# Save the output as your new CHANGELOG.md file.
# NOTE: All logging output goes to stderr.
unclog build > CHANGELOG.md

# Increase output logging verbosity on stderr and build your `.changelog`
# folder.
unclog -v build

# Get help
unclog --help
```

#### Releasing a new version's change set

```bash
# Moves all entries in your ".changelog/unreleased" folder to
# ".changelog/v0.2.0" and ensures the ".changelog/unreleased" folder is empty.
unclog release --version v0.2.0
```

### Components/Submodules

If your project has components or submodules to it (see the configuration below
for details on how to specify components), referencing them when creating
changelog entries allows you to group entries for one component together. For
example:

```bash
unclog add -i some-new-feature \
  -n 23 \
  -c submodule \
  -s breaking-changes \
  -m "Some *new* feature"
```

would result in an entry being created in
`.changelog/unreleased/submodule/breaking-changes/23-some-new-feature.md` which,
when rendered, would look like:

```markdown
- [submodule](./submodule)
  - Some *new* feature ([#23](https://github.com/org/project/issues/23))
```

### Configuration

Certain `unclog` settings can be overridden through the use of a configuration
file in `.changelog/config.toml`. The following TOML shows all of the defaults
for the configuration. If you don't have a `.changelog/config.toml` file, all of
the defaults will be assumed.

```toml
# The GitHub URL for your project.
#
# This is mainly necessary if you need to automatically generate changelog
# entries directly from the CLI. Right now we only support GitHub, but if
# anyone wants GitLab support please let us know and we'll try implement it
# too.
project_url = "https://github.com/org/project"

# The file to use as a Handlebars template for changes added directly through
# the CLI.
#
# Assumes that relative paths are relative to the `.changelog` folder. If this
# file does not exist, a default template will be used.
change_template = "change-template.md"

# The number of characters at which to wrap entries automatically added from
# the CLI.
wrap = 80

# The heading right at the beginning of the changelog.
heading = "# CHANGELOG"

# What style of bullet to use for the instances where unclog has to generate
# bullets for you. Can be "-" or "*".
bullet_style = "-"

# The message to output when your changelog has no entries yet.
empty_msg = "Nothing to see here! Add some entries to get started."

# The name of the file (relative to the `.changelog` directory) to use as an
# epilogue for your changelog (will be appended as-is to the end of your
# generated changelog).
epilogue_filename = "epilogue.md"


# Settings relating to unreleased changelog entries.
[unreleased]

# The name of the folder containing unreleased entries, relative to the
# `.changelog` folder.
folder = "unreleased"

# The heading to use for the unreleased entries section.
heading = "## Unreleased"


# Settings relating to sets (groups) of changes in the changelog. For example,
# the "BREAKING CHANGES" section would be considered a change set.
[change_sets]

# The filename containing a summary of the intended changes. Relative to the
# change set folder (e.g. `.changelog/unreleased/breaking-changes/summary.md`).
summary_filename = "summary.md"

# The extension of files in a change set.
entry_ext = "md"


# Settings related to components/sub-modules. Only relevant if you make use of
# components/sub-modules.
[components]

# The title to use for the section of entries not relating to a specific
# component.
general_entries_title = "General"

# The number of spaces to inject before each component-related entry.
entry_indent = 2

    # The components themselves. Each component has a name (used when rendered
    # to Markdown) and a path relative to the project folder (i.e. relative to
    # the parent of the `.changelog` folder).
    [components.all]
    component1 = { name = "Component 1", path = "component1" }
    docs = { name = "Documentation", path = "docs" }
```

### As a Library

By default, the `cli` feature is enabled, which builds the CLI. To use `unclog`
as a library instead without the CLI:

```toml
[dependencies]
unclog = { version = "0.3", default-features = false }
```

## License

Copyright © 2021 Informal Systems

Licensed under the Apache License, Version 2.0 (the "License");
you may not use the files in this repository except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

[crate-image]: https://img.shields.io/crates/v/unclog.svg
[crate-link]: https://crates.io/crates/unclog
[docs-image]: https://docs.rs/unclog/badge.svg
[docs-link]: https://docs.rs/unclog/
[build-image]: https://github.com/informalsystems/unclog/workflows/Rust/badge.svg
[build-link]: https://github.com/informalsystems/unclog/actions?query=workflow%3ARust
[audit-image]: https://github.com/informalsystems/unclog/workflows/Audit-Check/badge.svg
[audit-link]: https://github.com/informalsystems/unclog/actions?query=workflow%3AAudit-Check
[license-image]: https://img.shields.io/badge/license-Apache2.0-blue.svg
[license-link]: https://github.com/informalsystems/unclog/blob/master/LICENSE
[rustc-image]: https://img.shields.io/badge/rustc-stable-blue.svg
