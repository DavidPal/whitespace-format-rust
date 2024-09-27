use std::env;
use std::fs;
const FILE_NAME: &str = "README.md";

fn main() {
    // Print hello world.
    println!("Hello, world!");

    // Print command line arguments.
    let args: Vec<String> = env::args().collect();
    dbg!(args);

    // Print content of a file.
    let data: Vec<u8> = fs::read(&FILE_NAME).unwrap();
    dbg!(String::from_utf8_lossy(&data));
}
