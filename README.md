# SfontPlayer
![ci badge](https://github.com/sevonj/sfontplayer/actions/workflows/rust.yml/badge.svg)

![image](https://github.com/user-attachments/assets/e8a97024-355a-48f1-b7a8-cccafeb42afc)

A simple midi player that makes soundfont comparison easy.

## Main features:
- Changing between soundfonts takes one click
- Multiple workspaces

## Download
Go to [Releases](https://github.com/sevonj/sfontplayer/releases)  
Prebuilt binaries are available for Linux and Windows.

## Build
- Clone the repo
- Install [Rust](https://www.rust-lang.org/) if you don't have it already. Linux users may also may find it in the native package manager.
- Run `cargo build` at repository root. [read more](https://doc.rust-lang.org/cargo/commands/cargo-build.html)
- Get your executable from `target/<yourtarget>/`

## Continuous Integration
Pull requests are gatekept by [this action.](https://github.com/sevonj/sfontplayer/blob/master/.github/workflows/rust.yml) It will check if the code
- builds
- is formatted (run `cargo fmt`)
- has linter errors (run `cargo clippy`)