use crate::{dimension, transform};

/// Initial values followed by a snapshot after each processed bit.
#[derive(Debug)]
pub struct Trace {
    pub states: Vec<Vec<i64>>,
    pub inverse: bool,
}

impl Trace {
    /// Traces are intentionally restricted to small educational examples.
    pub fn new(input: &[i64], inverse: bool) -> Self {
        let n = dimension(input);
        assert!(n <= 8, "tracing supports at most 8 dimensions");
        let mut values = input.to_vec();
        let mut states = vec![values.clone()];
        for b in 0..n {
            transform::stage(&mut values, 1 << b, inverse);
            states.push(values.clone());
        }
        Self { states, inverse }
    }

    /// Payload bytes only, excluding Vec headers and allocator overhead.
    pub fn payload_bytes(&self) -> usize {
        self.states
            .iter()
            .map(|state| state.len() * size_of::<i64>())
            .sum()
    }
}
