use std::{
    env,
    fs::File,
    io::{self, Read},
    str::FromStr,
    time::{Duration, Instant},
};

use solitaire_game::{deck::Deck, kplus::KPlusSolitaire};
use solitaire_solver::{greedy::greedy_solve, Solution};

fn main() {
    let mut args = env::args();

    let Some(command) = args.nth(1) else {
        print_help();
        return;
    };

    match command.as_str() {
        "solve" => {
            let Some(method) = args.next() else {
                print_no_method();
                return;
            };
            let Some(path) = args.next() else {
                print_no_path();
                return;
            };
            let mut buf = String::new();
            if path.as_str() == "-" {
                io::stdin().read_to_string(&mut buf).expect("reading stdin");
            } else {
                let Ok(mut f) = File::open(&path) else {
                    print_path_not_found(&path);
                    return;
                };
                f.read_to_string(&mut buf).expect("reading file");
            }
            let json = matches!(args.next().as_deref(), Some("-j") | Some("--json"));
            solve(buf, method, json);
        }
        "random" => print_random(),
        _ => print_help(),
    }
}

fn solve(deck: String, method: String, json: bool) {
    let game = KPlusSolitaire::with_deck(Deck::from_str(&deck).unwrap());

    let (now, sol) = match method.to_lowercase().as_str() {
        "greedy" => {
            let now = Instant::now();
            (now, greedy_solve(game))
        }
        _ => {
            print_method_not_found();
            return;
        }
    };
    let elapsed = now.elapsed();

    if json {
        let j = solution_to_json(sol, elapsed);
        println!("{}", j);
    } else if let Some(sol) = sol {
        println!("Solution found in {:?}", elapsed);
        println!("{sol:?}");
    } else {
        println!("No solution found in {:?}", elapsed);
    }
}

fn solution_to_json(sol: Option<Solution>, elapsed: Duration) -> String {
    // ignore how actions are formatted for now
    format!(
        "{{
    \"success\": {},
    \"time_micro\": \"{}\",
    \"actions\": \"{:?}\"
}}",
        sol.is_some(),
        elapsed.as_micros(),
        sol.map(|s| s.moves).unwrap_or_default()
    )
}

fn print_help() {
    println!(
        "interface to solve solitaire puzzles on the command line and report statistics on solving"
    );
    println!("\tusage:\t{} <command> [opts]", env::args().next().unwrap());
    println!();
    println!("Available commands:");
    println!("\tsolve <method> <path> [-j | --json]: solve a puzzle located at <path> using <method> (use - for stdin) use -j for json structured output");
    println!("\t\tavailable methods: greedy");
    println!("\trandom: generate a random deck seed");
    println!("\thelp: print out this help message");
}

fn print_random() {
    let deck = Deck::new_shuffled();
    println!("{deck}");
}

fn print_no_path() {
    println!(
        "usage:\t{} solve <method> <path> [-j | --json]",
        env::args().next().unwrap()
    );
    println!("error: path is missing");
}

fn print_no_method() {
    println!(
        "usage:\t{} solve <method> <path> [-j | --json]",
        env::args().next().unwrap()
    );
    println!("error: method is missing");
}

fn print_method_not_found() {
    println!(
        "usage:\t{} solve <method> <path> [-j | --json]",
        env::args().next().unwrap()
    );
    println!("error: method is missing");
    println!("available methods: greedy");
}

fn print_path_not_found(path: &str) {
    println!(
        "usage:\t{} solve <path> [-j | --json]",
        env::args().next().unwrap()
    );
    println!("error: could not read file: {path}");
}
