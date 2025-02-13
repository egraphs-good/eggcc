fn modulo(a: i64, b: i64) -> i64 {
    let div: i64 = a / b; // Integer division
    let remainder: i64 = a - (div * b); // Compute remainder manually
    if remainder < 0 {
        return remainder + b; // Ensure non-negative result
    } else {
        return remainder;
    }
}

fn init_array(n: i64, nf: f64, a: &mut [[f64; 20]; 20]) {
    let mut i: i64 = 0;
    let mut i_float: f64 = 0.0;
    while i < n {
        let mut j: i64 = 0;
        let mut jf: f64 = 0.0;
        while j < n {
            a[i][j] = ((i_float * (jf + 2.0) + 2.0) / nf);
            j = j + 1;
            jf = jf + 1.0;
        }
        i = i + 1;
        i_float = i_float + 1.0;
    }
}

fn sum_array(n: i64, a: &[[f64; 20]; 20]) -> f64 {
    let mut i: i64 = 0;
    let mut sum: f64 = 0.0;
    while i < n {
        let mut j: i64 = 0;
        while j < n {
            let x: f64 = a[i][j];
            sum += x;
            if modulo((i * n + j), 20) == 0 {
                sum += 10.0;
            }
            j = j + 1;
        }
        i = i + 1;
    }
    return sum;
}

fn kernel_seidel_2d(tsteps: i64, n: i64, a: &mut [[f64; 20]; 20]) {
    let mut k: i64 = 0;
    while k < tsteps {
        let mut i: i64 = 1;
        while i < (n - 1) {
            let mut j: i64 = 1;
            while j < (n - 1) {
                a[i][j] = (a[i - 1][j - 1]
                    + a[i - 1][j]
                    + a[i - 1][j + 1]
                    + a[i][j - 1]
                    + a[i][j]
                    + a[i][j + 1]
                    + a[i + 1][j - 1]
                    + a[i + 1][j]
                    + a[i + 1][j + 1])
                    / 9.0;
                j = j + 1;
            }
            i = i + 1;
        }
        k = k + 1;
    }
}

fn main() {
    let n: i64 = 20;
    let nf: f64 = 20.0;
    let tsteps: i64 = 1000;

    let mut a: [[f64; 20]; 20] = [[0.0; 20]; 20];

    let res: f64 = 1.1;

    init_array(n, nf, &mut a);

    kernel_seidel_2d(tsteps, n, &mut a);

    let res: f64 = sum_array(n, &a);

    println!("{}", res);
}
