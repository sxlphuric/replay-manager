<div align="center">

### Replay Manager

![ReplayManagerIcon](https://github.com/sxlphuric/replay-manager/blob/main/assets/icon_256.png?raw=true)

**An opinionated video browser**

*Built with Rust and egui for high performance*

![Stars](https://img.shields.io/github/stars/sxlphuric/replay-manager)
![Last Commit](https://img.shields.io/github/last-commit/sxlphuric/replay-manager)

[Installation](#Installation) · [Roadmap](#Roadmap) · [Structure](#Structure)

</div>

---

## Overview

> **⚠️ WARNING:**
> This is **not** a finished project. Expect **bugs** (especially on Windows) and **unpolishedness**.

The Replay Manager is an opinionated video browser built using **Rust** and **egui**.

### Key Features

- 🎞 **Automatic Thumbnail Generation** - The application autonomatically generates thumbnails using FFmpeg
- 💾 **Cloud Save** - Upload your files to Catbox or Litterbox for easy sharing
- 📈 **Video Edit** - Open replays in an editor of choice (default losslesscut)
- ⭐ **Favorites** - Favorite replays to find them easily

## Screenshots

<div align="center">
  <img width="810" height="725" alt="image" src="https://github.com/user-attachments/assets/965d7eb0-e76f-4ca7-a75b-a3e948229db5" />
</div>

## Installation
> **Note:**
> I'm assuming you already have Rust installed. If not, please [install it](https://rustup.rs/).

> **Note for MacOS users:**
> I'm assuming you already have Homebrew installed. If not, please [install it](https://brew.sh).

> **Note for Windows users:**
> The LosslessCut Winget package is unofficial. If you're unsure, you can grab it from [their github releases](https://github.com/mifi/lossless-cut/releases) or use [Chocolatey](https://chocolatey.org/install).

```fish
# Install dependencies

## Arch Linux
sudo pacman -S ffmpeg
yay -S losslesscut
## Ubuntu and derivatives
sudo apt-get install git ffmpeg
sudo snap install losslesscut
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

## Structure
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
|_ videoutils - simple functions that return info on a file when given a path
README.md - this file
LICENSE - license (gplv3)
Cargo.lock - idk
Cargo.toml - Cargo dependencies
```

## Roadmap
- [x] Optimize
- [ ] Polish
- [-] Saved replays
- [x] Keyboard shortcuts
- [ ] Catbox authentication
- [x] Thumbnail generation multithreading
- [x] Cross-platform support
  - [x] Linux
  - [x] MacOS
  - [ ] Windows (almost...)
- [ ] Logging with [tracing](https://docs.rs/tracing)
### TODO
- Catbox authentication with user token
- Litterbox fallback when file too big for catbox
- Possibly rewrite gpu-screen-recorder in rust
- Add side panel for catbox file send operations
- Change catbox uploads to allow multiple uploads at the same time
- Working tests
- Remove images from like the egui_extras cache (one loaded image is 1.6 mb, and when you have a large folder, it can add up quickly)
- Fix Windows catbox uploads failing and litterbox uploads completing but the link returns an empty video
- Add Saved replays : basically renaming BUT it also moves/clones the replay to a folder called "Saved" or "Favorites" or etc., where it can be easily found or accessed in the program (The UX is half baked, make it better)
- Cleaner path management  with glob patterns : instead of using format!, use the .join method
