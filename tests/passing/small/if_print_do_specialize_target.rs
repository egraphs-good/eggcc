// ARGS: 2 2

fn main(a: i64, b: i64) {
    let mut res: i64 = 0;
    if (a * a * a * a * a * a == b) {
        res = b * b;
    } else {
        res = a * a * a * a * a * a * b;
    }

    println!("{}", res);
}
