#!/opt/homebrew/bin/python3

import argparse
import datetime
import math
import subprocess
import json
import numpy as np
from concurrent.futures import ThreadPoolExecutor, as_completed
from tqdm import tqdm


def win_rate_ci(successes, n, z=1.96):
    # use Wilson score just because
    if n == 0:
        return (0.0, 0.0, 0.0)

    p = successes / n

    denom = 1 + z**2 / n
    center = (p + z**2 / (2*n)) / denom
    margin = (z * math.sqrt((p*(1-p) + z**2/(4*n)) / n)) / denom

    lower = center - margin
    upper = center + margin

    return p, lower, upper


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


def verify_solution(seed, actions):
    try:
        # prepare stdin: seed + blank line + solution json
        verify_input = seed.strip() + "\n\n" + json.dumps(actions, indent=4)

        proc = subprocess.run(
            ["../../target/release/cli", "verify", "-", "-"],
            input=verify_input,
            capture_output=True,
            text=True
        )
    except subprocess.TimeoutExpired:
        return (False, "verify timeout")
    except Exception as e:
        return (False, str(e))

    try:
        result = json.loads(proc.stdout)
    except Exception:
        return (False, f"bad verify output: {proc.stdout}\n{proc.stderr}")

    valid = result.get("valid", False)
    error = result.get("error", "")

    return (valid, error)


def run_solver(method, seed, n, timeout):
    try:
        c = ["../../target/release/cli", "solve", method, "-", "-j", str(n)]
        proc = subprocess.run(
            c,
            input=seed,
            capture_output=True,
            text=True,
            timeout=timeout if timeout > 0 else None
        )
    except subprocess.TimeoutExpired:
        return ("timeout", None, None, seed)
    except Exception as e:
        return (f"error: {e}", None, None, seed)

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
        return ("error", None, None, seed)

    t = int(result.get("time_micro"))
    solved = result.get("success")
    actions = result.get("actions")

    if solved:
        return ("success", t, actions, seed)
    else:
        return ("failure", t, None, seed)


def main():
    parser = argparse.ArgumentParser(
        prog="bench",
        description="bench the solitaire solvers"
    )
    parser.add_argument("--method", default="nested")
    parser.add_argument("--iterations", type=int, default=1000)
    parser.add_argument("--jobs", type=int, default=8)
    parser.add_argument("--nest", type=int, default=2)
    parser.add_argument("--timeout", type=int, default=600, help="pass 0 for no timeout (seconds)")

    args = parser.parse_args()

    build_binary()

    seeds = generate_seeds(args.iterations)

    successes = 0
    failures = 0
    timeouts = 0
    verified_successes = 0
    invalid_solutions = 0

    all_times = []
    success_times = []
    failure_times = []

    print("Running solver...")
    with ThreadPoolExecutor(max_workers=args.jobs) as executor:
        futures = [
            executor.submit(run_solver, args.method, seed, args.nest, args.timeout)
            for seed in seeds
        ]
        pbar = tqdm(total=len(futures), desc="Solving")

        try:
            for f in as_completed(futures):
                status, t, actions, seed = f.result()

                if status == "success":
                    successes += 1
                    all_times.append(t)
                    success_times.append(t)

                    valid, err = verify_solution(seed, actions)

                    if valid:
                        verified_successes += 1
                    else:
                        invalid_solutions += 1
                        print("===== INVALID SOLUTION =====")
                        print("seed:", seed.replace('\n', ' '))
                        print("error:", err)
                elif status == "failure":
                    failures += 1
                    all_times.append(t)
                    failure_times.append(t)
                elif status == "timeout":
                    failures += 1
                    timeouts += 1
                else:
                    failures += 1

                pbar.set_postfix(
                    success=successes,
                    fail=failures,
                    invalid=invalid_solutions
                )
                pbar.update(1)
        except KeyboardInterrupt:
            for future in futures:
                future.cancel()
            executor.shutdown(wait=False, cancel_futures=True)
        finally:
            pbar.close()

    print("=== Benchmark Results ===")
    print(f"method: {args.method}")
    print(f"iterations: {args.iterations}")
    print(f"jobs: {args.jobs}")
    print(f"nest: {args.nest}")
    print()

    print(f"successes: {successes}")
    print(f"failures: {failures}")
    print(f"timeouts: {timeouts}")
    print(f"verified successes: {verified_successes}")
    print(f"invalid solutions: {invalid_solutions}")

    p, lo, hi = win_rate_ci(successes, args.iterations)
    print(f"win rate: {p:.4f} ({p*100:.2f}%)")
    print(f"95% CI: [{lo:.4f}, {hi:.4f}] ({lo*100:.2f}% - {hi*100:.2f}%)")
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
