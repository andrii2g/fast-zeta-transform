use fast_zeta_transform::{svg, trace::Trace};

#[test]
fn snapshots_explain_each_bit() {
    let trace = Trace::new(&[1, 2, 3, 4, 5, 6, 7, 8], false);
    assert_eq!(
        trace.states,
        vec![
            vec![1, 2, 3, 4, 5, 6, 7, 8],
            vec![1, 3, 3, 7, 5, 11, 7, 15],
            vec![1, 3, 4, 10, 5, 11, 12, 26],
            vec![1, 3, 4, 10, 6, 14, 16, 36]
        ]
    );
    assert_eq!(trace.payload_bytes(), 256);
    let inverse = Trace::new(trace.states.last().unwrap(), true);
    assert_eq!(inverse.states.last(), trace.states.first());
    for stage in 0..=3 {
        let svg = svg::stage_svg(&trace, stage);
        assert_eq!(svg.matches("<circle ").count(), 8);
        assert_eq!(svg.matches("<line ").count(), 12);
        assert_eq!(
            svg.matches("marker-end=").count(),
            if stage == 0 { 0 } else { 4 }
        );
    }
}
