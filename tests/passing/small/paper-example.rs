// ARGS: 5

fn main(input: i64) {
    let mut res: i64 = input;
    while (res < 100) {
        res = res * 2;
    }

    println!("{}", res);
}
