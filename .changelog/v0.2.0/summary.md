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
