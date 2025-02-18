fn init_array(n: i64, nf: f64, a: &mut [[f64; 400]; 400]) {
    let mut i: i64 = 0;
    let mut fi: f64 = 0.0;
    while i < n {
        let mut j: i64 = 0;
        let mut fj: f64 = 0.0;
        while j < n {
            a[i as usize][j as usize] = (fi * (fj + 2.0) + 2.0) / nf;
            j += 1;
            fj += 1.0;
        }
        i += 1;
        fi += 1.0;
    }
}

fn sum_array(n: i64, a: &[[f64; 400]; 400]) -> f64 {
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

fn kernel_seidel_2d(tsteps: i64, n: i64, a: &mut [[f64; 400]; 400]) {
    let mut t: i64 = 0;
    while t < tsteps {
        let mut i: i64 = 1;
        while i < n - 1 {
            let mut j: i64 = 1;
            while j < n - 1 {
                a[i as usize][j as usize] = (a[(i - 1) as usize][(j - 1) as usize]
                    + a[(i - 1) as usize][j as usize]
                    + a[(i - 1) as usize][(j + 1) as usize]
                    + a[i as usize][(j - 1) as usize]
                    + a[i as usize][j as usize]
                    + a[i as usize][(j + 1) as usize]
                    + a[(i + 1) as usize][(j - 1) as usize]
                    + a[(i + 1) as usize][j as usize]
                    + a[(i + 1) as usize][(j + 1) as usize])
                    / 9.0;
                j += 1;
            }
            i += 1;
        }
        t += 1;
    }
}

fn main() {
    let n: i64 = 400;
    let nf: f64 = 400.0;
    let tsteps: i64 = 100;

    let dummy: [f64; 400] = [0.0; 400];
    let mut a: [[f64; 400]; 400] = [dummy; 400];

    // Init
    let mut i: i64 = 0;
    while i < n {
        a[i] = [0.0; 400];
        i += 1;
    }
    drop(dummy);

    init_array(n, nf, &mut a);
    kernel_seidel_2d(tsteps, n, &mut a);
    let res: f64 = sum_array(n, &a);

    // Drop
    i = 0;
    while i < n {
        drop(a[i]);
        i += 1;
    }
    drop(a);

    println!("{}", res);
}
