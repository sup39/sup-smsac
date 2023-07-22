# sup-smsac
A tool to support Super Mario Sunshine academic research and TAS.

It is written in Rust (backend) and JavaScript (frontend),
and uses HTTP + WebSocket to communicate between backend and frontend.

It only contains a simple Object Viewer at the moment.

## Usage
Download the binary from the [releases page](https://github.com/sup39/sup-smsac/releases). Unzip and double click `sup-smsac.exe`. It should open browser automatically for you. If it doesn't, open browser and navigate to the url shown in the terminal manually.

## Building from Source (Windows only)
Requirements:
- [cargo](https://www.rust-lang.org/tools/install)
- [Git Bash](https://git-scm.com/download/win)

```sh
# Clone the repository
git clone https://github.com/sup39/sup-smsac

# cd to the directory of the repository
cd sup-smsac

# run the build script
sh build.sh

# the out files will be in "out/sup-smsac-$version"
```

Note that if you are using `cargo run`, you have to pass `-d path/to/repository/directory` as argument to specify the path to the directory of the repository:
```
# assuming you are in the directory of the repository
cargo run -- -d .
```

## TODO
- [ ] documentation of the WebSocket API
- [ ] add more ObjectParameters files
- [ ] UI improvement
