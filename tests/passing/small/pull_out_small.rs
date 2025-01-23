// ARGS: 20
fn main(x: i64) {
    let a: i64 = x * 5;
    if (x > 0) {
        let r: i64 = a + x;
        let z: i64 = 10;
        println!("{}", z);
    } else {
        let r: i64 = x + a;
        let z: i64 = 20;
        println!("{}", z);
    }
    println!("{}", r);
}
