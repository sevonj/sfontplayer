# SfontPlayer

![ci badge](https://github.com/sevonj/sfontplayer/actions/workflows/rust.yml/badge.svg)

![image](https://github.com/user-attachments/assets/cfc01da3-3ace-4b2e-a6cc-34c7ef213080)

A simple midi player that makes soundfont comparison easy.

## Features:

- **Changing between soundfonts takes one click**  
  This was the sole reason to start this project.
- **Workspaces**  
  Playlists can be set to automatically track the contents of a directory (and subdirectories).

## Advanced Features

<details>
<summary>Expand this if you're an advanced user</summary>

___
- **MIDI Inspector**  
  Examine MIDI files at event level

</details>

## Download

[➜ Go to Releases](https://github.com/sevonj/sfontplayer/releases)

Prebuilt binaries are available for Linux and Windows.

## Development

[➜ Project management](https://github.com/users/sevonj/projects/12)

Check out the linked project for an organized overview of issues.

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
