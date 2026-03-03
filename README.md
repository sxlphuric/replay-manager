# replay-manager
**⚠️ WARNING! This is not a finished project. Expect bugs and unpolishedness.**

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

## Roadmap
- Catbox authentication with user token
- Better search bar (make full screen width)
- Add settings for the replay folders and stuff
- Fix the grid so that it doesnt have only 3 rows and look weird on different screen resolutions
- Customize UI?
- Keyboard shortcuts
- Catbox upload popup button (Cancel and Ok)
- Make it so you can click or copy the link on the Catbox popup
- Implement a way to choose how to send files to Catbox and Add settings that allow you to choose which Catbox options (litter or normal, time, fallback to litter when file to big or not)
- Use the thumbnail or something crate/ ez-ffmpeg when ffmpeg-next is fixed (IF POSSIBLE)
- Possibly rewrite gpu-screen-recorder in rust :3
- Fix result error handling (It's absolutely horrible how expect is everywhere)
- Fix light mode
- More contrast between the top bar with the search bar and the ScrollView
- Add side panel for catbox file send operations
