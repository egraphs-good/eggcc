// STANDARD_DATASET parameters:
// #   define NITER 10000
// #   define LENGTH 64
// #   define MAXGRID 6

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
    maxgrid: i64,
    maxgridf: f64,
    sum_tang: &mut [[f64; 6]; 6],
    mean: &mut [[f64; 6]; 6],
    path: &mut [[f64; 6]; 6],
) {
    let mut i: i64 = 0;
    let mut fi: f64 = 0.0;
    while i < maxgrid {
        let mut j: i64 = 0;
        let mut fj: f64 = 0.0;
        while j < maxgrid {
            sum_tang[i][j] = (fi + 1.0) * (fj + 1.0);
            mean[i][j] = (fi - fj) / maxgridf;
            path[i][j] = (fi * (fj - 1.0)) / maxgridf;
            j += 1;
            fj += 1.0;
        }
        i += 1;
        fi += 1.0;
    }
}

fn sum_array(maxgrid: i64, path: &[[f64; 6]; 6]) -> f64 {
    let mut sum: f64 = 0.0;
    let mut i: i64 = 0;

    while i < maxgrid {
        let mut j: i64 = 0;
        while j < maxgrid {
            sum += path[i][j];
            if modulo(i * maxgrid + j, 20) == 0 {
                sum += 20.0;
            }
            j += 1;
        }
        i += 1;
    }
    return sum;
}

fn kernel_reg_detect(
    niter: i64,
    maxgrid: i64,
    length: i64,
    sum_tang: &[[f64; 6]; 6],
    mean: &mut [[f64; 6]; 6],
    path: &mut [[f64; 6]; 6],
    diff: &mut [[[f64; 25]; 6]; 6],
    sum_diff: &mut [[[f64; 25]; 6]; 6],
) {
    let mut t: i64 = 0;
    let mut i: i64 = 0;
    let mut j: i64 = 0;
    let mut cnt: i64 = 0;
    while t < niter {
        // loop 1
        j = 0;
        while j < maxgrid {
            i = j;
            while i < maxgrid {
                cnt = 0;
                while cnt < length {
                    diff[j][i][cnt] = sum_tang[j][i];
                    cnt += 1;
                }
                i += 1;
            }
            j += 1;
        }

        // loop 2
        j = 0;
        while j < maxgrid {
            i = j;
            while i < maxgrid {
                sum_diff[j][i][0] = diff[j][i][0];
                cnt = 1;
                while cnt < length {
                    sum_diff[j][i][cnt] = sum_diff[j][i][cnt - 1] + diff[j][i][cnt];
                    cnt += 1;
                }
                i += 1;
            }
            j += 1;
        }

        // loop 3
        i = 0;
        while i < maxgrid {
            path[0][i] = mean[0][i];
            i += 1;
        }

        // loop 4
        j = 1;
        while j < maxgrid {
            i = j;
            while i < maxgrid {
                path[j][i] = path[j - 1][i - 1] + mean[j][i];
                i += 1;
            }
            j += 1;
        }
        t += 1;
    }
}

fn main() {
    let niter: i64 = 10000;
    let length: i64 = 64;
    let maxgrid: i64 = 6;
    let maxgridf: f64 = 6.0;
    let mut sum_tang: [[f64; 6]; 6] = [[0.0; 6]; 6];
    let mut mean: [[f64; 6]; 6] = [[0.0; 6]; 6];
    let mut path: [[f64; 6]; 6] = [[0.0; 6]; 6];
    let mut diff: [[[f64; 64]; 6]; 6] = [[[0.0; 64]; 6]; 6];
    let mut sum_diff: [[[f64; 64]; 6]; 6] = [[[0.0; 64]; 6]; 6];

    init_array(maxgrid, maxgridf, &mut sum_tang, &mut mean, &mut path);
    kernel_reg_detect(
        niter,
        maxgrid,
        length,
        &sum_tang,
        &mut mean,
        &mut path,
        &mut diff,
        &mut sum_diff,
    );
    let res: f64 = sum_array(maxgrid, &path);

    println!("{}", res);
}
