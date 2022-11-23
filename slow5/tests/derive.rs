#[test]
fn derive_test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/run-pass-0.rs");
    t.pass("examples/derive_aux.rs");
}
