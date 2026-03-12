mod bank_account;
use bank_account::BankAccount;
use std::io::{self, Write};

fn prompt(msg: &str) -> String {
    print!("{msg}");
    io::stdout().flush().ok();
    let mut s = String::new();
    io::stdin().read_line(&mut s).ok();
    s.trim().to_string()
}

fn parse_amount(s: &str) -> Option<f64> {
    s.trim().replace(',', ".").parse::<f64>().ok()
}

fn read_initial_balance() -> BankAccount {
    loop {
        let input = prompt("Enter initial balance (>= 0): ");
        if let Some(v) = parse_amount(&input) {
            return BankAccount::new(v);
        } else {
            println!("Invalid number. Try again.");
        }
    }
}

fn main() {
    let mut acc = read_initial_balance();
    // didn't know how to fully demostrate bank_account functions without a menu
    loop {  // using set values wouldn't have fully demostrated functions
        println!("\n=== Bank Account Menu ===");
        println!("1) Deposit");
        println!("2) Withdraw");
        println!("3) Check balance");
        println!("4) New account");
        println!("0) Exit");

        let choice = prompt("Choose an option: ");
        match choice.as_str() {
            "1" => {
                let amt_inp = prompt("Amount to deposit: ");
                if let Some(amount) = parse_amount(&amt_inp) {
                    let before = acc.balance();
                    acc.deposit(amount);
                    let after = acc.balance();
                    if (after - before).abs() < 1e-12 {
                        println!("Deposit ignored (amount must be > 0).");
                    } else {
                        println!("Deposited {:.2}. New balance: {:.2}", amount, after);
                    }
                } else {
                    println!("Invalid number. Try again.");
                }
            }
            "2" => {
                let amt_inp = prompt("Amount to withdraw: ");
                if let Some(amount) = parse_amount(&amt_inp) {
                    let before = acc.balance();
                    if amount <= 0.0 {
                        println!("Withdrawal ignored (amount must be > 0).");
                    } else if amount > before {
                        println!("Withdrawal ignored (cannot overdraw). Current balance: {:.2}", before);
                    } else {
                        acc.withdraw(amount);
                        println!("Withdrew {:.2}. New balance: {:.2}", amount, acc.balance());
                    }
                } else {
                    println!("Invalid number. Try again.");
                }
            }
            "3" => {
                println!("Current balance: {:.2}", acc.balance());
            }
            "4" => {
                acc = read_initial_balance();
                println!("Created new account. Balance: {:.2}", acc.balance());
            }
            "0" | "q" | "Q" => break,
            _ => println!("Invalid choice. Please select 0–4."),
        }
    }
}