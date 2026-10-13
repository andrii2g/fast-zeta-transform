use std::process::Command;
#[test]
fn invalid_requests_and_verification() {
    let binary = env!("CARGO_BIN_EXE_fast-zeta-transform");
    for args in [
        vec!["demo", "--n", "4"],
        vec!["bench", "--repeats", "2"],
        vec!["verify", "--max-n"],
        vec!["unknown"],
        vec!["demo", "--seed", "42"],
    ] {
        assert!(
            !Command::new(binary)
                .args(args)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    assert!(
        Command::new(binary)
            .args(["verify", "--max-n", "10"])
            .output()
            .unwrap()
            .status
            .success()
    );
}
