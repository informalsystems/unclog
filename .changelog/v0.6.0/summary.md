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
