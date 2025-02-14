// STANDARD_DATASET parameters:
// #   define TSTEPS 20
// #   define N 1000

fn modulo(a: i64, b: i64) -> i64 {
    let div: i64 = a / b; // Integer division
    let remainder: i64 = a - (div * b); // Compute remainder manually
    if remainder < 0 {
        return remainder + b; // Ensure non-negative result
    } else {
        return remainder;
    }
}

fn init_array_2d(a: &mut [[f64; 1000]; 1000], b: &mut [[f64; 1000]; 1000]) {
    let n: i64 = 1000;
    let nf: f64 = 1000.0;
    let mut i: i64 = 0;
    let mut fi: f64 = 0.0;
    while i < n {
        let mut j: i64 = 0;
        let mut fj: f64 = 0.0;
        while j < n {
            a[i][j] = (fi * (fj + 2.0) + 2.0) / nf;
            b[i][j] = (fi * (fj + 3.0) + 3.0) / nf;
            j += 1;
            fj += 1.0;
        }
        i += 1;
        fi += 1.0;
    }
}

fn sum_array_2d(a: &[[f64; 1000]; 1000]) -> f64 {
    let n: i64 = 1000;
    let mut sum: f64 = 0.0;
    let mut i: i64 = 0;

    while i < n {
        let mut j: i64 = 0;
        while j < n {
            sum += a[i][j];
            if modulo(i * n + j, 20) == 0 {
                sum += 20.0;
            }
            j += 1;
        }
        i += 1;
    }
    return sum;
}

fn kernel_jacobi_2d_imper(a: &mut [[f64; 1000]; 1000], b: &mut [[f64; 1000]; 1000]) {
    let n: i64 = 1000;
    let mut t: i64 = 0;
    let mut i: i64 = 0;
    let mut j: i64 = 0;
    while t < 20 {
        i = 1;
        while i < n - 1 {
            j = 1;
            while j < n - 1 {
                b[i][j] = 0.2 * (a[i][j] + a[i][j - 1] + a[i][j + 1] + a[i + 1][j] + a[i - 1][j]);
                j += 1;
            }
            i += 1;
        }

        i = 1;
        while i < n - 1 {
            j = 1;
            while j < n - 1 {
                a[i][j] = b[i][j];
                j += 1;
            }
            i += 1;
        }
        t += 1;
    }
}

fn main() {
    let mut a2d: [[f64; 1000]; 1000] = [[0.0; 1000]; 1000];
    let mut b2d: [[f64; 1000]; 1000] = [[0.0; 1000]; 1000];

    init_array_2d(&mut a2d, &mut b2d);
    kernel_jacobi_2d_imper(&mut a2d, &mut b2d);
    let res: f64 = sum_array_2d(&a2d);
    println!("{}", res);
}
