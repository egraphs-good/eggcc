// ARGS: 20
fn main(x: i64) {
    let ten: i64 = 10;
    let done: i64 = ten;
    let i: i64 = 0;
    let res: i64 = 0;
    while !(done == 5) {
        i += 1;
        res += i;
        if i == x {
            done = 5;
        }
    }
    println!("{}", res);
}
