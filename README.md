# solitaire-solver

Implementation of Klondike Solitaire and accompanying solvers. Contains an application for playing solitaire and viewing the solvers. Follows Bjarnason's SEARCHING SOLITAIRE IN REAL TIME.

## Solitaire

In `crates/solitaire-game` there's an implementation of klondike solitaire as a library. It's a three draw mode. There's a GUI game version using [`macroquad`](https://crates.io/crates/macroquad),
You can load and save deck shufflings to replay games. It also has undo so you can try different ways to complete the game. It lets you test out what the solvers do (currently it will
look weird since the solvers use a sorted tableau).

## Solvers

The solvers are following [SEARCHING SOLITAIRE IN REAL TIME (Ronald Bjarnason Prasad Tadepalli Alan Fern)](https://web.engr.oregonstate.edu/~afern/papers/solitaire.pdf)
with greedy, nested rollout and multi stage rollout. You can test them using the bin in `solitaire-solver`, using the gui, or `bench.py`.

I've done my best to follow the paper, but my implementation is certainly not the best. The implementation differs for multiple reasons that can be described
as missing features (mentioned in the paper but not implemented), implementation differences (in the paper, but done differently) and extra features (not in
the paper).

### Missing features:
- Pruning (4.1); this uses the relaxed version of the game, but I have no idea how I'd implement that without rewriting my state representation
- Foundation Progression (4.4)
- Action ordering (4.4); the action ordering of my solvers doesn't match but this is on the TODO list
- Local loop prevention (4.4); my implementation will happily shuffle all the cards around before progressing. I'm not sure how I'd implement this.

### Implementation Differences:
- Action generation

### Extra Features:
- Sorted tableau stacks; my solver sorts the tableau stacks to collapse the search space

## Bench

There's a python script `crates/solitaire-solver/bench.py` which can use the `solitaire-solver` bin to test solvers using different configurations (such as nesting level) on
randomly generated shufflings to test solver effictiveness. You can limit solvers to different timeouts and limit the jobs running (it runs multiple solvers at once to speed things up).
The script then reports successes/failures and some summary stats on those (time taken, moves to solve, etc.).

```shell
# example run
python3 bench.py --method multistage --jobs 7 --timeout 300 --nest 3,1
# will run 1000 tests on 7 threads using the multistage solver with nesting
# levels of 3 and 1 for h1, h2 respectively
# will then report on stats for this run
```

## Profiling

You can also profile the program using [`samply`](https://crates.io/crates/samply) with e.g.:

```shell
# run in crates/solitaire-solver to test the solver using the nested method on decks/t shuffling
cargo b --bin cli --profile=profiling
samply record ../../target/profiling/cli solve nested ../../decks/t -j 2
```

The `profile.tar.gz` that are generated are already in the `.gitignore`.
