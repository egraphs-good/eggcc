fn init_array(n: i64, nf: f64, a: &mut [[[f64; 40]; 40]; 40], b: &mut [[[f64; 40]; 40]; 40]) {
    let mut i: i64 = 0;
    let mut fi: f64 = 0.0;
    while i < n {
        let mut j: i64 = 0;
        let mut fj: f64 = 0.0;
        while j < n {
            let mut k: i64 = 0;
            let mut fk: f64 = 0.0;
            while k < n {
                a[i as usize][j as usize][k as usize] = (fi + fj + (nf - fk)) * 10.0 / nf;
                b[i as usize][j as usize][k as usize] = a[i as usize][j as usize][k as usize];
                k += 1;
                fk += 1.0;
            }
            j += 1;
            fj += 1.0;
        }
        i += 1;
        fi += 1.0;
    }
}

fn sum_array(n: i64, a: &[[[f64; 40]; 40]; 40]) -> f64 {
    let mut sum: f64 = 0.0;
    let mut i: i64 = 0;
    while i < n {
        let mut j: i64 = 0;
        while j < n {
            let mut k: i64 = 0;
            while k < n {
                let x: f64 = a[i as usize][j as usize][k as usize];
                // Can't print multiple things on nightly.
                // Uncomment to run locally to check output.
                // println!("{} ", x);
                sum += x;
                k += 1;
            }
            j += 1;
        }
        i += 1;
    }
    return sum;
}

fn kernel_heat_3d(
    tsteps: i64,
    n: i64,
    a: &mut [[[f64; 40]; 40]; 40],
    b: &mut [[[f64; 40]; 40]; 40],
) {
    let mut t: i64 = 1;
    while t <= tsteps {
        let mut i: i64 = 1;
        while i < n - 1 {
            let mut j: i64 = 1;
            while j < n - 1 {
                let mut k: i64 = 1;
                while k < n - 1 {
                    b[i as usize][j as usize][k as usize] = 0.125
                        * (a[(i + 1) as usize][j as usize][k as usize]
                            - 2.0 * a[i as usize][j as usize][k as usize]
                            + a[(i - 1) as usize][j as usize][k as usize])
                        + 0.125
                            * (a[i as usize][(j + 1) as usize][k as usize]
                                - 2.0 * a[i as usize][j as usize][k as usize]
                                + a[i as usize][(j - 1) as usize][k as usize])
                        + 0.125
                            * (a[i as usize][j as usize][(k + 1) as usize]
                                - 2.0 * a[i as usize][j as usize][k as usize]
                                + a[i as usize][j as usize][(k - 1) as usize])
                        + a[i as usize][j as usize][k as usize];
                    k += 1;
                }
                j += 1;
            }
            i += 1;
        }

        i = 1;
        while i < n - 1 {
            let mut j: i64 = 1;
            while j < n - 1 {
                let mut k: i64 = 1;
                while k < n - 1 {
                    a[i as usize][j as usize][k as usize] = 0.125
                        * (b[(i + 1) as usize][j as usize][k as usize]
                            - 2.0 * b[i as usize][j as usize][k as usize]
                            + b[(i - 1) as usize][j as usize][k as usize])
                        + 0.125
                            * (b[i as usize][(j + 1) as usize][k as usize]
                                - 2.0 * b[i as usize][j as usize][k as usize]
                                + b[i as usize][(j - 1) as usize][k as usize])
                        + 0.125
                            * (b[i as usize][j as usize][(k + 1) as usize]
                                - 2.0 * b[i as usize][j as usize][k as usize]
                                + b[i as usize][j as usize][(k - 1) as usize])
                        + b[i as usize][j as usize][k as usize];
                    k += 1;
                }
                j += 1;
            }
            i += 1;
        }
        t += 1;
    }
}

fn main() {
    let n: i64 = 40;
    let nf: f64 = 40.0;
    let tsteps: i64 = 400;

    let inner_dummy: [f64; 40] = [0.0; 40];
    let dummy: [[f64; 40]; 40] = [inner_dummy; 40];
    let mut a: [[[f64; 40]; 40]; 40] = [dummy; 40];
    let mut b: [[[f64; 40]; 40]; 40] = [dummy; 40];

    let mut i: i64 = 0;
    while i < n {
        let a_elt: [[f64; 40]; 40] = [inner_dummy; 40];
        let b_elt: [[f64; 40]; 40] = [inner_dummy; 40];
        let mut j: i64 = 0;
        while j < n {
            a_elt[j] = [0.0; 40];
            b_elt[j] = [0.0; 40];
            j += 1;
        }
        a[i] = a_elt;
        b[i] = b_elt;
        i += 1;
    }

    init_array(n, nf, &mut a, &mut b);
    kernel_heat_3d(tsteps, n, &mut a, &mut b);
    let res: f64 = sum_array(n, &a);

    // Drop
    drop(inner_dummy);
    drop(dummy);
    i = 0;
    while i < n {
        j = 0;
        while j < n {
            drop(a[i][j]);
            drop(b[i][j]);
            j += 1;
        }
        drop(a[i]);
        drop(b[i]);
        i += 1;
    }
    drop(a);
    drop(b);

    println!("{}", res);
}
