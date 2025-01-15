// ARGS: 20
fn unrelated_fn(input: i64, res: i64) -> i64 {
    return input / (res / 3);
}

fn other_unrelated_fn(input: i64, res: i64) -> i64 {
    return res / (input / 5);
}

fn main(input: i64) {
    let mut res: i64 = abs(input) * 2;

    if (input > 0) {
        res = res + abs(input) - input;
        res = unrelated_fn(input, res);
    } else {
        res = res + abs(input) + input;
        res = other_unrelated_fn(input, res);
    }

    println!("{}", res);
}
