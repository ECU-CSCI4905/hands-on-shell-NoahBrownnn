use std::io::{self, Write};
use std::process::{Command, Stdio, Child};
use std::env;
use std::path::Path;

fn main() {
    // Set platform-specific PATH separator
    let path_separator: &str = if cfg!(target_os = "windows") { ";" } else { ":" };

    // Set initial custom PATH based on OS
    let mut current_path: String = if cfg!(target_os = "windows") {
        "C:/Windows/System32;C:/Windows;C:/Program Files/Git/cmd;C:/Program Files/Git/GitHub CLI;%USERPROFILE%/.cargo/bin".to_string()
    } else {
        "/usr/local/sbin:/usr/local/bin:/usr/bin:/bin:/sbin:/usr/sbin:/sbin/bin".to_string()
    };

    println!("The env var PATH separator is {path_separator}");
    println!("The env var PATH is {current_path}");

    loop {
        print!("my shell > ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        // Handle piping using `|` split
        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command: Option<Child> = None;

        while let Some(command) = commands.next() {
            let mut parts = command.trim().split_whitespace();
            let command = parts.next().unwrap();
            let args: Vec<&str> = parts.collect();

            match command {
                "cd" => {
                    let new_dir: &str = args.first().copied().unwrap_or("/");
                    let new_current_dir = Path::new(new_dir);
                    if let Err(e) = env::set_current_dir(&new_current_dir) {
                        eprintln!("cd error: {e}");
                    }
                    previous_command = None;
                }

                "showpath" => {
                    println!("Current PATH contains: {current_path}");
                    previous_command = None;
                }

                "addpath" => {
                    let new_path_part: &str = args.first().copied().unwrap_or("");
                    if new_path_part.is_empty() {
                        eprintln!("Usage: addpath <path>");
                    } else {
                        current_path.push_str(&format!("{path_separator}{new_path_part}"));
                    }
                    previous_command = None;
                }

                "exit" => return,

                command => {
                    // Set up stdin from previous command (if piped)
                    let stdin = previous_command
                        .as_mut()
                        .map(|output| Stdio::from(output.stdout.take().unwrap()))
                        .unwrap_or(Stdio::inherit());

                    // If another command follows, pipe the output; else, inherit stdout
                    let stdout = if commands.peek().is_some() {
                        Stdio::piped()
                    } else {
                        Stdio::inherit()
                    };

                    // Spawn the command
                    let output = Command::new(command)
                        .args(&args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .env("PATH", current_path.clone())
                        .spawn();

                    // Handle success/failure of command
                    match output {
                        Ok(child) => previous_command = Some(child),
                        Err(e) => {
                            eprintln!("Error: {e}");
                            previous_command = None;
                        }
                    }
                }
            }
        }

        // Wait for final piped command before prompt
        if let Some(mut final_command) = previous_command {
            let _ = final_command.wait();
        }
    }
}
