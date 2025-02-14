// STANDARD_DATASET parameters:
// #   define TSTEPS 100
// #   define N 10000

fn modulo(a: i64, b: i64) -> i64 {
    let div: i64 = a / b; // Integer division
    let remainder: i64 = a - (div * b); // Compute remainder manually
    if remainder < 0 {
        return remainder + b; // Ensure non-negative result
    } else {
        return remainder;
    }
}

fn init_array(a: &mut [f64; 10000], b: &mut [f64; 10000]) {
    let n: i64 = 10000;
    let nf: f64 = 10000.0;
    let mut i: i64 = 0;
    let mut fi: f64 = 0.0;
    while i < n {
        a[i] = (fi + 2.0) / nf;
        b[i] = (fi + 3.0) / nf;
        i += 1;
        fi += 1.0;
    }
}

fn sum_array(a: &[f64; 10000]) -> f64 {
    let n: i64 = 10000;
    let mut sum: f64 = 0.0;
    let mut i: i64 = 0;
    while i < n {
        sum += a[i];
        if modulo(i, 20) == 0 {
            sum += 20.0;
        }
        i += 1;
    }
    return sum;
}

fn kernel_jacobi_1d_imper(a: &mut [f64; 10000], b: &mut [f64; 10000]) {
    let n: i64 = 10000;
    let mut t: i64 = 0;
    while t < 100 {
        let mut i: i64 = 1;
        while i < n - 1 {
            b[i] = 0.33333 * (a[i - 1] + a[i] + a[i + 1]);
            i += 1;
        }
        let mut j: i64 = 1;
        while j < n - 1 {
            a[j] = b[j];
            j += 1;
        }

        t += 1;
    }
}

fn main() {
    let n: i64 = 10000;
    let mut a: [f64; 10000] = [0.0; 10000];
    let mut b: [f64; 10000] = [0.0; 10000];

    init_array(&mut a, &mut b);
    kernel_jacobi_1d_imper(&mut a, &mut b);
    let res: f64 = sum_array(&a);
    println!("{}", res);
}
