# hmm-rs

A Rust implementation of Haxe Module Manager ([`hmm`](https://github.com/andywhite37/hmm))

# Installation

`hmm-rs` can be installed as a binary from crates.io: https://crates.io/crates/hmm-rs

With [Rust installed](https://www.rust-lang.org/tools/install):
`cargo install hmm-rs`

or directly from git:

`cargo install --git https://github.com/ninjamuffin99/hmm-rs hmm-rs`

## TODO List

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
  - [ ] check if version is already installed
- [ ] from-hxml
- [ ] reinstall
- [ ] haxelib
- [ ] git
- [ ] hg
- [ ] dev
- [ ] update
- [ ] remove
- [ ] lock
