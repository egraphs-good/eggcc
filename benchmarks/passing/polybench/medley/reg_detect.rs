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
    sum_tang: &mut [[f64; 50]; 50],
    mean: &mut [[f64; 50]; 50],
    path: &mut [[f64; 50]; 50],
) {
    let mut i: i64 = 0;
    let mut fi: f64 = 0.0;
    while i < 50 {
        let mut j: i64 = 0;
        let mut fj: f64 = 0.0;
        while j < 50 {
            sum_tang[i][j] = (fi + 1.0) * (fj + 1.0);
            mean[i][j] = (fi - fj) / 50.0;
            path[i][j] = (fi * (fj - 1.0)) / 50.0;
            j += 1;
            fj += 1.0;
        }
        i += 1;
        fi += 1.0;
    }
}

fn sum_array(path: &[[f64; 50]; 50]) -> f64 {
    let mut sum: f64 = 0.0;
    let mut i: i64 = 0;

    while i < 50 {
        let mut j: i64 = 0;
        while j < 50 {
            sum += path[i][j];
            if modulo(i * 50 + j, 20) == 0 {
                sum += 20.0;
            }
            j += 1;
        }
        i += 1;
    }
    return sum;
}

fn kernel_reg_detect(
    sum_tang: &[[f64; 50]; 50],
    mean: &mut [[f64; 50]; 50],
    path: &mut [[f64; 50]; 50],
    diff: &mut [[[f64; 25]; 50]; 50],
    sum_diff: &mut [[[f64; 25]; 50]; 50],
) {
    let mut t: i64 = 0;
    let mut i: i64 = 0;
    let mut j: i64 = 0;
    let mut cnt: i64 = 0;
    while t < 10 {
        // loop 1
        j = 0;
        while j < 50 {
            i = j;
            while i < 50 {
                cnt = 0;
                while cnt < 25 {
                    diff[j][i][cnt] = sum_tang[j][i];
                    cnt += 1;
                }
                i += 1;
            }
            j += 1;
        }

        // loop 2
        j = 0;
        while j < 50 {
            i = j;
            while i < 50 {
                sum_diff[j][i][0] = diff[j][i][0];
                cnt = 0;
                while cnt < 25 {
                    sum_diff[j][i][cnt] = sum_diff[j][i][cnt - 1] + diff[j][i][cnt];
                    cnt += 1;
                }
                i += 1;
            }
            j += 1;
        }

        // loop 3
        i = 0;
        while i < 50 {
            path[0][i] = mean[0][i];
            i += 1;
        }

        // loop 4
        j = 1;
        while j < 50 {
            i = j;
            while i < 50 {
                path[j][i] = path[j - 1][i - 1] + mean[j][i];
                i += 1;
            }
            j += 1;
        }
        t += 1;
    }
}

fn main() {
    let mut sum_tang: [[f64; 50]; 50] = [[0.0; 50]; 50];
    let mut mean: [[f64; 50]; 50] = [[0.0; 50]; 50];
    let mut path: [[f64; 50]; 50] = [[0.0; 50]; 50];
    let mut diff: [[[f64; 25]; 50]; 50] = [[[0.0; 25]; 50]; 50];
    let mut sum_diff: [[[f64; 25]; 50]; 50] = [[[0.0; 25]; 50]; 50];

    init_array(&mut sum_tang, &mut mean, &mut path);
    kernel_reg_detect(&sum_tang, &mut mean, &mut path, &mut diff, &mut sum_diff);
    let res: f64 = sum_array(&path);

    println!("{}", res);
}
