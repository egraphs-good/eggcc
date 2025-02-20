fn init_array(m: i64, mf: f64, n: i64, data: &mut [[f64; 80]; 100]) {
    let mut i: i64 = 0;
    let mut fi: f64 = 0.0;
    while i < n {
        let mut j: i64 = 0;
        let mut fj: f64 = 0.0;
        while j < m {
            data[i as usize][j as usize] = (fi * fj) / mf;
            j += 1;
            fj += 1.0;
        }
        i += 1;
        fi += 1.0;
    }
}

fn sum_array(m: i64, cov: &[[f64; 80]; 80]) -> f64 {
    let mut sum: f64 = 0.0;
    let mut i: i64 = 0;
    while i < m {
        let mut j: i64 = 0;
        while j < m {
            let x: f64 = cov[i as usize][j as usize];
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

fn kernel_covariance(
    m: i64,
    n: i64,
    nf: f64,
    data: &mut [[f64; 80]; 100],
    cov: &mut [[f64; 80]; 80],
    mean: &mut [f64; 80],
) {
    let mut j: i64 = 0;
    while j < m {
        mean[j as usize] = 0.0;
        let mut i: i64 = 0;
        while i < n {
            let dij: f64 = data[i as usize][j as usize];
            mean[j as usize] = mean[j as usize] + dij;
            i += 1;
        }
        mean[j as usize] = mean[j as usize] / nf;
        j += 1;
    }

    let mut i: i64 = 0;
    while i < n {
        j = 0;
        while j < m {
            data[i as usize][j as usize] = data[i as usize][j as usize] - mean[j as usize];
            j += 1;
        }
        i += 1;
    }

    i = 0;
    while i < m {
        j = i;
        while j < m {
            cov[i as usize][j as usize] = 0.0;
            let mut k: i64 = 0;
            while k < n {
                cov[i as usize][j as usize] = cov[i as usize][j as usize]
                    + data[k as usize][i as usize] * data[k as usize][j as usize];
                k += 1;
            }
            cov[i as usize][j as usize] = cov[i as usize][j as usize] / (nf - 1.0);
            cov[j as usize][i as usize] = cov[i as usize][j as usize];
            j += 1;
        }
        i += 1;
    }
}

fn main() {
    let n: i64 = 100;
    let nf: f64 = 100.0;
    let m: i64 = 80;
    let mf: f64 = 80.0;
    let dummy: [f64; 80] = [0.0; 80];
    let mut data: [[f64; 80]; 100] = [dummy; 100];
    let mut cov: [[f64; 80]; 80] = [dummy; 80];
    let mut mean: [f64; 80] = [0.0; 80];

    // Init
    let mut i: i64 = 0;
    while i < n {
        data[i] = [0.0; 80];
        i += 1;
    }
    i = 0;
    while i < m {
        cov[i] = [0.0; 80];
        i += 1;
    }
    drop(dummy);

    init_array(m, mf, n, &mut data);
    kernel_covariance(m, n, nf, &mut data, &mut cov, &mut mean);
    let res: f64 = sum_array(m, &cov);

    // Drop
    i = 0;
    while i < n {
        drop(data[i]);
        i += 1;
    }
    i = 0;
    while i < m {
        drop(cov[i]);
        i += 1;
    }
    drop(data);
    drop(cov);
    drop(mean);

    println!("{}", res);
}
