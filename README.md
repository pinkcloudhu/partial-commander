<div align="center">
  <h1>partial-commander</h1>
  <h3>A simple console based directory tree navigator</h3>
  <p>Aiming to be quick to change directories</p>
  <img src="https://i.imgur.com/0ukqCXR.png">
</div>

## Features
- Preview text files
- Traverse directories with arrow keys (or Enter/Backspace for down/up)
- Windows and Linux support
- *More soon*

## Usage
### Prebuilt portable binary
On the [releases](https://github.com/pinkcloudhu/partial-commander/releases) page

### Building from source
- [Set up](https://www.rust-lang.org/tools/install) your rust environment if you haven't already
- Clone, then build the project
```sh
git clone https://github.com/pinkcloudhu/partial-commander
cd partial-commander
cargo build --release
```
- On Windows, the built executable can be found in `target\release\pc.exe`,
  on Linux and alike `target/release/pc`.