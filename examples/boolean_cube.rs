use fast_zeta_transform::{svg, trace::Trace};
fn main() -> std::io::Result<()> {
    svg::write_report(
        &Trace::new(&[1, 2, 3, 4, 5, 6, 7, 8], false),
        std::path::Path::new("artifacts/cube"),
    )
}
