use crate::{dimension, metrics::Metrics};

/// Enumerates every submask, including zero, for each destination mask.
pub fn zeta(values: &[i64]) -> (Vec<i64>, Metrics) {
    let n = dimension(values);
    let mut result = vec![0i64; values.len()];
    for (mask, sum) in result.iter_mut().enumerate() {
        let mut submask = mask;
        loop {
            *sum = sum
                .checked_add(values[submask])
                .expect("subset sum overflow");
            if submask == 0 {
                break;
            }
            submask = (submask - 1) & mask;
        }
    }
    let visits = 3u64.checked_pow(n as u32).expect("metric overflow");
    (
        result,
        Metrics {
            additions: visits,
            subset_visits: visits,
            ..Metrics::default()
        },
    )
}
