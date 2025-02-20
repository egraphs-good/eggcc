fn init_array(n: i64, nf: f64, a: &mut [[f64; 90]; 90], b: &mut [[f64; 90]; 90]) {
    let mut i: i64 = 0;
    let mut fi: f64 = 0.0;
    while i < n {
        let mut j: i64 = 0;
        let mut fj: f64 = 0.0;
        while j < n {
            a[i as usize][j as usize] = ((fi * (fj + 2.0)) + 2.0) / nf;
            b[i as usize][j as usize] = ((fi * (fj + 3.0)) + 3.0) / nf;
            j += 1;
            fj += 1.0;
        }
        i += 1;
        fi += 1.0;
    }
}

fn sum_array(n: i64, a: &[[f64; 90]; 90]) -> f64 {
    let mut sum: f64 = 0.0;
    let mut i: i64 = 0;
    while i < n {
        let mut j: i64 = 0;
        while j < n {
            let x: f64 = a[i as usize][j as usize];
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

fn kernel_jacobi_2d(tsteps: i64, n: i64, a: &mut [[f64; 90]; 90], b: &mut [[f64; 90]; 90]) {
    let mut t: i64 = 0;
    while t < tsteps {
        let mut i: i64 = 1;
        while i < n - 1 {
            let mut j: i64 = 1;
            while j < n - 1 {
                b[i as usize][j as usize] = 0.2
                    * (a[i as usize][j as usize]
                        + a[i as usize][(j - 1) as usize]
                        + a[i as usize][(j + 1) as usize]
                        + a[(i + 1) as usize][j as usize]
                        + a[(i - 1) as usize][j as usize]);
                j += 1;
            }
            i += 1;
        }

        i = 1;
        while i < n - 1 {
            let mut j: i64 = 1;
            while j < n - 1 {
                a[i as usize][j as usize] = 0.2
                    * (b[i as usize][j as usize]
                        + b[i as usize][(j - 1) as usize]
                        + b[i as usize][(j + 1) as usize]
                        + b[(i + 1) as usize][j as usize]
                        + b[(i - 1) as usize][j as usize]);
                j += 1;
            }
            i += 1;
        }
        t += 1;
    }
}

fn main() {
    let n: i64 = 90;
    let mut nf: f64 = 90.0;
    let tsteps: i64 = 40;

    let mut dummy: [f64; 90] = [0.0; 90];

    let mut a: [[f64; 90]; 90] = [dummy; 90];
    let mut b: [[f64; 90]; 90] = [dummy; 90];

    // Init
    let mut i: i64 = 0;
    while i < n {
        a[i] = [0.0; 90];
        b[i] = [0.0; 90];
        i += 1;
    }

    init_array(n, nf, &mut a, &mut b);
    kernel_jacobi_2d(tsteps, n, &mut a, &mut b);
    let res: f64 = sum_array(n, &a);

    // Drop
    drop(dummy);
    i = 0;
    while i < n {
        drop(a[i]);
        drop(b[i]);
        i += 1;
    }
    drop(a);
    drop(b);

    println!("{}", res);
}
