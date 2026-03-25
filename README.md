<div align="center">

### Replay Manager

![ReplayManagerIcon](https://github.com/sxlphuric/replay-manager/blob/main/assets/icon_256.png?raw=true)

**An opinionated video browser**

*Built with Rust and egui for high performance*

![Version](https://img.shields.io/badge/version-0.1.0-yellow)
![Commit Activity](https://img.shields.io/github/commit-activity/m/sxlphuric/replay-manager?style=flat)
![CI](https://img.shields.io/github/actions/workflow/status/sxlphuric/replay-manager/rust.yml?label=CI&style=flat)
![Stars](https://img.shields.io/github/stars/sxlphuric/replay-manager?style=flat)

[Installation](#Building) • [Roadmap](#Roadmap) • [Structure](#Structure)

</div>

---

## Overview

> **⚠️ WARNING:**
> This is **not** a finished project. Expect **bugs** (especially on Windows) and **unpolishedness**.

The Replay Manager is an opinionated video browser built using **Rust** and **egui**. It's goal is to *somewhat* replicate **Medal**'s features (excluding the social part)

### Key Features

- 🎞 **Automatic Thumbnail Generation** - The application autonomatically generates thumbnails using FFmpeg
- 💾 **Cloud Save** - Upload your files to Catbox or Litterbox for easy sharing
- 📈 **Video Edit** - Open replays in an editor of choice (default losslesscut)
- ⭐ **Favorites** - Favorite replays to find them easily

## Screenshots

<div align="center">
  <img width="810" height="725" alt="image" src="https://github.com/user-attachments/assets/965d7eb0-e76f-4ca7-a75b-a3e948229db5" />
</div>

---

## Dependencies

- [FFmpeg](https://git.ffmpeg.org/ffmpeg/)
- [Rust](https://rustup.rs/) 1.94.0 or later
- [LosslessCut](https://github.com/mifi/lossless-cut/) **(optional)**

## Building


This is currently the only way to install the Replay Manager.

#### Support
[![Linux](https://img.shields.io/badge/Linux-FCC624?logo=linux&logoColor=black&style=for-the-badge)](#) [![macOS](https://img.shields.io/badge/macOS-000000?logo=apple&logoColor=F0F0F0&style=for-the-badge)](#) [![Windows](https://custom-icon-badges.demolab.com/badge/Windows-0078D6?logo=windows11&logoColor=white&style=for-the-badge)](#) (partial)

### 1. Install dependencies

Install all the dependencies listed above.
These commands will also install `git` on your machine.

On **Windows**:
> **Note:**
> The LosslessCut Winget package is unofficial. If you're unsure, you can grab it from [their github releases](https://github.com/mifi/lossless-cut/releases) or use [Chocolatey](https://chocolatey.org/install).

`winget install ch.LosslessCut Gyan.FFmpeg Git.Git`

*or*

`choco install losslesscut ffmpeg git`

if you prefer [Chocolatey](https://chocolatey.org/install).


On **MacOS**:
> **Note:**
> I'm assuming you already have Homebrew installed. If not, please [install it](https://brew.sh).


`brew install losslesscut ffmpeg`

On **Linux**:

Use your package manager to install `ffmpeg` and `git`.

- On **Arch Linux and derivatives**: Use `yay` or your preferred AUR helper to install LosslessCut (package `losslesscut`)

- On **other distros**: Use [**snap**](https://snapcraft.io) to install LosslessCut (package `losslesscut`)

### 2. Cloning the repository

Clone the repository to your local machine. This can be done with

```fish
# Clone the repository
git clone https://github.com/sxlphuric/replay-manager.git

# Go into the repository's folder
cd replay-manager
```

### 3. Building

Build and install the package with `cargo`:

`cargo install --path .`

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
- [x] Saved replays
  - [ ] Better UX
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
