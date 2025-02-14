// // STANDARD_DATASET parameters:
// #   define TMAX 50
// #   define NX 1000
// #   define NY 1000

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
    tmax: i64,
    nx: i64,
    nxf: f64,
    ny: i64,
    nyf: f64,
    ex: &mut [[f64; 1000]; 1000],
    ey: &mut [[f64; 1000]; 1000],
    hz: &mut [[f64; 1000]; 1000],
    fict: &mut [f64; 50],
) {
    let mut i: i64 = 0;
    let mut fi: f64 = 0.0;
    while i < tmax {
        fict[i] = fi;
        i = i + 1;
        fi = fi + 1.0;
    }
    i = 0;
    fi = 0.0;
    while i < nx {
        let mut j: i64 = 0;
        let mut fj: f64 = 0.0;
        while j < ny {
            ex[i][j] = (fi * (fj + 1.0)) / nxf;
            ey[i][j] = (fi * (fj + 2.0)) / nyf;
            hz[i][j] = (fi * (fj + 3.0)) / nxf;
            j = j + 1;
            fj = fj + 1.0;
        }
        i = i + 1;
        fi = fi + 1.0;
    }
}

fn sum_array(
    nx: i64,
    ny: i64,
    ex: &mut [[f64; 1000]; 1000],
    ey: &mut [[f64; 1000]; 1000],
    hz: &mut [[f64; 1000]; 1000],
) -> f64 {
    let mut sum_ex: f64 = 0.0;
    let mut sum_ey: f64 = 0.0;
    let mut sum_hz: f64 = 0.0;
    let mut mod_adjust: f64 = 0.0;
    let mut i: i64 = 0;
    while i < nx {
        let mut j: i64 = 0;
        while j < ny {
            sum_ex = sum_ex + ex[i][j];
            sum_ey = sum_ey + ey[i][j];
            sum_hz = sum_hz + hz[i][j];
            if modulo(i * nx + j, 20) == 0 {
                mod_adjust = mod_adjust + 20.0;
            }
            j = j + 1;
        }
        i = i + 1;
    }
    return sum_ex + 2.0 * sum_ey + 3.0 * sum_hz + 4.0 * mod_adjust;
}

fn kernel_fdtd_2d(
    tmax: i64,
    nx: i64,
    ny: i64,
    ex: &mut [[f64; 1000]; 1000],
    ey: &mut [[f64; 1000]; 1000],
    hz: &mut [[f64; 1000]; 1000],
    fict: &mut [f64; 50],
) {
    let mut t: i64 = 0;
    while t < tmax {
        let mut j: i64 = 0;
        while j < ny {
            ey[0][j] = fict[t];
            j = j + 1;
        }

        let mut i: i64 = 0;
        let mut fi: f64 = 0.0;
        while i < nx {
            j = 0;
            while j < ny {
                ey[i][j] -= 0.5 * (hz[i][j] - hz[i - 1][j]);
                j = j + 1;
            }
            i = i + 1;
        }

        i = 0;
        while i < nx {
            j = 0;
            while j < ny {
                ex[i][j] -= 0.5 * (hz[i][j] - hz[i][j - 1]);
                j = j + 1;
            }
            i = i + 1;
        }

        i = 0;
        while i < nx - 1 {
            j = 0;
            while j < ny - 1 {
                hz[i][j] -= 0.7 * (ex[i][j + 1] - ex[i][j] + ey[i + 1][j] - ey[i][j]);
                j = j + 1;
            }
            i = i + 1;
        }

        t = t + 1;
    }
}

fn main() {
    let tmax: i64 = 50;
    let nx: i64 = 1000;
    let ny: i64 = 1000;

    let nxf: f64 = 1000.0;
    let nyf: f64 = 1000.0;

    let mut ex: [[f64; 1000]; 1000] = [[0.0; 1000]; 1000];
    let mut ey: [[f64; 1000]; 1000] = [[0.0; 1000]; 1000];
    let mut hz: [[f64; 1000]; 1000] = [[0.0; 1000]; 1000];
    let mut fict: [f64; 50] = [0.0; 50];

    init_array(tmax, nx, nxf, ny, nyf, &mut ex, &mut ey, &mut hz, &mut fict);

    kernel_fdtd_2d(tmax, nx, ny, &mut ex, &mut ey, &mut hz, &fict);

    let res: f64 = sum_array(nx, ny, &ex, &ey, &hz);
    println!("{}", res);
}
