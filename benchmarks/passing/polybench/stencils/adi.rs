// STANDARD_DATASET parameters:
// #   define TSTEPS 50
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

fn init_array(
    n: i64,
    nf: f64,
    x: &mut [[f64; 1024]; 1024],
    a: &mut [[f64; 1024]; 1024],
    b: &mut [[f64; 1024]; 1024],
) {
    let mut i: i64 = 0;
    let mut fi: f64 = 0.0;
    while i < n {
        let mut j: i64 = 0;
        let mut fj: f64 = 0.0;
        while j < n {
            x[i][j] = (fi * (fj + 1.0) + 1.0) / nf;
            a[i][j] = (fi * (fj + 2.0) + 2.0) / nf;
            b[i][j] = (fi * (fj + 3.0) + 3.0) / nf;
            j = j + 1;
            fj = fj + 1.0;
        }
        i = i + 1;
        fi = fi + 1.0;
    }
}

fn sum_array(x: &[[f64; 10]; 10]) -> f64 {
    let mut sum: f64 = 0.0;
    let mut i: i64 = 0;
    while i < 10 {
        let mut j: i64 = 0;
        while j < 10 {
            sum = sum + x[i][j];
            if modulo(i * 10 + j, 20) == 0 {
                sum = sum + 20.0;
            }
            j = j + 1;
        }
        i = i + 1;
    }
    return sum;
}

fn kernel_adi(
    tsteps: i64,
    n: i64,
    x: &mut [[f64; 10]; 10],
    a: &[[f64; 10]; 10],
    b: &mut [[f64; 10]; 10],
) {
    let mut t: i64 = 0;
    while t < tsteps {
        // loop vars
        let mut i1: i64 = 0;
        let mut i2: i64 = 0;

        // loop 1
        i1 = 0;
        i2 = 1;
        while i1 < n {
            i2 = 1;
            while i2 < n {
                x[i1][i2] = x[i1][i2] - (x[i1][i2 - 1] * a[i1][i2] / b[i1][i2 - 1]);
                i2 = i2 + 1;
            }
            i1 = i1 + 1;
        }

        // loop 2
        i1 = 0;
        while i1 < n {
            x[i1][n - 1] /= b[i1][n - 1];
            i1 = i1 + 1;
        }

        // loop 3
        i1 = 0;
        i2 = 0;
        while i1 < n {
            i2 = 0;
            while i2 < n - 2 {
                x[i1][n - i2 - 2] =
                    (x[i1][n - i2 - 2] - x[i1][n - i2 - 3] * a[i1][n - i2 - 3]) / b[i1][n - i2 - 3];
                i2 = i2 + 1;
            }
            i1 = i1 + 1;
        }

        // loop 4
        i1 = 1;
        i2 = 0;
        while i1 < n {
            i2 = 0;
            while i2 < n {
                x[i1][i2] -= x[i1 - 1][i2] * a[i1][i2] / b[i1 - 1][i2];
                b[i1][i2] -= a[i1][i2] * a[i1][i2] / b[i1 - 1][i2];
                i2 = i2 + 1;
            }
            i1 = i1 + 1;
        }

        // loop 5
        i2 = 0;
        while i2 < n {
            x[n - 1][i2] /= b[n - 1][i2];
            i2 = i2 + 1;
        }

        // loop 6
        i1 = 0;
        i2 = 0;
        while i1 < n - 2 {
            i2 = 0;
            while i2 < n {
                x[n - i1 - 2][i2] =
                    (x[n - i1 - 2][i2] - x[n - i1 - 3][i2] * a[n - i1 - 3][i2]) / b[n - i1 - 2][i2];
                i2 = i2 + 1;
            }
            i1 = i1 + 1;
        }

        t = t + 1;
    }
}

fn main() {
    let tsteps: i64 = 50;
    let n: i64 = 1024;
    let nf: f64 = 1024.0;
    let mut x: [[f64; 1024]; 1024] = [[0.0; 1024]; 1024];
    let mut a: [[f64; 1024]; 1024] = [[0.0; 1024]; 1024];
    let mut b: [[f64; 1024]; 1024] = [[0.0; 1024]; 1024];

    init_array(n, nf, &mut x, &mut a, &mut b);
    kernel_adi(&mut x, &a, &mut b);
    let mut res: f64 = sum_array(&x);

    println!("{}", res);
}
