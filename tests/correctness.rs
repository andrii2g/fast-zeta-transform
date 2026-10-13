use fast_zeta_transform::{convolution, naive, random_values, transform};

#[test]
fn deterministic_functions_through_dimension_16() {
    for n in 1..=16 {
        for seed in [0, 42, 123456789] {
            let input = random_values(n, seed);
            let mut actual = input.clone();
            let metrics = transform::zeta(&mut actual);
            assert_eq!(metrics.additions, n as u64 * (1u64 << (n - 1)));
            if seed == 42 || n <= 8 {
                let (expected, work) = naive::zeta(&input);
                assert_eq!(actual, expected, "n={n}, seed={seed}");
                assert_eq!(work.subset_visits, 3u64.pow(n));
            }
            transform::mobius(&mut actual);
            assert_eq!(actual, input);
        }
    }
}

#[test]
fn random_convolution_matches_all_pairs() {
    for n in 0..=8 {
        for seed in [0, 42, 99] {
            let f = random_values(n, seed);
            let g = random_values(n, seed + 1);
            assert_eq!(
                convolution::fast(&f, &g).0,
                convolution::brute_force(&f, &g).0
            );
        }
    }
}

#[test]
fn sparse_convolution_at_large_dimensions() {
    for n in 9..=16 {
        let len = 1 << n;
        let mut f = vec![0; len];
        let mut g = vec![0; len];
        let left = [(0, -3), (5, 7), (len - 1, 2)];
        let right = [(0, 4), (10, -2), (len / 2, 3)];
        for &(mask, value) in &left {
            f[mask] = value;
        }
        for &(mask, value) in &right {
            g[mask] = value;
        }
        let mut expected = vec![0; len];
        for &(a, x) in &left {
            for &(b, y) in &right {
                expected[a | b] += x * y;
            }
        }
        assert_eq!(convolution::fast(&f, &g).0, expected);
    }
}

#[test]
fn edge_cases_and_basis_vectors() {
    let mut singleton = [7];
    assert_eq!(transform::zeta(&mut singleton).additions, 0);
    transform::mobius(&mut singleton);
    assert_eq!(singleton, [7]);
    assert_eq!(naive::zeta(&[7]).0, [7]);
    let mut zeros = [0; 8];
    transform::zeta(&mut zeros);
    assert_eq!(zeros, [0; 8]);
    for source in 0..8 {
        let mut basis = [0; 8];
        basis[source] = 1;
        transform::zeta(&mut basis);
        for (mask, value) in basis.iter().enumerate() {
            assert_eq!(*value, i64::from(mask & source == source));
        }
    }
}

#[test]
#[should_panic(expected = "power of two")]
fn rejects_empty_input() {
    transform::zeta(&mut []);
}

#[test]
#[should_panic(expected = "power of two")]
fn rejects_invalid_length() {
    transform::mobius(&mut [0; 3]);
}

#[test]
#[should_panic(expected = "lengths must match")]
fn rejects_mismatched_convolution() {
    convolution::fast(&[1, 2], &[1]);
}

#[test]
#[should_panic(expected = "overflow")]
fn detects_overflow() {
    transform::zeta(&mut [i64::MAX, 1]);
}
