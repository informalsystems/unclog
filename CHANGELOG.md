# CHANGELOG

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
