# hmm-rs

A Rust implementation of Haxe Module Manager ([`hmm`](https://github.com/andywhite37/hmm))

In progress and may or may not be completed, I'm stil learning :)

# Installation

`hmm-rs` can be installed as a binary from crates.io: https://crates.io/crates/hmm-rs

With [Rust installed](https://www.rust-lang.org/tools/install):
`cargo install hmm-rs`

or directly from git:

`cargo install --git https://github.com/ninjamuffin99/hmm-rs hmm-rs`

## TODO List

- [ ] Install Haxelibs from Git
- [ ] Install haxelibs from haxelib (lib.haxe.org)
- [ ] Download to `.haxelib/` folder
- [ ] Organize code so it's not just main.rs lol!
- [ ] Github actions
  - [ ] Windows Build
  - [ ] Linux Build
  - [ ] Mac Build
  - [ ] Github Releases
- [x] Help info for commands
- [ ] Create tests against haxe `hmm`

### hmm commands to implement

- [ ] version - prints hmm version
- [ ] install - installs libraries listed in hmm.json
  - [ ] haxelib
  - [ ] git
  - [ ] check if version is already installed
- [ ] from-hxml
- [x] to-hxml
- [ ] reinstall
- [ ] setup
- [ ] haxelib
- [ ] git
- [ ] hg
- [ ] dev
- [ ] update
- [ ] remove
- [ ] lock
- [x] check
- [x] clean
