fn modulo(a: f64, b: f64) -> f64 {
    let mut remainder: f64 = a;
    while remainder >= b {
        remainder -= b;
    }
    while remainder < 0.0 {
        remainder += b;
    }
    return remainder;
}

fn init_array(w: i64, h: i64, img_in: &mut [[f64; 480]; 720], img_out: &mut [[f64; 480]; 720]) {
    let mut i: i64 = 0;
    let mut fi: f64 = 0.0;
    while i < w {
        let mut j: i64 = 0;
        let mut fj: f64 = 0.0;
        while j < h {
            img_in[i as usize][j as usize] = modulo((313.0 * fi + 991.0 * fj), 65536.0) / 65535.0;
            j += 1;
            fj += 1.0;
        }
        i += 1;
        fi += 1.0;
    }
}

fn sum_array(w: i64, h: i64, arr: &[[f64; 480]; 720]) -> f64 {
    let mut sum: f64 = 0.0;
    let mut i: i64 = 0;
    while i < w {
        let mut j: i64 = 0;
        while j < h {
            let x: f64 = arr[i as usize][j as usize];
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

fn custom_exp(x: f64) -> f64 {
    let mut result: f64 = 1.0;
    let mut term: f64 = 1.0;
    let mut n: i64 = 1;
    let mut nf: f64 = 1.0;
    while n < 20 {
        term *= x / nf;
        result += term;
        n += 1;
        nf += 1.0;
    }
    return result;
}

fn kernel_deriche(
    w: i64,
    h: i64,
    alpha: f64,
    img_in: &mut [[f64; 480]; 720],
    img_out: &mut [[f64; 480]; 720],
    y1: &mut [[f64; 480]; 720],
    y2: &mut [[f64; 480]; 720],
) {
    let k: f64 = (1.0 - custom_exp(-alpha)) * (1.0 - custom_exp(-alpha))
        / (1.0 + 2.0 * alpha * custom_exp(-alpha) - custom_exp(2.0 * alpha));
    let a1: f64 = k;
    let a2: f64 = k * custom_exp(-alpha) * (alpha - 1.0);
    let a3: f64 = k * custom_exp(-alpha) * (alpha + 1.0);
    let a4: f64 = -k * custom_exp(-2.0 * alpha);
    let a5: f64 = a1;
    let a6: f64 = a2;
    let a7: f64 = a3;
    let a480: f64 = a4;
    // Doing this in our subset of rust is kinda tricky...
    // Luckily, we know alpha is 0.25, so it's just 2^(-.25)
    // let b1: f64 = pow(2.0, -alpha);
    let b1: f64 = 0.84089641525;
    let b2: f64 = -custom_exp(-2.0 * alpha);
    let c1: f64 = 1.0;
    let c2: f64 = 1.0;

    let mut i: i64 = 0;
    while i < w {
        let mut ym1: f64 = 0.0;
        let mut ym2: f64 = 0.0;
        let mut xm1: f64 = 0.0;
        let mut j: i64 = 0;
        while j < h {
            y1[i as usize][j as usize] =
                a1 * img_in[i as usize][j as usize] + a2 * xm1 + b1 * ym1 + b2 * ym2;
            xm1 = img_in[i as usize][j as usize];
            ym2 = ym1;
            ym1 = y1[i as usize][j as usize];
            j += 1;
        }
        i += 1;
    }

    i = 0;
    while i < w {
        let mut xp1: f64 = 0.0;
        let mut xp2: f64 = 0.0;
        let mut yp1: f64 = 0.0;
        let mut yp2: f64 = 0.0;
        let mut j: i64 = h - 1;
        while j >= 0 {
            y2[i as usize][j as usize] = a3 * xp1 + a4 * xp2 + b1 * yp1 + b2 * yp2;

            // the following line doesn't work if you don't multiply by 1.0
            // this shouldn't be necessary!! rs2bril bug probably
            xp2 = xp1 * 1.0;
            xp1 = img_in[i as usize][j as usize];

            yp2 = yp1 * 1.0;
            yp1 = y2[i][j];

            j -= 1;
        }
        i += 1;
    }

    i = 0;
    while i < w {
        let mut j: i64 = 0;
        while j < h {
            img_out[i as usize][j as usize] =
                c1 * (y1[i as usize][j as usize] + y2[i as usize][j as usize]);
            j += 1;
        }
        i += 1;
    }

    let mut j: i64 = 0;
    while j < h {
        let mut tm1: f64 = 0.0;
        let mut ym1: f64 = 0.0;
        let mut ym2: f64 = 0.0;
        i = 0;
        while i < w {
            y1[i as usize][j as usize] =
                a5 * img_out[i as usize][j as usize] + a6 * tm1 + b1 * ym1 + b2 * ym2;
            tm1 = img_out[i as usize][j as usize] * 1.0;
            ym2 = ym1 * 1.0;
            ym1 = y1[i as usize][j as usize] * 1.0;
            i += 1;
        }
        j += 1;
    }

    j = 0;
    while j < h {
        let mut tp1: f64 = 0.0;
        let mut tp2: f64 = 0.0;
        let mut yp1: f64 = 0.0;
        let mut yp2: f64 = 0.0;
        i = w - 1;
        while i >= 0 {
            y2[i as usize][j as usize] = a7 * tp1 + a480 * tp2 + b1 * yp1 + b2 * yp2;
            tp2 = tp1 * 1.0;
            tp1 = img_out[i as usize][j as usize] * 1.0;
            yp2 = yp1 * 1.0;
            yp1 = y2[i as usize][j as usize] * 1.0;
            i -= 1;
        }
        j += 1;
    }

    i = 0;
    while i < w {
        let mut j: i64 = 0;
        while j < h {
            img_out[i as usize][j as usize] =
                c2 * (y1[i as usize][j as usize] + y2[i as usize][j as usize]);
            j += 1;
        }
        i += 1;
    }
}

fn main() {
    let w: i64 = 720;
    let h: i64 = 480;
    let mut alpha: f64 = 0.25;
    let dummy: [f64; 480] = [0.0; 480];
    let mut img_in: [[f64; 480]; 720] = [dummy; 720];
    let mut img_out: [[f64; 480]; 720] = [dummy; 720];
    let mut y1: [[f64; 480]; 720] = [dummy; 720];
    let mut y2: [[f64; 480]; 720] = [dummy; 720];

    // Init
    let mut i: i64 = 0;
    while i < w {
        img_in[i] = [0.0; 480];
        img_out[i] = [0.0; 480];
        y1[i] = [0.0; 480];
        y2[i] = [0.0; 480];
        i += 1;
    }
    drop(dummy);

    init_array(w, h, &mut img_in, &mut img_out);

    kernel_deriche(w, h, alpha, &img_in, &mut img_out, &mut y1, &mut y2);
    let res: f64 = sum_array(w, h, &img_out);

    // Drop
    i = 0;
    while i < w {
        drop(img_in[i]);
        drop(img_out[i]);
        drop(y1[i]);
        drop(y2[i]);
        i += 1;
    }
    drop(img_in);
    drop(img_out);
    drop(y1);
    drop(y2);

    println!("{}", res);
}
