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

fn init_array(n: i64, path: &mut [[i64; 180]; 180]) {
    let mut i: i64 = 0;
    while i < n {
        let mut j: i64 = 0;
        while j < n {
            path[i as usize][j as usize] = modulo(i * j, 7) + 1;
            if modulo(i + j, 13) == 0 || modulo(i + j, 7) == 0 || modulo(i + j, 11) == 0 {
                path[i as usize][j as usize] = 999;
            }
            j += 1;
        }
        i += 1;
    }
}

fn sum_array(n: i64, path: &[[i64; 180]; 180]) -> i64 {
    let mut sum: i64 = 0;
    let mut i: i64 = 0;
    while i < n {
        let mut j: i64 = 0;
        while j < n {
            let x: i64 = path[i as usize][j as usize];
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

fn kernel_floyd_warshall(n: i64, path: &mut [[i64; 180]; 180]) {
    let mut k: i64 = 0;
    while k < n {
        let mut i: i64 = 0;
        while i < n {
            let mut j: i64 = 0;
            while j < n {
                if path[i as usize][j as usize]
                    > path[i as usize][k as usize] + path[k as usize][j as usize]
                {
                    path[i as usize][j as usize] =
                        path[i as usize][k as usize] + path[k as usize][j as usize];
                }
                j += 1;
            }
            i += 1;
        }
        k += 1;
    }
}

fn main() {
    let n: i64 = 180;

    let dummy: [i64; 180] = [0; 180];
    let mut path: [[i64; 180]; 180] = [dummy; 180];

    // Init
    let mut i: i64 = 0;
    while i < n {
        path[i] = [0; 180];
        i += 1;
    }
    drop(dummy);

    init_array(n, &mut path);
    kernel_floyd_warshall(n, &mut path);
    let res: i64 = sum_array(n, &path);

    // Drop
    i = 0;
    while i < n {
        drop(path[i]);
        i += 1;
    }
    drop(path);

    println!("{}", res);
}
