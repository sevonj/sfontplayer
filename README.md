# SfontPlayer
![ci badge](https://github.com/sevonj/sfontplayer/actions/workflows/rust.yml/badge.svg)

![image](https://github.com/user-attachments/assets/75bdb581-f072-4c62-ad05-362e40c4125f)

A simple midi player that makes soundfont comparison easy.

## Main features:
- **Changing between soundfonts takes one click**  
  This was my sole reason to start this project.
- **Multiple workspaces**  
  A workspace can automatically track contents of a directory, or be used as a playlist.

## Download
[➜ Go to Releases](https://github.com/sevonj/sfontplayer/releases)

Prebuilt binaries are available for Linux and Windows.

## Development

Check out the [linked project](https://github.com/users/sevonj/projects/12) for an overview of issues.

### Build
- Clone the repo
- Install [Rust](https://www.rust-lang.org/) if you don't have it already. Linux users may also may find it in the native package manager.
- Run `cargo build` at repository root. [read more](https://doc.rust-lang.org/cargo/commands/cargo-build.html)
- Get your executable from `target/<yourtarget>/`

### Continuous Integration
Pull requests are gatekept by [this workflow.](https://github.com/sevonj/sfontplayer/blob/master/.github/workflows/rust.yml) It will check if the code
- builds all targets
- passes unit tests (run `cargo test`)
- has linter warnings (run `cargo clippy`)
- is formatted (run `cargo fmt`)
