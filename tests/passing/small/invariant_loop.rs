// ARGS: 2 2

fn main(a: i64, b: i64) {
    let mut res: i64 = 0;
    let mut counter: i64 = 0;
    while (counter < a) {
        let mut inner_res: i64 = 0;
        let mut inner_counter: i64 = 0;
        while (inner_counter < b) {
            inner_res += a;
            inner_counter += 1;
        }
        res += inner_res;
    }

    println!("{}", inner_res);
}
