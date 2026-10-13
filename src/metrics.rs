/// Algorithmic work counts; timing and allocation are measured separately.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Metrics {
    pub additions: u64,
    pub subtractions: u64,
    pub multiplications: u64,
    pub subset_visits: u64,
    pub pair_visits: u64,
    pub stages: usize,
}
