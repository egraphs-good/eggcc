fn modulo(a: i64, b: i64) -> i64 {
    let div: i64 = a / b; // Integer division
    let remainder: i64 = a - (div * b); // Compute remainder manually
    if remainder < 0 {
        return remainder + b; // Ensure non-negative result
    } else {
        return remainder;
    }
}

fn init_array(n: i64, path: &mut [[i64; 500]; 500]) {
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

fn sum_array(n: i64, path: &[[i64; 500]; 500]) -> i64 {
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

fn kernel_floyd_warshall(n: i64, path: &mut [[i64; 500]; 500]) {
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
    let n: i64 = 500;

    let dummy: [i64; 500] = [0; 500];
    let mut path: [[i64; 500]; 500] = [dummy; 500];

    // Init
    let mut i: i64 = 0;
    while i < n {
        path[i] = [0; 500];
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
