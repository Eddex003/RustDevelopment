// Assignment 1: Temperature Converter
const FP: f64 = 32.0;

fn fahrenheit_to_celsius(f: f64) -> f64 {
    (f - 32.0) * 5.0 / 9.0
}

fn celsius_to_fahrenheit(c: f64) -> f64 {
    c * 9.0 / 5.0 + 32.0
}

fn main() {
    let mut temp_f: f64 = FP;
    let temp_c = fahrenheit_to_celsius(temp_f);
    println!("{:.0}°F = {:.2}°C", temp_f, temp_c);

    for offset in 1..=5 {
        temp_f = FP + offset as f64;
        let c = fahrenheit_to_celsius(temp_f);
        println!("{:.0}°F = {:.2}°C", temp_f, c);
    }
}