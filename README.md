# replay-manager
**⚠️ WARNING! This is not a finished project. Expect bugs and unpolishedness.**

This is the Replay Manager.
It can go in a folder you specify, and will list all the files with a specific file extension. It even creates thumbnails for them with the custom thumbnail script. Furthermore, you can share files to Catbox (litter) and open videos in the Losslesscut video editor.

<img width="810" height="725" alt="image" src="https://github.com/user-attachments/assets/965d7eb0-e76f-4ca7-a75b-a3e948229db5" />

## Project Structure
```
/
.github/workflows - workflows
|_ rust.yml - workflow for Cargo tests
test - files for test workflows
|_ bounce.webm - test video for thumbnail test
assets - program assets
|_ icon_256.png - program icon (256x256px)
src
|_ app.rs - the egui app
|_ main.rs - the main rust code that just launches the egui app
|_ thumbnails.rs - generates thumbnails
|_ videoutils - simple functions that return info on a file when given a pathbuf
README.md - this file
LICENSE - license (gplv3)
Cargo.lock - idk
Cargo.toml - Cargo dependencies
```

## Installation
I'm assuming you already have Rust installed. If not, please [install it](https://rustup.rs/).

**Notice for MacOS users:** I'm assuming you already have Homebrew installed. If not, please [install it](https://brew.sh).

**Notice for Windows users:** The LosslessCut Winget package is unofficial. If you're unsure, you can [use Chocolatey](https://chocolatey.org/install) or grab it from [their github releases](https://github.com/mifi/lossless-cut/releases).

```bash
# Install dependencies

## Arch Linux
sudo pacman -S ffmpeg
yay -S losslesscut
## MacOS
brew install losslesscut ffmpeg
## Windows (Winget)
winget install ch.LosslessCut Gyan.FFmpeg Git.Git
## Windows (Chocolatey)
choco install losslesscut ffmpeg git

# Clone the repo
git clone https://github.com/sxlphuric/replay-manager

# Go into the folder
cd replay-manager

# Install with cargo
cargo install --path .
```

## TODO
- Catbox authentication with user token
- Litterbox fallback when file too big for catbox
- Possibly rewrite gpu-screen-recorder in rust :3
- Add side panel for catbox file send operations
- Change catbox uploads to allow multiple uploads at the same time
- Working tests
- Remove images from like the egui_extras cache (one loaded image is 1.6 mb, and when you have a large folder, it can add up quickly)
- Fix Windows catbox uploads failing and litterbox uploads completing but the link returns an empty video
- Add a keyboard shortcut to refresh replays view
## Roadmap
- [x] Optimize
- [ ] Polish
- [x] Keyboard shortcuts
- [ ] Catbox authentication
- [x] Thumbnail generation multithreading
- [x] Cross-platform support
  - [x] Linux
  - [x] MacOS
  - [ ] Windows (almost...)
- [ ] Logging with [tracing](https://docs.rs/tracing)
