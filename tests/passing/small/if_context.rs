// ARGS: 20
fn main(x: i64) {
    let res: i64 = 0;
    if (x == 3) {
        res = x;
    } else {
        res = x * 3;
    }
    println!("{}", res);
}
