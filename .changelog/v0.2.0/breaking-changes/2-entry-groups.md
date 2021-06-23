* Add support for grouping entries by way of their **component**. This refactors
  the interface for loading changelogs such that you first need to construct a
  `Project`, and then use the `Project` instance to read the changelog.
  **NOTE**: This interface is unstable and will most likely change.
  ([#2](https://github.com/informalsystems/unclog/issues/2))
