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

## How to install
```bash
# Install dependencies
yay -S losslesscut

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
- Catbox upload popup button (Cancel and Ok)
- Litterbox fallback when file too big for catbox
- Use the thumbnail or something crate/ ez-ffmpeg when ffmpeg-next is fixed (IF POSSIBLE)
- Possibly rewrite gpu-screen-recorder in rust :3
- Fix light mode
- More contrast between the top bar with the search bar and the ScrollView
- Add side panel for catbox file send operations
- Optimize, the app seems to use alot of cpu while multithreading
- Add multiple file display modes (list, grid etc)
- Change catbox uploads to allow multiple uploads at the same time
- Add icon
- Working tests
- Fix the inputs in settings not being aligned
- Use a toast instead of a scary modal to show errors
## Roadmap
- [] Fix light mode
- [] Optimize
- [] Polish
- [] Keyboard shortcuts
- [] Catbox authentication
- [] Thumbnail generation multithreading
- [] Cross-platform support
