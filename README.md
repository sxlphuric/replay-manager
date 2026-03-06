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
**Notice for MacOS users:** I'm assuming you already have Homebrew installed. If not, please [install it](https://brew.sh)

**⚠️ WARNING! The Winget package of LosslessCut is not affiliated or endorsed by the original developers. If you are unsure, you can download LosslessCut from [their github releases](https://github.com/mifi/lossless-cut/releases).**

```bash
# Install dependencies

## Arch Linux
sudo pacman -S ffmpeg
yay -S losslesscut
## MacOS
brew install losslesscut ffmpeg
## Windows
winget install ch.LosslessCut Gyan.FFmpeg

# Clone the repo
git clone https://github.com/sxlphuric/replay-manager

# Go into the folder
cd replay-manager

# Install with cargo
cargo install --path .
```

## TODO
- Catbox authentication with user token
- Better search bar (make full screen width)
- Fix the grid so that it resizes properly
- Customize UI?
- Keyboard shortcuts
- Litterbox fallback when file too big for catbox
- Possibly rewrite gpu-screen-recorder in rust :3
- Fix light mode
- Add side panel for catbox file send operations
- Optimize, the app uses around 7% cpu at rest
- Change catbox uploads to allow multiple uploads at the same time
- Working tests
- Use a toast instead of a scary modal to show errors
- Add whether to hide hidden files as an option
- Find a way to make the toasts not stay on the screen (put them outside of the for loop?)
## Roadmap
- [ ] Fix light mode
- [ ] Optimize
- [ ] Polish
- [ ] Keyboard shortcuts
- [ ] Catbox authentication
- [x] Thumbnail generation multithreading
- [ ] Cross-platform support
  - [x] Linux
  - [x] MacOS
  - [ ] Windows
- [ ] Logging with [tracing](https://docs.rs/tracing)
