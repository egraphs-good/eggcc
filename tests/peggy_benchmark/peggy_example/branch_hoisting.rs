fn main(n: i64) -> i64 {
    let y: i64 = 0;
    let x: i64 = 0;
    while (y < 500) {
        if (n == 0) {
            x = y * 2;
        } else {
            x = y * 3;
        }
        y += 1;
    }
    return x;
}
