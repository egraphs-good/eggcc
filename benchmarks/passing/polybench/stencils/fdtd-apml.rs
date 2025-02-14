// STANDARD_DATASET parameters:
// #   define CZ 256
// #   define CYM 256
// #   define CXM 256

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
    cz: i64,
    czf: f64,
    cxm: i64,
    cxmf: f64,
    cym: i64,
    cymf: f64,
    ax: &mut [[f64; 257]; 257],
    ry: &mut [[f64; 257]; 257],
    ex: &mut [[[f64; 257]; 257]; 257],
    ey: &mut [[[f64; 257]; 257]; 257],
    hz: &mut [[[f64; 257]; 257]; 257],
    czm: &mut [f64; 257],
    czp: &mut [f64; 257],
    cxmh: &mut [f64; 257],
    cxph: &mut [f64; 257],
    cymh: &mut [f64; 257],
    cyph: &mut [f64; 257],
) {
    let mut i: i64 = 0;
    let mut fi: f64 = 0.0;

    i = 0;
    fi = 0.0;
    while i < cz {
        czm[i] = (fi + 1.0) / cxmf;
        czp[i] = (fi + 2.0) / cxmf;
        i += 1;
        fi += 1.0;
    }

    i = 0;
    fi = 0.0;
    while i < cxm {
        cxmh[i] = (fi + 3.0) / cxmf;
        cxph[i] = (fi + 4.0) / cxmf;
        i += 1;
        fi += 1.0;
    }

    i = 0;
    fi = 0.0;
    while i < cym {
        cymh[i] = (fi + 5.0) / cxmf;
        cyph[i] = (fi + 6.0) / cxmf;
        i += 1;
        fi += 1.0;
    }

    i = 0;
    fi = 0.0;
    while i < cz {
        let mut j: i64 = 0;
        let mut fj: f64 = 0.0;
        while j < cym {
            ry[i][j] = ((fi * (fj + 1.0)) + 10.0) / cymf;
            ax[i][j] = ((fi * (fj + 2.0)) + 11.0) / cymf;
            let mut k: i64 = 0;
            let mut fk: f64 = 0.0;
            while k < cxm {
                ex[i][j][k] = ((fi * (fj + 3.0)) + fk + 1.0) / cxmf;
                ey[i][j][k] = ((fi * (fj + 4.0)) + fk + 2.0) / cymf;
                hz[i][j][k] = ((fi * (fj + 5.0)) + fk + 3.0) / czf;
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

fn sum_array(
    cz: i64,
    cxm: i64,
    cym: i64,
    bza: &[[[f64; 257]; 257]; 257],
    ex: &[[[f64; 257]; 257]; 257],
    ey: &[[[f64; 257]; 257]; 257],
    hz: &[[[f64; 257]; 257]; 257],
) -> f64 {
    let mut sum_bza: f64 = 0.0;
    let mut sum_ex: f64 = 0.0;
    let mut sum_ey: f64 = 0.0;
    let mut sum_hz: f64 = 0.0;
    let mut mmod: f64 = 0.0;

    let mut i: i64 = 0;
    while i < cz {
        let mut j: i64 = 0;
        while j < cym {
            let mut k: i64 = 0;
            while k < cxm {
                sum_bza += bza[i][j][k];
                sum_ex += ex[i][j][k];
                sum_ey += ey[i][j][k];
                sum_hz += hz[i][j][k];
                if modulo(i * cxm + j, 20) == 0 {
                    mmod += 20.0;
                }
                k += 1;
            }
            j += 1;
        }
        i += 1;
    }

    return sum_bza + 2.0 * sum_ex + 3.0 * sum_ey + 4.0 * sum_hz + 5.0 * mmod;
}

fn kernel_fdtd_apml(
    cz: i64,
    cxm: i64,
    cym: i64,
    mui: f64,
    ch: f64,
    ax: &[[f64; 257]; 257],
    ry: &[[f64; 257]; 257],
    clf: &mut [[f64; 257]; 257],
    tmp: &mut [[f64; 257]; 257],
    bza: &mut [[[f64; 257]; 257]; 257],
    ex: &[[[f64; 257]; 257]; 257],
    ey: &[[[f64; 257]; 257]; 257],
    hz: &mut [[[f64; 257]; 257]; 257],
    czm: &[f64; 257],
    czp: &[f64; 257],
    cxmh: &[f64; 257],
    cxph: &[f64; 257],
    cymh: &[f64; 257],
    cyph: &[f64; 257],
) {
    let mut iz: i64 = 0;
    while iz < cz {
        let mut iy: i64 = 0;
        while iy < cym {
            let mut ix: i64 = 0;
            while ix < cxm {
                clf[iy][ix] =
                    ex[iz][iy][ix] - ex[iz][iy + 1][ix] + ey[iz][iy][ix + 1] - ey[iz][iy][ix];
                tmp[iy][ix] =
                    (cymh[iy] / cyph[iy]) * bza[iz][iy][ix] - (ch / cyph[iy]) * clf[iy][ix];
                hz[iz][iy][ix] = (cxmh[ix] / cxph[ix]) * hz[iz][iy][ix]
                    + (mui * czp[iz] / cxph[ix]) * tmp[iy][ix]
                    - (mui * czm[iz] / cxph[ix]) * bza[iz][iy][ix];
                bza[iz][iy][ix] = tmp[iy][ix];
                ix += 1;
            }
            clf[iy][cxm] = ex[iz][iy][cxm] - ex[iz][iy + 1][cxm] + ry[iz][iy] - ey[iz][iy][cxm];
            tmp[iy][cxm] =
                (cymh[iy] / cyph[iy]) * bza[iz][iy][cxm] - (ch / cyph[iy]) * clf[iy][cxm];
            hz[iz][iy][cxm] = (cxmh[cxm] / cxph[cxm]) * hz[iz][iy][cxm]
                + (mui * czp[iz] / cxph[cxm]) * tmp[iy][cxm]
                - (mui * czm[iz] / cxph[cxm]) * bza[iz][iy][cxm];
            bza[iz][iy][cxm] = tmp[iy][cxm];

            ix = 0;
            while ix < cxm {
                clf[iy][ix] = ex[iz][cym][ix] - ax[iz][ix] + ey[iz][cym][ix + 1] - ey[iz][cym][ix];
                tmp[iy][ix] =
                    (cymh[cym] / cyph[iy]) * bza[iz][iy][ix] - (ch / cyph[iy]) * clf[iy][ix];
                hz[iz][cym][ix] = (cxmh[ix] / cxph[ix]) * hz[iz][cym][ix]
                    + (mui * czp[iz] / cxph[ix]) * tmp[iy][ix]
                    - (mui * czm[iz] / cxph[ix]) * bza[iz][cym][ix];
                bza[iz][cym][ix] = tmp[iy][ix];
                ix += 1;
            }

            clf[iy][cxm] = ex[iz][cym][cxm] - ax[iz][cxm] + ry[iz][cym] - ey[iz][cym][cxm];
            tmp[iy][cxm] =
                (cymh[cym] / cyph[cym]) * bza[iz][cym][cxm] - (ch / cyph[cym]) * clf[iy][cxm];
            hz[iz][cym][cxm] = (cxmh[cxm] / cxph[cxm]) * hz[iz][cym][cxm]
                + (mui * czp[iz] / cxph[cxm]) * tmp[iy][cxm]
                - (mui * czm[iz] / cxph[cxm]) * bza[iz][cym][cxm];
            bza[iz][cym][cxm] = tmp[iy][cxm];
            iy += 1;
        }
        iz += 1;
    }
}

fn main() {
    let cz: i64 = 256;
    let czf: i64 = 256.0;
    let cxm: i64 = 256;
    let cxmf: i64 = 256.0;
    let cym: i64 = 256;
    let cymf: i64 = 256.0;

    // Variable declaration/allocation
    let mut mui: f64 = 2341.0;
    let mut ch: f64 = 42.0;
    let mut ax: [[f64; 257]; 257] = [[0.0; 257]; 257];
    let mut ry: [[f64; 257]; 257] = [[0.0; 257]; 257];
    let mut clf: [[f64; 257]; 257] = [[0.0; 257]; 257];
    let mut tmp: [[f64; 257]; 257] = [[0.0; 257]; 257];
    let mut bza: [[[f64; 257]; 257]; 257] = [[[0.0; 257]; 257]; 257];
    let mut ex: [[[f64; 257]; 257]; 257] = [[[0.0; 257]; 257]; 257];
    let mut ey: [[[f64; 257]; 257]; 257] = [[[0.0; 257]; 257]; 257];
    let mut hz: [[[f64; 257]; 257]; 257] = [[[0.0; 257]; 257]; 257];
    let mut czm: [f64; 257] = [0.0; 257];
    let mut czp: [f64; 257] = [0.0; 257];
    let mut cxmh: [f64; 257] = [0.0; 257];
    let mut cxph: [f64; 257] = [0.0; 257];
    let mut cymh: [f64; 257] = [0.0; 257];
    let mut cyph: [f64; 257] = [0.0; 257];

    // Initialize arrays
    init_array(
        cz, czf, cxm, cxmf, cym, cymf, &mut ax, &mut ry, &mut ex, &mut ey, &mut hz, &mut czm,
        &mut czp, &mut cxmh, &mut cxph, &mut cymh, &mut cyph,
    );

    // Run kernel
    kernel_fdtd_apml(
        cz, cxm, cym, mui, ch, &ax, &ry, &mut clf, &mut tmp, &mut bza, &ex, &ey, &mut hz, &czm,
        &mut czp, &mut cxmh, &mut cxph, &mut cymh, &mut cyph,
    );

    // Print result
    let res: f64 = sum_array(cz, cxm, cym, &bza, &ex, &ey, &hz);
    println!("{}", res);
}
