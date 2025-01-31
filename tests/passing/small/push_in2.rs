fn main(x: i64) {
    let a: i64 = x * 3;
    if x > 0 {
        if a > 0 {
            let y: i64 = 1;
        } else {
            let y: i64 = 2;
        }
    } else {
        if a > 0 {
            let y: i64 = 3;
        } else {
            let y: i64 = 4;
        }
    }
    println!("{}", y);
}

// target: select(x > 0, 1, 4)
