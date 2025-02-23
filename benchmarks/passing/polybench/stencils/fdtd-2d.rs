fn init_array(
    tmax: i64,
    nx: i64,
    nxf: f64,
    ny: i64,
    nyf: f64,
    fict: &mut [f64; 40],
    ex: &mut [[f64; 80]; 60],
    ey: &mut [[f64; 80]; 60],
    hz: &mut [[f64; 80]; 60],
) {
    let mut i: i64 = 0;
    let mut fi: f64 = 0.0;
    while i < tmax {
        fict[i as usize] = fi;
        i += 1;
        fi += 1.0;
    }

    i = 0;
    fi = 0.0;
    while i < nx {
        let mut j: i64 = 0;
        let mut fj: f64 = 0.0;
        while j < ny {
            ex[i as usize][j as usize] = (fi * (fj + 1.0)) / nxf;
            ey[i as usize][j as usize] = (fi * (fj + 2.0)) / nyf;
            hz[i as usize][j as usize] = (fi * (fj + 3.0)) / nxf;
            j += 1;
            fj += 1.0;
        }
        i += 1;
        fi += 1.0;
    }
}

fn kernel_fdtd_2d(
    tmax: i64,
    nx: i64,
    ny: i64,
    fict: &[f64; 40],
    ex: &mut [[f64; 80]; 60],
    ey: &mut [[f64; 80]; 60],
    hz: &mut [[f64; 80]; 60],
) {
    let mut t: i64 = 0;
    while t < tmax {
        let mut j: i64 = 0;
        while j < ny {
            ey[0][j as usize] = fict[t as usize];
            j += 1;
        }

        let mut i: i64 = 1;
        while i < nx {
            j = 0;
            while j < ny {
                ey[i as usize][j as usize] = ey[i as usize][j as usize]
                    - (0.5 * (hz[i as usize][j as usize] - hz[(i - 1) as usize][j as usize]));
                j += 1;
            }
            i += 1;
        }

        i = 0;
        while i < nx {
            j = 1;
            while j < ny {
                ex[i as usize][j as usize] = ex[i as usize][j as usize]
                    - (0.5 * (hz[i as usize][j as usize] - hz[i as usize][(j - 1) as usize]));
                j += 1;
            }
            i += 1;
        }

        i = 0;
        while i < nx - 1 {
            j = 0;
            while j < ny - 1 {
                hz[i as usize][j as usize] = hz[i as usize][j as usize]
                    - (0.7
                        * (ex[i as usize][(j + 1) as usize] - ex[i as usize][j as usize]
                            + ey[(i + 1) as usize][j as usize]
                            - ey[i as usize][j as usize]));
                j += 1;
            }
            i += 1;
        }

        t += 1;
    }
}

fn sum_array(
    nx: i64,
    ny: i64,
    ex: &[[f64; 80]; 60],
    ey: &[[f64; 80]; 60],
    hz: &[[f64; 80]; 60],
) -> f64 {
    let mut sum: f64 = 0.0;
    let mut i: i64 = 0;
    let mut j: i64 = 0;
    while i < nx {
        while j < ny {
            let x: f64 = ex[i as usize][j as usize];
            // Can't print multiple things on nightly.
            // Uncomment to run locally to check output.
            // println!("{} ", x);
            sum += x;
            j += 1;
        }
        i += 1;
    }

    i = 0;
    j = 0;
    while i < nx {
        while j < ny {
            let x: f64 = ey[i as usize][j as usize];
            // Can't print multiple things on nightly.
            // Uncomment to run locally to check output.
            // println!("{} ", x);
            sum += x;
            j += 1;
        }
        i += 1;
    }

    i = 0;
    j = 0;
    while i < nx {
        while j < ny {
            let x: f64 = hz[i as usize][j as usize];
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

fn main() {
    let tmax: i64 = 40;
    let nx: i64 = 60;
    let nxf: f64 = 60.0;
    let ny: i64 = 80;
    let nyf: f64 = 80.0;

    let dummy: [f64; 80] = [0.0; 80];
    let mut fict: [f64; 40] = [0.0; 40];
    let mut ex: [[f64; 80]; 60] = [dummy; 60];
    let mut ey: [[f64; 80]; 60] = [dummy; 60];
    let mut hz: [[f64; 80]; 60] = [dummy; 60];

    // Init
    let mut i: i64 = 0;
    while i < nx {
        ex[i] = [0.0; 80];
        ey[i] = [0.0; 80];
        hz[i] = [0.0; 80];
        i += 1;
    }
    drop(dummy);

    init_array(tmax, nx, nxf, ny, nyf, &mut fict, &mut ex, &mut ey, &mut hz);
    kernel_fdtd_2d(tmax, nx, ny, &fict, &mut ex, &mut ey, &mut hz);
    let res: f64 = sum_array(nx, ny, &ex, &ey, &hz);

    // Drop
    i = 0;
    while i < nx {
        drop(ex[i]);
        drop(ey[i]);
        drop(hz[i]);
        i += 1;
    }
    drop(fict);
    drop(ex);
    drop(ey);
    drop(hz);

    println!("{}", res);
}
