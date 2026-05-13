# solitaire-solver

Implementation of Klondike Solitaire and accompanying solvers. Contains an application for playing solitaire and viewing the solvers (WIP). Generally follows Bjarnason's SEARCHING SOLITAIRE IN REAL TIME.

## Solitaire

In `crates/solitaire-game` there's an implementation of klondike solitaire as a library. It's a three draw mode. There's a GUI game version using [`macroquad`](https://crates.io/crates/macroquad),
You can load and save deck shufflings to replay games. It also has undo so you can try different ways to complete the game.

## Solvers

So far the solvers are following [SEARCHING SOLITAIRE IN REAL TIME (Ronald Bjarnason Prasad Tadepalli Alan Fern)](https://web.engr.oregonstate.edu/~afern/papers/solitaire.pdf)
with greedy, nested rollout and multi stage rollout. You can test them using the bin in `solitaire-solver`. There's an in progress way to see what the solvers are doing in the
game GUI.

As a side note, I've built the solvers so terribly while trying to follow the paper, that they both perform worse and solve less than the source material. This is probably from
my poor understanding of the algorithms, optimizations and poorly optimized implementation (of both solitaire and solving). Help with these would be appreciated!

## Bench

There's a python script `crates/solitaire-solver/bench.py` which can use the `solitaire-solver` bin to test solvers using different configurations (such as nesting level) on
randomly generated shufflings to test solver effictiveness. You can limit solvers to different timeouts and limit the jobs running (it runs multiple solvers at once to speed things up).
The script then reports successes/failures and some summary stats on those (time taken, moves to solve, etc.).

## Profiling

You can also profile the program using [`samply`](https://crates.io/crates/samply) with e.g.:

```shell
# run in crates/solitaire-solver to test the solver using the nested method on decks/t shuffling
samply record ../../target/release/cli solve nested ../../decks/t -j 2
```

The `profile.tar.gz` that are generated are already in the `.gitignore`.
