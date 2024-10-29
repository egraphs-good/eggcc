// ARGS: 20 30
fn main(x: i64, y: i64) {
    let res: i64 = 0;
    if (x * y < 20) {
        res = x;
    } else {
        res = y;
    }
    println!("{}", res);
}
