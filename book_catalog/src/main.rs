use std::fs::File;
use std::io::{Write, BufRead, BufReader};

struct Book {
    title: String,
    author: String,
    year: u16,
}

fn save_books(books: &Vec<Book>, filename: &str) {
    let mut file = match File::create(filename) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Could not create {}: {}", filename, e);
            return;
        }
    };

    for b in books {
        if let Err(e) = writeln!(file, "{},{},{}", b.title, b.author, b.year) {
            eprintln!("Failed writing a line: {}", e);
            return;
        }
    }
}

fn load_books(filename: &str) -> Vec<Book> {
    let file = match File::open(filename) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Could not open {}: {}", filename, e);
            return Vec::new();
        }
    };

    let reader = BufReader::new(file);
    let mut books = Vec::new();

    for line_res in reader.lines() {
        match line_res {
            Ok(line) => {
                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                if parts.len() != 3 {
                    eprintln!("Skipping malformed line: {}", line);
                    continue;
                }
                let year = match parts[2].parse::<u16>() {
                    Ok(y) => y,
                    Err(_) => {
                        eprintln!("Invalid year on line: {}", line);
                        continue;
                    }
                };
                books.push(Book {
                    title: parts[0].to_string(),
                    author: parts[1].to_string(),
                    year,
                });
            }
            Err(e) => {
                eprintln!("Could not read a line: {}", e);
            }
        }
    }

    books
}

fn main() {
    let books = vec![
        Book { title: "1984".to_string(), author: "George Orwell".to_string(), year: 1949 },
        Book { title: "To Kill a Mockingbird".to_string(), author: "Harper Lee".to_string(), year: 1960 },
    ];

    save_books(&books, "books.txt");
    println!("Books saved to file.");

    let loaded_books = load_books("books.txt");
    println!("Loaded books:");
    for book in loaded_books {
        println!("\"{}\" by {}, published in {}", book.title, book.author, book.year);
    }
}