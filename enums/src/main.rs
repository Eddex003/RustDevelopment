use std::io::{self, Write};
use std::process::{Command};

enum FileOperation {
    List(String),                     // Directory path
    Display(String),                  // File path
    Create { path: String, content: String }, // File path and content
    Remove(String),                   // File path
    Pwd,                              // Print working directory
}

fn main() {
    println!("Welcome to the File Operations Program!");

    loop {
        print_menu();

        match read_choice("Enter your choice (0-5): ") {
            Some(0) => {
                println!("\nGoodbye!");
                break;
            }
            Some(1) => {
                let dir = prompt("Enter directory path: ");
                perform_operation(FileOperation::List(dir));
            }
            Some(2) => {
                let path = prompt("Enter file path: ");
                perform_operation(FileOperation::Display(path));
            }
            Some(3) => {
                let path = prompt("Enter file path: ");
                let content = prompt("Enter content: ");
                perform_operation(FileOperation::Create { path, content });
            }
            Some(4) => {
                let path = prompt("Enter file path: ");
                perform_operation(FileOperation::Remove(path));
            }
            Some(5) => {
                perform_operation(FileOperation::Pwd);
            }
            _ => {
                eprintln!("Invalid choice. Please enter a number from 0 to 5.");
            }
        }
        println!(); 
    }
}

fn print_menu() {
    println!();
    println!("File Operations Menu:");
    println!("1. List files in a directory");
    println!("2. Display file contents");
    println!("3. Create a new file");
    println!("4. Remove a file");
    println!("5. Print working directory");
    println!("0. Exit");
    println!();
}

fn prompt(msg: &str) -> String {
    print!("{msg}");
    io::stdout().flush().expect("failed to flush stdout");
    let mut s = String::new();
    io::stdin().read_line(&mut s).expect("failed to read line");
    s.trim().to_string()
}

fn read_choice(msg: &str) -> Option<u32> {
    let s = prompt(msg);
    s.parse::<u32>().ok()
}

fn perform_operation(op: FileOperation) {
    match op {
        FileOperation::List(dir) => {
            // ls -la <dir>
            let output = Command::new("ls")
                .arg("-la")
                .arg(dir)
                .output();

            match output {
                Ok(out) => {
                    if out.status.success() {
                        print_output(&out.stdout);
                    } else {
                        eprintln!("Failed to execute ls");
                        print_error(&out.stderr);
                    }
                }
                Err(e) => eprintln!("Error starting ls: {e}"),
            }
        }

        FileOperation::Display(path) => {
            // cat <file>
            let output = Command::new("cat")
                .arg(path)
                .output();

            match output {
                Ok(out) => {
                    if out.status.success() {
                        print_output(&out.stdout);
                    } else {
                        eprintln!("Failed to execute cat");
                        print_error(&out.stderr);
                    }
                }
                Err(e) => eprintln!("Error starting cat: {e}"),
            }
        }

        FileOperation::Create { path, content } => {
            let content_escaped = sh_single_quote_escape(&content);
            let path_escaped = sh_single_quote_escape(&path);

            let user_cmd = format!("printf '%s' '{}' > '{}'", content_escaped, path_escaped);

            let output = Command::new("sh")
                .arg("-c")
                .arg(user_cmd)
                .output();

            match output {
                Ok(out) => {
                    if out.status.success() {
                        println!("File '{}' created successfully.", path);
                    } else {
                        eprintln!("Failed to create file '{}'.", path);
                        print_error(&out.stderr);
                    }
                }
                Err(e) => eprintln!("Error starting shell to create file: {e}"),
            }
        }

        FileOperation::Remove(path) => {
            // rm <file>
            let output = Command::new("rm")
                .arg(path.clone())
                .output();

            match output {
                Ok(out) => {
                    if out.status.success() {
                        println!("File '{}' removed successfully.", path);
                    } else {
                        eprintln!("Failed to remove file '{}'.", path);
                        print_error(&out.stderr);
                    }
                }
                Err(e) => eprintln!("Error starting rm: {e}"),
            }
        }

        FileOperation::Pwd => {
            // pwd
            let output = Command::new("pwd").output();

            match output {
                Ok(out) => {
                    if out.status.success() {
                        print!("Current working directory: ");
                        print_output(&out.stdout);
                    } else {
                        eprintln!("Failed to execute pwd");
                        print_error(&out.stderr);
                    }
                }
                Err(e) => eprintln!("Error starting pwd: {e}"),
            }
        }
    }
}

fn print_output(bytes: &[u8]) {
    if !bytes.is_empty() {
        println!("{}", String::from_utf8_lossy(bytes));
    }
}

fn print_error(bytes: &[u8]) {
    if !bytes.is_empty() {
        eprintln!("{}", String::from_utf8_lossy(bytes));
    }
}


fn sh_single_quote_escape(s: &str) -> String {
    s.replace('\'', "'\"'\"'")
}