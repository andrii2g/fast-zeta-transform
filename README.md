# fast-zeta-transform

A dependency-free Rust exploration of subset sums, Möbius inversion, and
OR-convolution, with exact work counts, deterministic verification, runtime
experiments, and an offline SVG Boolean-cube viewer.

## Run

Use a Rust toolchain supporting edition 2024 (developed with Rust 1.97.1).

```sh
cargo test
cargo run --release -- verify --max-n 16 --seed 42
cargo run --release -- demo --n 3 --output artifacts/demo
cargo run --release -- bench --max-n 16 --repeats 7 --output artifacts/metrics.csv
cargo run --release --example boolean_cube
```

Open `artifacts/demo/index.html` in a browser and select a transformation stage.
Each stage also has a standalone SVG. Binary masks label subsets with bit 0
on the right. Teal arrows show active additions; equations explain every update.
Demo supports n=0..3; library traces support n=0..8. Artifacts are Git-ignored.

## Why it works

An array of length 2^n stores f(S), indexed by the integer bitmask of S.
We want F(S) = sum of f(T) over all T contained in S.

Naïve enumeration starts at `submask = mask` and repeatedly uses
`submask = (submask - 1) & mask`, processing zero before stopping. Across all
destinations there are 3^n visits: an element is outside S, in S but outside T,
or in T. Each visited term is added to an initially zero sum.

```text
for each bit b:
    for each mask containing b:
        values[mask] += values[mask without b]
```

After processing some bits, each entry sums original values obtained by
clearing any combination of those bits. The next bit partitions contributions
into disjoint groups containing or omitting it. No term is duplicated. The
source lacks the current bit and is unchanged in this stage, allowing in-place
updates. After all bits, every submask contributes once. Work is n*2^(n-1)
additions (zero for n=0), O(n*2^n) time and O(2^n) array storage.

Möbius inversion replaces addition with subtraction. Bit operators commute,
so the same bit order recovers f. Equivalently, f(S) is the sum over T contained
in S of (-1)^(|S|-|T|) F(T).

## OR-convolution

Define h(S) as the sum of f(A)*g(B) over A union B = S. Then

```text
zeta(h)[S] = sum_{A subset S, B subset S} f[A] * g[B]
           = zeta(f)[S] * zeta(g)[S]
h = mobius(zeta(f) * zeta(g))
```

Two forward transforms and one inverse use 3*n*2^(n-1) additions/subtractions;
the pointwise product uses 2^n multiplications. The independent reference loops
over all 4^n pairs, accumulating at `A | B`.

## Library

```rust
use fast_zeta_transform::{transform, convolution};
let original = vec![1, 2, 3, 4];
let mut values = original.clone();
assert_eq!(transform::zeta(&mut values).additions, 4);
assert_eq!(values, [1, 3, 4, 10]);
transform::mobius(&mut values);
assert_eq!(values, original);
assert_eq!(convolution::fast(&original, &original).0,
           convolution::brute_force(&original, &original).0);
```

Nonzero power-of-two lengths are required; convolution lengths must match.
Invalid shapes and arithmetic overflow panic. Checked i64 arithmetic remains
checked when linked into other release profiles. Length one represents the
empty ground set. Library callers control allocations; the CLI caps n at 16.

## Verification and numerical bounds

Tests cover three deterministic random functions per n=1..16, inverse recovery,
naïve zeta agreement at every dimension, random convolution against all pairs
through n=8, and independent sparse convolution oracles through n=16. Additional
cases cover zeros, negative values, basis vectors, the empty ground set,
invalid inputs, overflow, stage values, cube edges, and CLI errors. `verify`
runs analogous checks with a chosen seed. SplitMix64 generates values in -10..10
without an external RNG dependency.

For these inputs and n<=16, zeta values have magnitude at most 10*2^n,
transformed products at most 100*4^n, and a conservative bound on inverse
intermediate values is 100*8^n <= 2.82e16, below i64::MAX. Arbitrary library
inputs can exceed those bounds, so every arithmetic operation is checked.

## Metrics

CSV includes additions, subtractions, multiplications, subset visits, pair
visits, bit stages, median nanoseconds, array payload bytes, and measured
speedup. Exact counts come from loop bounds outside hot loops. Fast transforms
report zero submask-enumeration visits, although they scan masks every bit.
Convolution stages sum all three transforms.

One untimed warmup precedes an odd number of samples (default seven).
In-place buffer setup and result destruction are outside timing; allocations
inside out-of-place algorithms are included. `black_box` prevents elimination
of computed work. Use release mode; small cases are sensitive to timer
resolution and system load. Speedup is reference median / fast median, blank
when no reference is measured. Nothing is extrapolated. Naïve zeta runs through
n=16; brute OR stops at n=8 because n=16 would visit 4,294,967,296 pairs.

`peak_buffer_bytes` is algorithm array payload, including inputs: one array
for zeta/Möbius, two for naïve zeta, three for brute OR, four for fast OR.
This is not measured process RSS; harness memory, Vec headers, allocator
overhead and scalars are excluded. Optional trace payload is (n+1)*2^n*8 bytes,
printed by the demo. Tracing changes storage to O(n*2^n) and is not timed.

## Layout

- `transform`, `naive`, `convolution`: algorithms and independent references.
- `metrics`: work counts.
- `trace`, `svg`: snapshots and offline diagrams.
- `main`: CLI, verification and benchmarks.
- `tests`: correctness and CLI checks.

No external Rust dependencies or network services are needed.
