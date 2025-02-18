fn init_array(n: i64, nf: f64, u: &mut [[f64; 200]; 200]) {
    let mut i: i64 = 0;
    let mut fi: f64 = 0.0;
    while i < n {
        let mut j: i64 = 0;
        let mut fj: f64 = 0.0;
        while j < n {
            u[i as usize][j as usize] = (fi + (nf - fj)) / nf;
            j += 1;
            fj += 1.0;
        }
        i += 1;
        fi += 1.0;
    }
}

fn sum_array(n: i64, u: &[[f64; 200]; 200]) -> f64 {
    let mut sum: f64 = 0.0;
    let mut i: i64 = 0;
    while i < n {
        let mut j: i64 = 0;
        while j < n {
            let x: f64 = u[i as usize][j as usize];
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

fn kernel_adi(
    tsteps: i64,
    tstepsf: f64,
    n: i64,
    nf: f64,
    u: &mut [[f64; 200]; 200],
    v: &mut [[f64; 200]; 200],
    p: &mut [[f64; 200]; 200],
    q: &mut [[f64; 200]; 200],
) {
    let dx: f64 = 1.0 / nf;
    let dy: f64 = 1.0 / nf;
    let dt: f64 = 1.0 / tstepsf;
    let b1: f64 = 2.0;
    let b2: f64 = 1.0;
    let mul1: f64 = b1 * dt / (dx * dx);
    let mul2: f64 = b2 * dt / (dy * dy);

    let a: f64 = -mul1 / 2.0;
    let b: f64 = 1.0 + mul1;
    let c: f64 = a;
    let d: f64 = -mul2 / 2.0;
    let e: f64 = 1.0 + mul2;
    let f: f64 = d;

    let mut t: i64 = 1;
    while t <= tsteps {
        let mut i: i64 = 1;
        while i < n - 1 {
            v[0][i as usize] = 1.0;
            p[i as usize][0] = 0.0;
            q[i as usize][0] = v[0][i as usize];
            let mut j: i64 = 1;
            while j < n - 1 {
                p[i as usize][j as usize] = -c / (a * p[i as usize][(j - 1) as usize] + b);
                q[i as usize][j as usize] = (-d * u[j as usize][(i - 1) as usize]
                    + (1.0 + 2.0 * d) * u[j as usize][i as usize]
                    - f * u[j as usize][(i + 1) as usize]
                    - a * q[i as usize][(j - 1) as usize])
                    / (a * p[i as usize][(j - 1) as usize] + b);
                j += 1;
            }
            i += 1;
        }
        t += 1;
    }
}

fn main() {
    let n: i64 = 200;
    let nf: f64 = 200.0;
    let tsteps: i64 = 100;
    let tstepsf: f64 = 100.0;

    let mut dummy: [f64; 200] = [0.0; 200];

    let mut u: [[f64; 200]; 200] = [dummy; 200];
    let mut v: [[f64; 200]; 200] = [dummy; 200];
    let mut p: [[f64; 200]; 200] = [dummy; 200];
    let mut q: [[f64; 200]; 200] = [dummy; 200];

    // Init
    let mut i: i64 = 0;
    while i < n {
        u[i] = [0.0; 200];
        v[i] = [0.0; 200];
        p[i] = [0.0; 200];
        q[i] = [0.0; 200];
        i += 1;
    }

    init_array(n, nf, &mut u);
    kernel_adi(tsteps, tstepsf, n, nf, &mut u, &mut v, &mut p, &mut q);
    let res: f64 = sum_array(n, &u);

    // Drop
    drop(dummy);
    i = 0;
    while i < n {
        drop(u[i]);
        drop(v[i]);
        drop(p[i]);
        drop(q[i]);
        i += 1;
    }
    drop(u);
    drop(v);
    drop(p);
    drop(q);

    println!("{}", res);
}
