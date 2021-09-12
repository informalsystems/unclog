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

