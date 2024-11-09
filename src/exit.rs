use std::process;

pub enum ExitCode {
    FileNotFound = 1,
    FailedToReadDirectory = 2,
    FailedToReadDirectoryEntry = 3,
    FailedToReadFile = 4,
}

pub fn die(message: &str, exit_code: ExitCode) -> ! {
    println!("{}", message);
    process::exit(exit_code as i32);
}
