// ARGS: 20
fn main(input: i64) {
    let a: i64 = input * 2;
    if (input > 0) {
        let x: i64 = a * 5;
    } else {
        let x: i64 = a * -3;
    }

    if (x >= 0) {
        let res: i64 = x * 10;
    } else {
        let res: i64 = x * 37;
    }

    println!("{}", res);
}
