## Bug fixes

- Using --graph-ref with a graph ref containing a branch that has slashes or other special characters in its name now works as expected, instead of behaving as if the branch did not exist.
- Fixed a panic when using resolver extensions with non-virtual subgraphs (subgraphs with URLs). Resolver extensions now properly validate that they can only be used with virtual subgraphs and return a clear error message instead of panicking.
