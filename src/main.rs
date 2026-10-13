use fast_zeta_transform::{
    convolution, metrics::Metrics, naive, random_values, svg, trace::Trace, transform,
};
use std::{env, error::Error, fmt::Write as _, fs, hint::black_box, path::Path, time::Instant};

const HELP: &str = "fast-zeta-transform
  demo   [--n 3] [--output artifacts/demo]
  verify [--max-n 16] [--seed 42]
  bench  [--max-n 16] [--seed 42] [--repeats 7] [--output artifacts/metrics.csv]
Dimensions: demo 0..3, verify/bench 0..16. Repeats: odd, 3..101.";

struct Options {
    command: String,
    n: u32,
    seed: u64,
    repeats: usize,
    output: String,
}

fn parse(args: Vec<String>) -> Result<Option<Options>, String> {
    if args.is_empty() || matches!(args[0].as_str(), "--help" | "-h" | "help") {
        return Ok(None);
    }
    let command = args[0].clone();
    if !["demo", "verify", "bench"].contains(&command.as_str()) {
        return Err(format!("unknown command: {command}"));
    }
    let mut o = Options {
        n: if command == "demo" { 3 } else { 16 },
        seed: 42,
        repeats: 7,
        output: if command == "demo" {
            "artifacts/demo"
        } else {
            "artifacts/metrics.csv"
        }
        .into(),
        command,
    };
    let mut seen = std::collections::HashSet::new();
    let mut pairs = args[1..].chunks_exact(2);
    for pair in &mut pairs {
        let key = pair[0].as_str();
        let value = &pair[1];
        if !seen.insert(key) {
            return Err(format!("duplicate option: {key}"));
        }
        match key {
            "--n" if o.command == "demo" => o.n = value.parse().map_err(|_| "invalid n")?,
            "--max-n" if o.command != "demo" => o.n = value.parse().map_err(|_| "invalid max-n")?,
            "--seed" if o.command != "demo" => {
                o.seed = value.parse().map_err(|_| "invalid seed")?
            }
            "--repeats" if o.command == "bench" => {
                o.repeats = value.parse().map_err(|_| "invalid repeats")?
            }
            "--output" if o.command != "verify" => o.output = value.clone(),
            _ => return Err(format!("unsupported option: {key}")),
        }
    }
    if !pairs.remainder().is_empty() {
        return Err("each option needs a value".into());
    }
    if o.n > if o.command == "demo" { 3 } else { 16 } {
        return Err("dimension out of range".into());
    }
    if !(3..=101).contains(&o.repeats) || o.repeats.is_multiple_of(2) {
        return Err("repeats must be odd and in 3..101".into());
    }
    Ok(Some(o))
}

fn verify(o: &Options) {
    for n in 0..=o.n {
        for offset in 0..3 {
            let f = random_values(n, o.seed.wrapping_add(offset));
            let g = random_values(n, o.seed.wrapping_add(offset + 10));
            let mut actual = f.clone();
            transform::zeta(&mut actual);
            if offset == 0 || n <= 8 {
                assert_eq!(actual, naive::zeta(&f).0);
            }
            transform::mobius(&mut actual);
            assert_eq!(actual, f);
            if n <= 8 {
                assert_eq!(
                    convolution::fast(&f, &g).0,
                    convolution::brute_force(&f, &g).0
                );
            } else {
                let len = f.len();
                let left = [(0, f[0]), (len / 3, f[1]), (len - 1, f[2])];
                let right = [(0, g[0]), (len / 2, g[1]), (len - 2, g[2])];
                let mut a = vec![0; len];
                let mut b = vec![0; len];
                let mut expected = vec![0; len];
                for &(mask, value) in &left {
                    a[mask] = value;
                }
                for &(mask, value) in &right {
                    b[mask] = value;
                }
                for &(mask, x) in &left {
                    for &(other, y) in &right {
                        expected[mask | other] += x * y;
                    }
                }
                assert_eq!(convolution::fast(&a, &b).0, expected);
            }
        }
        println!("n={n}: three seeds passed (zeta, inverse, convolution)");
    }
}

// Setup and destruction are outside the clock; first run is an untimed warmup.
fn median<S, R>(
    repeats: usize,
    mut setup: impl FnMut() -> S,
    mut run: impl FnMut(&mut S) -> R,
) -> u128 {
    black_box(run(&mut setup()));
    let mut samples = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        let mut state = setup();
        let start = Instant::now();
        let result = black_box(run(black_box(&mut state)));
        samples.push(start.elapsed().as_nanos());
        black_box(result);
    }
    samples.sort_unstable();
    samples[repeats / 2]
}

fn row(
    csv: &mut String,
    n: u32,
    name: &str,
    work: Metrics,
    ns: u128,
    bytes: usize,
    speedup: Option<f64>,
) {
    writeln!(
        csv,
        "{n},{name},{},{},{},{},{},{},{ns},{bytes},{}",
        work.additions,
        work.subtractions,
        work.multiplications,
        work.subset_visits,
        work.pair_visits,
        work.stages,
        speedup.map(|s| format!("{s:.3}")).unwrap_or_default()
    )
    .unwrap();
}

fn bench(o: &Options) -> Result<(), Box<dyn Error>> {
    let mut csv = String::from(
        "n,algorithm,additions,subtractions,multiplications,subset_visits,pair_visits,stages,median_ns,peak_buffer_bytes,speedup_vs_reference\n",
    );
    for n in 0..=o.n {
        let f = random_values(n, o.seed);
        let g = random_values(n, o.seed.wrapping_add(1));
        let bytes = f.len() * size_of::<i64>();
        let mut transformed = f.clone();
        let work = transform::zeta(&mut transformed);
        let (expected, naive_work) = naive::zeta(&f);
        assert_eq!(transformed, expected);
        let slow_ns = median(o.repeats, || (), |_| naive::zeta(black_box(&f)));
        let fast_ns = median(o.repeats, || f.clone(), |a| transform::zeta(a));
        row(
            &mut csv,
            n,
            "naive_zeta",
            naive_work,
            slow_ns,
            2 * bytes,
            None,
        );
        row(
            &mut csv,
            n,
            "zeta",
            work,
            fast_ns,
            bytes,
            (fast_ns > 0).then(|| slow_ns as f64 / fast_ns as f64),
        );
        let inverse_work = transform::mobius(&mut transformed.clone());
        let inverse_ns = median(o.repeats, || transformed.clone(), |a| transform::mobius(a));
        row(&mut csv, n, "mobius", inverse_work, inverse_ns, bytes, None);
        let (result, conv_work) = convolution::fast(&f, &g);
        let conv_ns = median(
            o.repeats,
            || (),
            |_| convolution::fast(black_box(&f), black_box(&g)),
        );
        let mut speedup = None;
        if n <= 8 {
            let (reference, reference_work) = convolution::brute_force(&f, &g);
            assert_eq!(result, reference);
            let reference_ns = median(
                o.repeats,
                || (),
                |_| convolution::brute_force(black_box(&f), black_box(&g)),
            );
            row(
                &mut csv,
                n,
                "brute_or",
                reference_work,
                reference_ns,
                3 * bytes,
                None,
            );
            if conv_ns > 0 {
                speedup = Some(reference_ns as f64 / conv_ns as f64);
            }
        }
        row(
            &mut csv,
            n,
            "fast_or",
            conv_work,
            conv_ns,
            4 * bytes,
            speedup,
        );
        println!("n={n}: zeta {fast_ns} ns, naive {slow_ns} ns");
    }
    let path = Path::new(&o.output);
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, csv)?;
    println!("Wrote {}", path.display());
    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    let Some(o) = parse(env::args().skip(1).collect())? else {
        println!("{HELP}");
        return Ok(());
    };
    match o.command.as_str() {
        "demo" => {
            let input: Vec<i64> = (1..=(1 << o.n)).collect();
            let trace = Trace::new(&input, false);
            svg::write_report(&trace, Path::new(&o.output))?;
            for (stage, values) in trace.states.iter().enumerate() {
                println!("stage {stage}: {values:?}");
            }
            println!(
                "Trace payload: {} bytes; report: {}/index.html",
                trace.payload_bytes(),
                o.output
            );
        }
        "verify" => verify(&o),
        "bench" => bench(&o)?,
        _ => unreachable!(),
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}\n\n{HELP}");
        std::process::exit(1);
    }
}
