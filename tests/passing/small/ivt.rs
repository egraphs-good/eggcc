// ARGS: 20

fn main(max: i64) {
    let mut i: i64 = 0;
    let unrelated: i64 = 20;
    while (i < max) {
        println!("{}", i);
        i += 1;
    }
    println!("{}", unrelated);
}
