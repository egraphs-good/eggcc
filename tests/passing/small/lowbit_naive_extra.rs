// ARGS: 21324
// Compute the lowest 1 bit of an integer in the naive way
// Added x to test the state-edge passthrough
fn main(a:i64) {
    let mut n : i64 = a;
    let mut lb : i64 = 1;
    let mut x : i64 = 0;
    while (n == n / 2 * 2) {
        n = n / 2;
        lb = lb * 2;
        x = x + 1;
    }
    println!("{}", lb);
    println!("{}", x);
}
