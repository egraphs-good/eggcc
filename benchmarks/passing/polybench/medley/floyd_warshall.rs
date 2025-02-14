// STANDARD_DATASET parameters:
// #   define N 1024

fn modulo(a: i64, b: i64) -> i64 {
    let div: i64 = a / b; // Integer division
    let remainder: i64 = a - (div * b); // Compute remainder manually
    if remainder < 0 {
        return remainder + b; // Ensure non-negative result
    } else {
        return remainder;
    }
}

fn init_array(path: &mut [[f64; 1024]; 1024]) {
    let n: i64 = 1024;
    let mut i: i64 = 0;
    let mut fi: f64 = 0.0;
    while i < n {
        let mut j: i64 = 0;
        let mut fj: f64 = 0.0;
        while j < n {
            path[i][j] = ((fi + 1.0) * (fj + 1.0)) / 1024.0;
            j += 1;
            fj += 1.0;
        }
        i += 1;
        fi += 1.0;
    }
}

fn sum_array(path: &[[f64; 1024]; 1024]) -> f64 {
    let mut sum: f64 = 0.0;
    let mut i: i64 = 0;

    while i < 1024 {
        let mut j: i64 = 0;
        while j < 1024 {
            sum += path[i][j];
            if modulo(i * 1024 + j, 20) == 0 {
                sum += 20.0;
            }
            j += 1;
        }
        i += 1;
    }
    return sum;
}

fn kernel_floyd_warshall(path: &mut [[f64; 1024]; 1024]) {
    let n: i64 = 1024;
    let mut k: i64 = 0;
    let mut i: i64 = 0;
    let mut j: i64 = 0;
    while k < 1024 {
        i = 0;
        while i < n {
            j = 0;
            while j < n {
                if path[i][j] < (path[i][k] + path[k][j]) {
                    path[i][j] = path[i][j];
                } else {
                    path[i][j] = path[i][k] + path[k][j];
                }
                j += 1;
            }
            i += 1;
        }
        k += 1;
    }
}

fn main() {
    let mut path: [[f64; 1024]; 1024] = [[0.0; 1024]; 1024];

    init_array(&mut path);
    kernel_floyd_warshall(&mut path);
    let res: f64 = sum_array(&path);
    println!("{}", res);
}
