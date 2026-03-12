#!/opt/homebrew/bin/python3

import argparse
import datetime
import subprocess
import json
import numpy as np
from concurrent.futures import ThreadPoolExecutor, as_completed
from tqdm import tqdm


def summarize(times):
    if len(times) == 0:
        return {}

    def f(t):
        return f"{datetime.timedelta(microseconds=t).total_seconds() * 1000:.5f}ms"

    arr = np.array(times)
    return {
        "count": len(arr),
        "mean": f(float(np.mean(arr))),
        "std": f(float(np.std(arr))),
        "min": f(float(np.min(arr))),
        "max": f(float(np.max(arr))),
        "median": f(float(np.median(arr))),
    }


def build_binary():
    print("Building latest release binary...")
    subprocess.run(["cargo", "build", "--release"], check=True)


def generate_seeds(n):
    print(f"Generating {n} seeds...")
    seeds = []
    for _ in range(n):
        seed = subprocess.run(
            ["../../target/release/cli", "random"],
            capture_output=True,
            text=True,
            check=True
        ).stdout
        seeds.append(seed)
    return seeds


def run_solver(method, seed):
    try:
        c = ["../../target/release/cli", "solve", method, "-", "-j"]
        proc = subprocess.run(
            c,
            input=seed,
            capture_output=True,
            text=True,
            timeout=20
        )
    except subprocess.TimeoutExpired:
        return ("timeout", None)
    except Exception:
        return ("error", None)

    try:
        result = json.loads(proc.stdout)
    except Exception as e:
        print("=========== Error on", c, ":", e, "===========")
        print("==seed==")
        print(seed.replace('\n', ' '))
        print("==stdout==")
        print(proc.stdout)
        print("==stderr==")
        print(proc.stderr)
        return ("error", None)

    t = int(result.get("time_micro"))
    solved = result.get("success")

    if solved:
        return ("success", t)
    else:
        return ("failure", t)


def main():
    parser = argparse.ArgumentParser(
        prog="bench",
        description="bench the solitaire solvers"
    )
    parser.add_argument("--method", default="greedy")
    parser.add_argument("--iterations", type=int, default=1000)
    parser.add_argument("--jobs", type=int, default=8)

    args = parser.parse_args()

    build_binary()

    seeds = generate_seeds(args.iterations)

    successes = 0
    failures = 0
    timeouts = 0

    all_times = []
    success_times = []
    failure_times = []

    print("Running solver...")
    with ThreadPoolExecutor(max_workers=args.jobs) as executor:
        futures = [
            executor.submit(run_solver, args.method, seed)
            for seed in seeds
        ]
        pbar = tqdm(total=len(futures), desc="Solving")

        for f in as_completed(futures):
            status, t = f.result()

            if status == "success":
                successes += 1
                all_times.append(t)
                success_times.append(t)
            elif status == "failure":
                failures += 1
                all_times.append(t)
                failure_times.append(t)
            elif status == "timeout":
                failures += 1
                timeouts += 1
            else:
                failures += 1

            pbar.set_postfix(success=successes, fail=failures)
            pbar.update(1)

        pbar.close()

    print("=== Benchmark Results ===")
    print(f"method: {args.method}")
    print(f"iterations: {args.iterations}")
    print(f"jobs: {args.jobs}")
    print()

    print(f"successes: {successes}")
    print(f"failures: {failures}")
    print(f"timeouts: {timeouts}")
    print(f"success rate: {successes / args.iterations:.3f}")
    print()

    print("overall time stats:")
    print(summarize(all_times))
    print()

    print("success time stats:")
    print(summarize(success_times))
    print()

    print("failure time stats:")
    print(summarize(failure_times))


if __name__ == "__main__":
    main()
