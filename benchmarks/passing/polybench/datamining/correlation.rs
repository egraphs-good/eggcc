fn custom_sqrt(x: f64) -> f64 {
    let mut guess: f64 = x / 2.0;
    let mut i: i64 = 0;
    while i < 10 {
        guess = (guess + x / guess) / 2.0;
        i += 1;
    }
    return guess;
}

fn init_array(m: i64, mf: f64, n: i64, data: &mut [[f64; 80]; 100]) {
    let mut i: i64 = 0;
    let mut fi: f64 = 0.0;
    while i < n {
        let mut j: i64 = 0;
        let mut fj: f64 = 0.0;
        while j < m {
            let x: f64 = (fi * fj) / mf + fi;
            data[i as usize][j as usize] = x;
            j += 1;
            fj += 1.0;
        }
        i += 1;
        fi += 1.0;
    }
}

fn sum_array(m: i64, corr: &[[f64; 80]; 80]) -> f64 {
    let mut sum: f64 = 0.0;
    let mut i: i64 = 0;
    while i < m {
        let mut j: i64 = 0;
        while j < m {
            let x: f64 = corr[i as usize][j as usize];
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

fn kernel_correlation(
    m: i64,
    n: i64,
    nf: f64,
    data: &mut [[f64; 80]; 100],
    corr: &mut [[f64; 80]; 80],
    mean: &mut [f64; 80],
    stddev: &mut [f64; 80],
) {
    let eps: f64 = 0.1;
    let mut j: i64 = 0;
    while j < m {
        mean[j as usize] = 0.0;
        let mut i: i64 = 0;
        while i < n {
            mean[j as usize] = mean[j as usize] + data[i as usize][j as usize];

            i += 1;
        }

        mean[j as usize] = mean[j as usize] / nf;

        j += 1;
    }

    j = 0;
    while j < m {
        stddev[j as usize] = 0.0;
        let mut i: i64 = 0;
        while i < n {
            stddev[j as usize] = stddev[j as usize]
                + (data[i as usize][j as usize] - mean[j as usize])
                    * (data[i as usize][j as usize] - mean[j as usize]);
            i += 1;
        }
        stddev[j as usize] = stddev[j as usize] / nf;
        stddev[j as usize] = custom_sqrt(stddev[j as usize]);
        if stddev[j as usize] <= eps {
            stddev[j as usize] = 1.0;
        }
        j += 1;
    }

    let mut i: i64 = 0;
    while i < n {
        j = 0;
        while j < m {
            data[i as usize][j as usize] = data[i as usize][j as usize] - mean[j as usize];
            data[i as usize][j as usize] =
                data[i as usize][j as usize] / (custom_sqrt(nf) * stddev[j as usize]);
            j += 1;
        }
        i += 1;
    }

    i = 0;
    while i < m - 1 {
        corr[i as usize][i as usize] = 1.0;
        j = i + 1;
        while j < m {
            corr[i as usize][j as usize] = 0.0;
            let mut k: i64 = 0;
            while k < n {
                corr[i as usize][j as usize] = corr[i as usize][j as usize]
                    + data[k as usize][i as usize] * data[k as usize][j as usize];
                k += 1;
            }
            corr[j as usize][i as usize] = corr[i as usize][j as usize];
            j += 1;
        }
        i += 1;
    }
    corr[(m - 1) as usize][(m - 1) as usize] = 1.0;
}

fn main() {
    let n: i64 = 100;
    let nf: f64 = 100.0;
    let m: i64 = 80;
    let mf: f64 = 80.0;
    let mut dummy: [f64; 80] = [0.0; 80];
    let mut data: [[f64; 80]; 100] = [dummy; 100];
    let mut corr: [[f64; 80]; 80] = [dummy; 80];
    let mut mean: [f64; 80] = [0.0; 80];
    let mut stddev: [f64; 80] = [0.0; 80];

    // Init
    let mut i: i64 = 0;
    while i < n {
        data[i] = [0.0; 80];
        i += 1;
    }
    i = 0;
    while i < m {
        corr[i] = [0.0; 80];
        i += 1;
    }
    drop(dummy);

    init_array(m, mf, n, &mut data);
    kernel_correlation(m, n, nf, &mut data, &mut corr, &mut mean, &mut stddev);
    let res: f64 = sum_array(m, &corr);

    // Drop
    i = 0;
    while i < n {
        drop(data[i]);
        i += 1;
    }
    i = 0;
    while i < m {
        drop(corr[i]);
        i += 1;
    }
    drop(data);
    drop(corr);
    drop(mean);
    drop(stddev);

    println!("{}", res);
}
