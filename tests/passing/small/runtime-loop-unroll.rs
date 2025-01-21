// ARGS: 20
fn main(iters: i64) {
    let mut iter = 0;
    let mut res = 1;
    while (iter < iters) {
        res = res * 2 + 1;
    }

    println!("{}", res);
}

// target is unrolled 8 times, allowing it to add 8 each iteration
