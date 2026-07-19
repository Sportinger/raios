pub fn fixture_value() -> usize {
    #[cfg(feature = "broken")]
    let value: usize = "deliberate fixture error";

    #[cfg(not(feature = "broken"))]
    let value: usize = 1;

    value
}
