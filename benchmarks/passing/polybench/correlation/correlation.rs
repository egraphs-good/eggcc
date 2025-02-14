// dimension 1000 x 1000 corresponds to STANDARD_DATASET size

fn fabs(x: f64) -> f64 {
    if x < 0.0 {
        return -x;
    } else {
        return x;
    }
}

fn sqrt(x: f64) -> f64 {
    if x < 0.0 {
        return -1.0;
    }
    if x == 0.0 {
        return 0.0;
    }

    let mut guess: f64 = (x + x) / x * 0.5; // Initial guess
    let mut prev_guess: f64 = x;

    while fabs(guess - prev_guess) > 0.0000000001 {
        prev_guess = guess;
        guess = (guess + x / guess) * 0.5; // Newton-Raphson iteration
    }
    return guess;
}

fn modulo(a: i64, b: i64) -> i64 {
    let div: i64 = a / b; // Integer division
    let remainder: i64 = a - (div * b); // Compute remainder manually
    if remainder < 0 {
        return remainder + b; // Ensure non-negative result
    } else {
        return remainder;
    }
}

fn init_array(m: i64, n: i64, data: &mut [[f64; 1000]; 1000]) {
    let mut i: i64 = 0;
    let mut fi: f64 = 0.0;
    while i < m {
        let mut j: i64 = 0;
        let mut fj: f64 = 0.0;
        while j < n {
            data[i][j] = (fi * fj) / 1000.0;
            j += 1;
            fj += 1.0;
        }
        i += 1;
        fi += 1.0;
    }
}

fn sum_array(m: i64, symmat: &[[f64; 1000]; 1000]) -> f64 {
    let mut sum: f64 = 0.0;
    let mut i: i64 = 0;

    while i < m {
        let mut j: i64 = 0;
        while j < m {
            sum += symmat[i][j];
            if modulo(i * m + j, 20) == 0 {
                sum += 20.0;
            }
            j += 1;
        }
        i += 1;
    }
    return sum;
}

fn kernel_correlation(
    m: i64,
    n: i64,
    float_n: f64,
    data: &mut [[f64; 1000]; 1000],
    symmat: &mut [[f64; 1000]; 1000],
    mean: &mut [f64; 1000],
    stddev: &mut [f64; 1000],
) {
    let eps: f64 = 0.1;

    let mut i: i64 = 0;
    let mut j: i64 = 0;
    // loop 1
    j = 0;
    while j < m {
        mean[j] = 0.0;
        i = 0;
        while i < n {
            mean[j] += data[i][j];
            i += 1;
        }
        mean[j] /= float_n;
        j += 1;
    }

    // loop 2
    j = 0;
    while j < m {
        stddev[j] = 0.0;
        i = 0;
        while i < n {
            let diff: f64 = data[i][j] - mean[j];
            let squared: f64 = diff * diff;
            stddev[j] += squared;
            i += 1;
        }
        stddev[j] /= float_n;
        stddev[j] = sqrt(stddev[j]);
        if stddev[j] <= eps {
            stddev[j] = 1.0;
        }
        j += 1;
    }

    // loop 3
    j = 0;
    while j < m {
        i = 0;
        while i < n {
            data[i][j] -= mean[j];
            data[i][j] /= sqrt(float_n) * stddev[j];
            i += 1;
        }
        j += 1;
    }

    // loop 4
    j = 0;
    while j < m - 1 {
        symmat[j][j] = 1.0;
        let mut j2: i64 = j + 1;
        while j2 < m {
            symmat[j][j2] = 0.0;
            i = 0;
            while i < n {
                symmat[j][j2] += data[i][j] * data[i][j2];
                i += 1;
            }
            symmat[j2][j] = symmat[j][j2];
            j2 += 1;
        }
        j += 1;
    }

    symmat[m - 1][m - 1] = 1.0;
}

fn main() {
    let mut data: [[f64; 1000]; 1000] = [[0.0; 1000]; 1000];
    let mut symmat: [[f64; 1000]; 1000] = [[0.0; 1000]; 1000];
    let mut mean: [f64; 1000] = [0.0; 1000];
    let mut stddev: [f64; 1000] = [0.0; 1000];

    init_array(1000, 1000, &mut data);

    let mut float_n: f64 = 1.2;

    kernel_correlation(
        1000,
        1000,
        float_n,
        &mut data,
        &mut symmat,
        &mut mean,
        &mut stddev,
    );

    let res: f64 = sum_array(1000, &symmat);
    println!("{}", res);
}
