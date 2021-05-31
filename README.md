# unclog

**Unclog your changelog**

Build your changelog from a structured collection of independent files in your
project's `.changelog` folder. This helps avoid annoying merge conflicts when
working on multiple PRs simultaneously.

It's assumed your changelog will be output in **Markdown** format.

## Requirements

* Rust v1.52.1+ with `cargo`

## Installation

```bash
# Install to ~/.cargo/bin/
cargo install unclog
```

## Usage

### Example `.changelog` folder

An example layout for a project's `.changelog` folder is as follows:

```
.changelog/                - The project's .changelog folder, in the root of the repo.
|__ unreleased/            - Changes to be released in the next version.
|   |__ breaking-changes/  - "BREAKING CHANGES" section entries.
|   |   |__ 890-block.md   - An entry in the "BREAKING CHANGES" section.
|   |
|   |__ bug-fixes/         - "BUG FIXES" section entries.
|   |__ features/          - "FEATURES" section entries.
|   |
|   |__ summary.md         - A summary of the next release.
|
|__ v0.1.0/                - Changes released historically in v0.1.0.
|   |__ breaking-changes/  - "BREAKING CHANGES" section entries for v0.1.0.
|   |   |__ 467-api.md     - An entry in the "BREAKING CHANGES" section for v0.1.0.
|   |   |__ 479-rpc.md     - Another entry in the "BREAKING CHANGES" section for v0.1.0.
|   |
|   |__ bug-fixes/         - "BUG FIXES" section entries for v0.1.0.
|   |
|   |__ summary.md         - A summary of release v0.1.0.
|
|__ epilogue.md            - Any content to be added to the end of the generated CHANGELOG.
```

### CLI

Once you've installed the `unclog` binary:

```bash
# Run from your project's directory to build your '.changelog' folder.
# Builds your CHANGELOG.md and writes it to stdout.
unclog

# Save the output as your new CHANGELOG.md file.
unclog > CHANGELOG.md

# Get help
unclog --help
```

### As a Library

By default, the `cli` feature is enabled, which builds the CLI. To use `unclog`
as a library instead without the CLI:

```toml
[dependencies]
unclog = { version = "0.1.0", default-features = false }
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
