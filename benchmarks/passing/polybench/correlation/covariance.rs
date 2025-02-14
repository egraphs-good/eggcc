// dimension 1000 x 1000 corresponds to STANDARD_DATASET size

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

fn kernel_covariance(
    m: i64,
    n: i64,
    float_n: f64,
    data: &mut [[f64; 1000]; 1000],
    symmat: &mut [[f64; 1000]; 1000],
    mean: &mut [f64; 1000],
) {
    let mut j: i64 = 0;
    let mut i: i64 = 0;
    // Determine mean of column vectors
    j = 0;
    while j < m {
        i = 0;
        while i < n {
            mean[j] += data[i][j];
            i += 1;
        }
        mean[j] /= float_n;
        j += 1;
    }

    // Center the column vectors
    i = 0;
    while i < n {
        j = 0;
        while j < m {
            data[i][j] -= mean[j];
            j += 1;
        }
        i += 1;
    }

    // Calculate the m * m covariance matrix
    j = 0;
    while j < m {
        let mut j2: i64 = j;
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
}

fn main() {
    let n: i64 = 1000;
    let m: i64 = 1000;

    // Variable declaration/allocation
    let mut data: [[f64; 1000]; 1000] = [[0.0; 1000]; 1000];
    let mut symmat: [[f64; 1000]; 1000] = [[0.0; 1000]; 1000];
    let mut mean: [[f64; 1000]; 1000] = [0.0; 1000];

    // Initialize arrays
    init_array(m, n, &mut data);
    let mut float_n: f64 = 1.2;

    // Run kernel
    kernel_covariance(m, n, float_n, &mut data, &mut symmat, &mut mean);

    // // Print result
    let res: f64 = sum_array(m, &symmat);
    println!("{}", res);
}
