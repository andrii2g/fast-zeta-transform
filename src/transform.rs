use crate::{dimension, metrics::Metrics};

/// In-place subset sums. Returns exact work counts without per-edge counters.
pub fn zeta(values: &mut [i64]) -> Metrics {
    apply(values, false)
}

/// Inverse of [`zeta`].
pub fn mobius(values: &mut [i64]) -> Metrics {
    apply(values, true)
}

pub(crate) fn stage(values: &mut [i64], bit: usize, inverse: bool) {
    for mask in 0..values.len() {
        if mask & bit != 0 {
            values[mask] = if inverse {
                values[mask].checked_sub(values[mask ^ bit])
            } else {
                values[mask].checked_add(values[mask ^ bit])
            }
            .expect("transform arithmetic overflow");
        }
    }
}

fn apply(values: &mut [i64], inverse: bool) -> Metrics {
    let n = dimension(values);
    for b in 0..n {
        stage(values, 1 << b, inverse);
    }
    let operations = n as u64 * (values.len() / 2) as u64;
    Metrics {
        additions: if inverse { 0 } else { operations },
        subtractions: if inverse { operations } else { 0 },
        stages: n,
        ..Metrics::default()
    }
}
