use crate::{dimension, metrics::Metrics, transform};

/// Computes h[S] = sum_{A union B = S} f[A] g[B].
pub fn fast(f: &[i64], g: &[i64]) -> (Vec<i64>, Metrics) {
    let n = dimension(f);
    assert_eq!(f.len(), g.len(), "input lengths must match");
    let mut a = f.to_vec();
    let mut b = g.to_vec();
    let work = transform::zeta(&mut a).additions;
    transform::zeta(&mut b);
    for (x, y) in a.iter_mut().zip(b) {
        *x = x.checked_mul(y).expect("convolution product overflow");
    }
    transform::mobius(&mut a);
    (
        a,
        Metrics {
            additions: 2 * work,
            subtractions: work,
            multiplications: f.len() as u64,
            stages: 3 * n,
            ..Metrics::default()
        },
    )
}

/// Independent O(4^n) reference implementation.
pub fn brute_force(f: &[i64], g: &[i64]) -> (Vec<i64>, Metrics) {
    dimension(f);
    assert_eq!(f.len(), g.len(), "input lengths must match");
    let mut result = vec![0i64; f.len()];
    for (a, &x) in f.iter().enumerate() {
        for (b, &y) in g.iter().enumerate() {
            let product = x.checked_mul(y).expect("convolution product overflow");
            result[a | b] = result[a | b]
                .checked_add(product)
                .expect("convolution sum overflow");
        }
    }
    let pairs = (f.len() as u64).checked_pow(2).expect("metric overflow");
    (
        result,
        Metrics {
            additions: pairs,
            multiplications: pairs,
            pair_visits: pairs,
            ..Metrics::default()
        },
    )
}
