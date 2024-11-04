# hmm-rs

A Rust implementation of Haxe Module Manager ([`hmm`](https://github.com/andywhite37/hmm))

# Installation

`hmm-rs` can be installed as a binary from crates.io: https://crates.io/crates/hmm-rs

With [Rust installed](https://www.rust-lang.org/tools/install):
`cargo install hmm-rs`

or directly from git:

`cargo install --git https://github.com/ninjamuffin99/hmm-rs hmm-rs`

## TODO List

The below is a broad todo list / notes for myself.

- [ ] Github actions
  - [x] Windows Build
  - [x] Linux Build
  - [x] Mac Build
  - [ ] Github Releases
- [ ] Create tests against haxe `hmm`

### hmm commands to implement

- [ ] install - installs libraries listed in hmm.json
  - [ ] haxelib
  - [ ] git
    - Figure out how to use lib2git instead of gitoxide perhaps
    - allow writing / initalizing non-empty directories for clones?
    - instead of re-cloning, fetch and then check out specific commit
    - install with `--no-tags` for quicker install
  - [ ] check if version is already installed
- [ ] from-hxml
- [ ] reinstall
  - this should function the way that `hmm reinstall -f` would, where it force reinstalls everything.
    `hmm-rs install` should be used for cases when you updated your hmm.json manually or something
- [ ] haxelib
- [ ] git
- [ ] hg
  - probably not planned since I don't use mecurial personally or know any haxelib repos that do!
- [ ] dev
- [ ] update
- [ ] remove
- [ ] lock
  - how much depth should this go to for dependencies?
