
# Oxide pkgs collection

[This](https://github.com/OxidePM/oxide-pkgs) is the **work in progress** collection of packages that can be installed with the [oxide](https://github.com/OxidePM/oxide) package manager.

Short term goals:
- [ ] Move `build`, `development`, `misc`, `stdenv`, ... to own crate
to not have to recompile those parts if something in another crate changes
- [ ] Add `UNIX` tools that are built using `./configure; make; make install`

