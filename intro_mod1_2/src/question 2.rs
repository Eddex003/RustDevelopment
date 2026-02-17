// Assignment 2: Number Analyzer
fn is_even(n: i32) -> bool {
    n % 2 == 0
}

fn main() {
    let nums: [i32; 10] = [3, 5, 8, 15, 22, 1, 30, 4, 9, 11];

    for &n in nums.iter() {
        let label: &str = if n % 15 == 0 {
            "FizzBuzz"
        } else if n % 3 == 0 {
            "Fizz"
        } else if n % 5 == 0 {
            "Buzz"
        } else if is_even(n) {
            "even"
        } else {
            "odd"
        };
        println!("{n} -> {label}");
    }

    let mut i = 0usize;
    let mut sum = 0i32;
    while i < nums.len() {
        sum += nums[i];
        i += 1;
    }
    println!("sum = {sum}");

    let mut max = nums[0];
    for &n in nums.iter().skip(1) {
        if n > max {
            max = n;
        }
    }
    println!("max = {max}");
}