
# Oxide pkgs collection

[This](https://github.com/OxidePM/oxide-pkgs) is the **work in progress** collection of packages that can be installed with the [oxide](https://github.com/OxidePM/oxide) package manager.

**It is far from complete and in many parts there are still nix specific setups**\
**TBD: I tried to move every bash global or env variable to be UPPERCASE and every function to be snake_case but there is a mixture of both**\
__what naming conventions should be enforced?__

Short term goals:
- [ ] Move `build`, `development`, `misc`, `stdenv`, ... to own crate
to not have to recompile those parts if something in another crate changes
- [ ] Add `UNIX` tools that are built using `./configure; make; make install`

