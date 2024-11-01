// ARGS: 20 30
fn main(x: i64, y: i64) {
    let res: i64 = 0;
    // if P then A else B where A and B are inputs to the region
    if (x * y < 20) {
        res = x;
    } else {
        res = y;
    }
    println!("{}", res);

    // if P then C1 else C2
    if (x * y > 10) {
        res = 4;
    } else {
        res = 5;
    }
    println!("{}", res);

    // if P then C1 (and implicitly, the else is a passthrough)
    if (x * y > 20) {
        res = 10;
    }
    println!("{}", res);

    // if P then X else Y where X and Y are small, pure expressions
    if (x * y == 40) {
        res = x * 2;
    } else {
        res = x + 5;
    }
    println!("{}", res);
}
