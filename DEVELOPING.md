# Setting up the development environment

Install Rust compiler and Cargo package manager for Rust. The easiest way to do
it is to install `rustup` by following the instructions at https://rustup.rs/.

Verify that `rustc` and `cargo` are installed properly by running the following
two commands:
```shell
rustc --version
cargo --version
```

## Compiling from source

Compile the code with
```shell
cargo build --release
```
The compiled executable will be placed in `./target/release/` directory. The
executable file will be called `whitespace-format`.

## Building Debian package

First, install `cargo-deb` extension by running the command:
```shell
cargo install cargo-deb
```

Build the Debian package by running:
```shell
cargo clean
cargo deb
```
The Debian package will be placed in `./target/debian/` directory. The file
name will end with the `.deb` extension.

### Installing and uninstalling the Debian package locally

Assuming you are running on Debian-based Linux distribution (Debian, Ubuntu,
Mint), you can install and uninstall the Debian package.

Install the package with the command run:
```shell
sudo dpkg --install ./target/debian/whitespace-format*.deb
```

Verify that the package was correctly installed by running:
```shell
whitespace-format --help
which whitespace
```
The output of the second command should be `/usr/bin/whitespace-format`.

Uninstall the package with the command:
```shell
sudo dpkg --remove whitespace-format
```
