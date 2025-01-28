// ARGS: 21324
// Compute the lowest 1 bit of an integer
fn main(n:i64) {
    let mut lb : i64 = n & (-n);
    println!("{}", lb);
}