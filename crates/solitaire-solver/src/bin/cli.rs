use std::{
    env,
    fmt::Write,
    fs::File,
    io::{self, Read},
    str::FromStr,
    time::{Duration, Instant},
};

use solitaire_game::{deck::Deck, kplus::KPlusSolitaire};
use solitaire_solver::{
    greedy::greedy_solve,
    heuristic::{h1, h2},
    multistage_nested_rollout::multistage_rollout_solve,
    nested_rollout::nested_rollout_solve,
    Solution,
};

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
            let mut arg = args.next();
            let json = matches!(arg.as_deref(), Some("-j") | Some("--json"));
            let mut n = None;
            if method == "nested" {
                if json {
                    arg = args.next();
                }
                n = arg.as_deref().and_then(|s| {
                    s.split(',')
                        .map(|s| usize::from_str(s).ok())
                        .collect::<Option<Vec<usize>>>()
                });
            }
            solve(buf, method, json, n);
        }
        "random" => print_random(),
        "verify" => {
            let Some(deck_path) = args.next() else {
                print_no_path();
                return;
            };
            let Some(solution_path) = args.next() else {
                print_no_path();
                return;
            };
            let mut buf = String::new();
            let mut deck_buf = String::new();
            let mut solution_buf = String::new();
            if deck_path.as_str() == "-" && solution_path == "-" {
                io::stdin().read_to_string(&mut buf).expect("reading stdin");
                let Some((deck, sol)) = buf.split_once("\n\n") else {
                    println!("error: can't find deck and solution. should be split by blank line with deck first.");
                    return;
                };
                writeln!(&mut deck_buf, "{deck}").unwrap();
                solution_buf.push_str(sol);
            } else if deck_path.as_str() == "-" {
                io::stdin()
                    .read_to_string(&mut deck_buf)
                    .expect("reading stdin");
                let Ok(mut f) = File::open(&solution_path) else {
                    print_path_not_found(&solution_path);
                    return;
                };
                f.read_to_string(&mut solution_buf)
                    .expect("reading solution file");
            } else if solution_path == "-" {
                io::stdin()
                    .read_to_string(&mut solution_buf)
                    .expect("reading stdin");
                let Ok(mut f) = File::open(&deck_path) else {
                    print_path_not_found(&deck_path);
                    return;
                };
                f.read_to_string(&mut deck_buf).expect("reading deck file");
            } else {
                let Ok(mut f) = File::open(&deck_path) else {
                    print_path_not_found(&deck_path);
                    return;
                };
                f.read_to_string(&mut deck_buf).expect("reading deck file");
                let Ok(mut f) = File::open(&solution_path) else {
                    print_path_not_found(&solution_path);
                    return;
                };
                f.read_to_string(&mut solution_buf)
                    .expect("reading solution file");
            }
            verify(deck_buf, solution_buf);
        }
        _ => print_help(),
    }
}

fn solve(deck: String, method: String, json: bool, n: Option<Vec<usize>>) {
    let game = KPlusSolitaire::with_deck(Deck::from_str(&deck).unwrap());

    let (now, sol) = match method.to_lowercase().as_str() {
        "greedy" => {
            let now = Instant::now();
            (now, greedy_solve(game))
        }
        "nested" => {
            let now = Instant::now();
            (now, nested_rollout_solve(game, n.unwrap_or(vec![2])[0]))
        }
        "multistage" => {
            let now = Instant::now();
            (
                now,
                multistage_rollout_solve(
                    game,
                    &n.map(|n| [n[0], n[1]]).unwrap_or([2, 1]),
                    &[&h1, &h2],
                ),
            )
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

fn verify(deck_buf: String, solution_buf: String) {
    let mut game = KPlusSolitaire::with_deck(Deck::from_str(&deck_buf).unwrap());
    let solution: Solution = serde_json::from_str(&solution_buf).unwrap();

    for action in solution.moves {
        game.do_move(action);
    }

    if game.state.is_win() {
        println!("{{\"valid\": true, \"error\": \"\"}}");
    } else {
        println!("{{\"valid\": false, \"error\": \"{:?}\"}}", game.state);
    }
}

fn solution_to_json(sol: Option<Solution>, elapsed: Duration) -> String {
    format!(
        "{{
    \"success\": {},
    \"time_micro\": \"{}\",
    \"actions\": {}
}}",
        sol.is_some(),
        elapsed.as_micros(),
        sol.and_then(|s| serde_json::to_string(&s).ok())
            .unwrap_or_else(|| "{}".to_string())
    )
}

fn print_help() {
    println!(
        "interface to solve solitaire puzzles on the command line and report statistics on solving"
    );
    println!("\tusage:\t{} <command> [opts]", env::args().next().unwrap());
    println!();
    println!("Available commands:");
    println!("\tsolve <method> <path> [-j | --json] [n]: solve a puzzle located at <path> using <method> (use - for stdin) use -j for json structured output");
    println!("\t\tavailable methods: greedy, nested, multistage");
    println!("\t\tn: level of nesting for applicable solvers (comma separated list of length two for multistage)");
    println!("\tverify <path> <solution-path>: apply moves from to a state and verify if they solve the puzzle");
    println!("\trandom: generate a random deck seed");
    println!("\thelp: print out this help message");
}

fn print_random() {
    let deck = Deck::new_shuffled();
    println!("{deck}");
}

fn print_no_path() {
    println!("error: path is missing");
}

fn print_no_method() {
    println!(
        "usage:\t{} solve <method> <path> [-j | --json] [n]",
        env::args().next().unwrap()
    );
    println!("error: method is missing");
}

fn print_method_not_found() {
    println!(
        "usage:\t{} solve <method> <path> [-j | --json] [n]",
        env::args().next().unwrap()
    );
    println!("error: method is missing");
    println!("available methods: greedy, nested, multistage");
}

fn print_path_not_found(path: &str) {
    println!("error: could not read file: {path}");
}
