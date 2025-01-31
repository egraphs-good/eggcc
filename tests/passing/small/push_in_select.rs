fn main(x: i64) {
    if x < 0 {
        let mult: i64 = -2;
    } else {
        let mult: i64 = 3;
    }
    if x < 0 {
        let res: i64 = mult * x;
    } else {
        let res: i64 = abs(mult * x);
    }
    println!("{}", res);
}

// target:
// let res = select(x < 0, -2 * x, 3 * x)
