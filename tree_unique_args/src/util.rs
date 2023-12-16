pub(crate) fn n_vars(n: usize, base: &str) -> Vec<String> {
    (0..n)
        .map(|i| format!("{base}{i}"))
        .collect::<Vec<String>>()
}
