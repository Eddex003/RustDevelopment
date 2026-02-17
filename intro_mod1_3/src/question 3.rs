// Assignment 3: Guessing Game
fn check_guess(guess: i32, secret: i32) -> i32 {
    if guess == secret {
        0
    } else if guess > secret {
        1
    } else {
        -1
    }
}

fn main() {
    let mut secret: i32 = 44;
    let guesses: [i32; 6] = [10, 20, 30, 40, 50, 60];
    let mut attempts: i32 = 0;
    let mut last_guess: i32 = guesses[0];

    'game: loop {
        for &g in guesses.iter() {
            last_guess = g;
            attempts += 1;
            let res = check_guess(g, secret);
            if res == 0 {
                println!("Guess #{attempts}: {g} -> correct");
                break 'game;
            } else if res > 0 {
                println!("Guess #{attempts}: {g} -> too high");
            } else {
                println!("Guess #{attempts}: {g} -> too low");
            }
        }

        while last_guess != secret {
            last_guess += if last_guess < secret { 1 } else { -1 };
            attempts += 1;
            let res = check_guess(last_guess, secret);
            if res == 0 {
                println!("Guess #{attempts}: {last_guess} -> correct");
                break 'game;
            } else if res > 0 {
                println!("Guess #{attempts}: {last_guess} -> too high");
            } else {
                println!("Guess #{attempts}: {last_guess} -> too low");
            }
        }
        break 'game;
    }
    println!("It took {attempts} guess(es).");
}