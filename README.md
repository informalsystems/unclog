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

#### Initializing a changelog

```bash
# Creates a ".changelog" folder in the current directory.
unclog init

# Creates a ".changelog" folder in the current directory, and also copies your
# existing CHANGELOG.md into it as an epilogue (to be appended at the end of
# the final changelog built by unclog).
unclog init -e CHANGELOG.md
```

#### Adding a new unreleased entry

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
