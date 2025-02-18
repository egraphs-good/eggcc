fn init_array(n: i64, nf: f64, a: &mut [f64; 400], b: &mut [f64; 400]) {
    let mut i: i64 = 0;
    let mut fi: f64 = 0.0;
    while i < n {
        a[i as usize] = (fi + 2.0) / nf;
        b[i as usize] = (fi + 3.0) / nf;
        i += 1;
        fi += 1.0;
    }
}

fn sum_array(n: i64, a: &[f64; 400]) -> f64 {
    let mut i: i64 = 0;
    let mut sum: f64 = 0.0;
    while i < n {
        let x: f64 = a[i as usize];
        // Can't print multiple things on nightly.
        // Uncomment to run locally to check output.
        // println!("{} ", x);
        sum += x;
        i += 1;
    }
    return sum;
}

fn kernel_jacobi_1d(tsteps: i64, n: i64, a: &mut [f64; 400], b: &mut [f64; 400]) {
    let mut t: i64 = 0;
    while t < tsteps {
        let mut i: i64 = 1;
        while i < n - 1 {
            b[i as usize] = 0.33333 * (a[(i - 1) as usize] + a[i as usize] + a[(i + 1) as usize]);
            i += 1;
        }
        i = 1;
        while i < n - 1 {
            a[i as usize] = 0.33333 * (b[(i - 1) as usize] + b[i as usize] + b[(i + 1) as usize]);
            i += 1;
        }
        t += 1;
    }
}

fn main() {
    let n: i64 = 400;
    let nf: f64 = 400.0;
    let tsteps: i64 = 100;

    let mut a: [f64; 400] = [0.0; 400];
    let mut b: [f64; 400] = [0.0; 400];

    init_array(n, nf, &mut a, &mut b);
    kernel_jacobi_1d(tsteps, n, &mut a, &mut b);

    let res: f64 = sum_array(n, &a);

    drop(a);
    drop(b);

    println!("{}", res);
}
