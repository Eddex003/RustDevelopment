#[derive(Debug)]
pub struct BankAccount {
    balance: f64,
}

impl BankAccount {
    pub fn new(initial_balance: f64) -> BankAccount {
        BankAccount {
            balance: initial_balance.max(0.0),
        }
    }

    pub fn deposit(&mut self, amount: f64) {
        if amount > 0.0 {
            self.balance += amount;
        }
    }

    pub fn withdraw(&mut self, amount: f64) {
        if amount > 0.0 && amount <= self.balance {
            self.balance -= amount;
        }
    }

    pub fn balance(&self) -> f64 {
        self.balance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-10;
    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPS
    }

    #[test]
    fn test_new_account() { // Creating a new account
        let acc = BankAccount::new(100.0);
        assert!(approx_eq(acc.balance(), 100.0));
    }

    #[test]
    fn test_deposit() { // Depositing Money
        let mut acc = BankAccount::new(50.0);
        acc.deposit(25.0);
        assert!(approx_eq(acc.balance(), 75.0));
    }

    #[test]
    fn test_withdraw() { // Withdrawing money
        let mut acc = BankAccount::new(100.0);
        acc.withdraw(40.0);
        assert!(approx_eq(acc.balance(), 60.0));
    }

    #[test]
    fn test_balance() { // Checking the balance
        let mut acc = BankAccount::new(10.0);
        acc.deposit(5.0);
        acc.withdraw(3.0);
        assert!(approx_eq(acc.balance(), 12.0));
    }

    #[test]
    fn test_edge_cases() { // Edge cases (e.g., depositing/withdrawing negative amounts, withdrawing more than the balance)
        let mut acc = BankAccount::new(-20.0);
        assert!(approx_eq(acc.balance(), 0.0));
        acc.deposit(-5.0);
        assert!(approx_eq(acc.balance(), 0.0));
        acc.withdraw(-2.0);
        assert!(approx_eq(acc.balance(), 0.0));
        acc.deposit(10.0);
        acc.withdraw(20.0);
        assert!(approx_eq(acc.balance(), 10.0));
    }
}