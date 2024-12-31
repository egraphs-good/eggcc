// ARGS: -20

// TODO make this built in to bril
fn abs(input: i64) -> i64 {
    if (input < 0) {
        return -input;
    } else {
        return input;
    }
}

fn main(input: i64) {
    let mut res: i64 = abs(input) * 2;

    if (input > 0) {
        res = res + abs(input) + 1;
    } else {
        res = res + abs(input) - 1;
    }

    print(res);
}

fn target(input: i64) {
    let mut res: i64 = abs(input) * 3;
    if (input > 0) {
        res = res + 1;
    } else {
        res = res - 1;
    }

    print(res);
}
