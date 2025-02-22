fn modulo(a: i64, b: i64) -> i64 {
    let div: i64 = a / b; // Integer division
    let remainder: i64 = a - (div * b); // Compute remainder manually
    let mut res: i64 = 0;
    if remainder < 0 {
        res = remainder + b; // Ensure non-negative result
    } else {
        res = remainder;
    }
    return res;
}

fn max_score(s1: i64, s2: i64) -> i64 {
    if s1 >= s2 {
        return s1;
    } else {
        return s2;
    }
}

fn match_base(b1: i64, b2: i64) -> i64 {
    if (b1 + b2) == 3 {
        return 1;
    } else {
        return 0;
    }
}

fn init_array(n: i64, seq: &mut [i64; 180], table: &mut [[i64; 180]; 180]) {
    let mut i: i64 = 0;
    while i < n {
        seq[i as usize] = modulo(i + 1, 4);
        i += 1;
    }

    i = 0;
    while i < n {
        let mut j: i64 = 0;
        while j < n {
            table[i as usize][j as usize] = 0;
            j += 1;
        }
        i += 1;
    }
}

fn sum_array(n: i64, table: &[[i64; 180]; 180]) -> i64 {
    let mut sum: i64 = 0;
    let mut i: i64 = 0;
    while i < n {
        let mut j: i64 = i;
        while j < n {
            let x: i64 = table[i as usize][j as usize];
            // Can't print multiple things on nightly.
            // Uncomment to run locally to check output.
            // println!("{} ", x);
            sum += x;
            j += 1;
        }
        i += 1;
    }
    return sum;
}

fn kernel_nussinov(n: i64, seq: &mut [i64; 180], table: &mut [[i64; 180]; 180]) {
    let mut i: i64 = n - 1;
    while i >= 0 {
        let mut j: i64 = i + 1;
        while j < n {
            if j - 1 >= 0 {
                table[i as usize][j as usize] = max_score(
                    table[i as usize][j as usize],
                    table[i as usize][(j - 1) as usize],
                );
            }
            if i + 1 < n {
                table[i as usize][j as usize] = max_score(
                    table[i as usize][j as usize],
                    table[(i + 1) as usize][j as usize],
                );
            }
            if j - 1 >= 0 && i + 1 < n {
                if i < j - 1 {
                    table[i as usize][j as usize] = max_score(
                        table[i as usize][j as usize],
                        table[(i + 1) as usize][(j - 1) as usize]
                            + match_base(seq[i as usize], seq[j as usize]),
                    );
                } else {
                    table[i as usize][j as usize] = max_score(
                        table[i as usize][j as usize],
                        table[(i + 1) as usize][(j - 1) as usize],
                    );
                }
            }
            let mut k: i64 = i + 1;
            while k < j {
                table[i as usize][j as usize] = max_score(
                    table[i as usize][j as usize],
                    table[i as usize][k as usize] + table[(k + 1) as usize][j as usize],
                );
                k += 1;
            }
            j += 1;
        }
        i -= 1;
    }
}

fn main() {
    let n: i64 = 180;
    let mut seq: [i64; 180] = [0; 180];
    let dummy: [i64; 180] = [0; 180];
    let mut table: [[i64; 180]; 180] = [dummy; 180];

    // Init
    let mut i: i64 = 0;
    while i < n {
        table[i] = [0; 180];
        i += 1;
    }
    drop(dummy);

    init_array(n, &mut seq, &mut table);
    kernel_nussinov(n, &mut seq, &mut table);
    let res: i64 = sum_array(n, &table);

    // Drop
    drop(seq);
    i = 0;
    while i < n {
        drop(table[i]);
        i += 1;
    }
    drop(table);

    println!("{}", res);
}
