pub(crate) fn n_vars(n: usize, base: &str) -> Vec<String> {
    (0..n)
        .map(|i| format!("{base}{i}"))
        .collect::<Vec<String>>()
}

#[allow(dead_code)]
pub(crate) fn cons_list(els: &Vec<&str>) -> String {
    let mut res = vec![];
    for el in els {
        res.push(format!("(Cons {el} "));
    }
    res.push("(Nil)".to_string());
    for _ in els {
        res.push(")".to_string());
    }
    res.join("")
}
