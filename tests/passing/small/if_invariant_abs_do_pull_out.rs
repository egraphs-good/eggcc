// ARGS: -20

// TODO make this built in to bril
fn abs(input: i64) -> i64 {
    if (input < 0) {
        return -input;
    } else {
        return input;
    }
}

fn unrelated_fn(input: i64) -> i64 {
    return input / 4;
}

fn other_unrelated_fn(input: i64) -> i64 {
    return (input * 3) / 5;
}

fn main(input: i64) {
    let mut res: i64 = abs(input) * 2;

    if (input > 0) {
        res = res + abs(input);
        res = res + unrelated_fn(input);
    } else {
        res = res + abs(input);
        res = res + other_unrelated_fn(input);
    }

    println!("{}", res);
}

/*
fn target(input: i64) {
    let mut res: i64 = abs(input) * 3;

    if (input > 0) {
        res = res + unrelated_fn(input);
    } else {
        res = res + other_unrelated_fn(input);
    }

    println!("{}", res);
}
*/
