// ARGS: 2 3

fn main(a: i64, b: i64) {
    let mut res: i64 = 0;
    if (a > b) {
        res = a * b;
    } else {
        res = b * a;
    }

    println!("{}", res);
}
