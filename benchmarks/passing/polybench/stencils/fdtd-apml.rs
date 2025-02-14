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
    ex: &mut [[f64; 1024]; 1024],
    ey: &mut [[f64; 1024]; 1024],
    hz: &mut [[f64; 1024]; 1024],
) {
    let n: i64 = 1024;
    let mut i: i64 = 0;
    let mut fi: f64 = 0.0;
    while i < n {
        let mut j: i64 = 0;
        let mut fj: f64 = 0.0;
        while j < n {
            ex[i][j] = (fi * (fj + 1.0)) / 1024.0;
            ey[i][j] = (fi * (fj + 2.0)) / 1024.0;
            hz[i][j] = (fi * (fj + 3.0)) / 1024.0;
            j += 1;
            fj += 1.0;
        }
        i += 1;
        fi += 1.0;
    }
}

fn sum_array(hz: &[[f64; 1024]; 1024]) -> f64 {
    let mut sum: f64 = 0.0;
    let mut i: i64 = 0;

    while i < 1024 {
        let mut j: i64 = 0;
        while j < 1024 {
            sum += hz[i][j];
            if modulo(i * 1024 + j, 20) == 0 {
                sum += 20.0;
            }
            j += 1;
        }
        i += 1;
    }
    return sum;
}

fn kernel_fdtd_apml(
    ex: &mut [[f64; 1024]; 1024],
    ey: &mut [[f64; 1024]; 1024],
    hz: &mut [[f64; 1024]; 1024],
) {
    let mut n: i64 = 1024;
    let mut t: i64 = 0;
    let mut i: i64 = 0;
    let mut j: i64 = 0;
    while t < 100 {
        // loop 1
        i = 1;
        while i < n {
            j = 0;
            while j < n {
                ex[i][j] -= 0.5 * (hz[i][j] - hz[i - 1][j]);
                j += 1;
            }
            i += 1;
        }

        // loop 2
        i = 0;
        while i < n {
            j = 1;
            while j < n {
                ey[i][j] -= 0.5 * (hz[i][j] - hz[i][j - 1]);
                j += 1;
            }
            i += 1;
        }

        // loop 3
        i = 0;
        while i < n - 1 {
            j = 0;
            while j < n - 1 {
                hz[i][j] -= 0.7 * (ex[i + 1][j] - ex[i][j] + ey[i][j + 1] - ey[i][j]);
                j += 1;
            }
            i += 1;
        }
        t += 1;
    }
}

fn main() {
    let mut ex: [[f64; 1024]; 1024] = [[0.0; 1024]; 1024];
    let mut ey: [[f64; 1024]; 1024] = [[0.0; 1024]; 1024];
    let mut hz: [[f64; 1024]; 1024] = [[0.0; 1024]; 1024];

    init_array(&mut ex, &mut ey, &mut hz);
    kernel_fdtd_apml(&mut ex, &mut ey, &mut hz);
    let res: f64 = sum_array(&hz);
    println!("{}", res);
}
