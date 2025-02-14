// STANDARD_DATASET parameters:
// TSTEPS = 10000
// LENGTH = 50

fn modulo(a: i64, b: i64) -> i64 {
    let div: i64 = a / b; // Integer division
    let remainder: i64 = a - (div * b); // Compute remainder manually
    if remainder < 0 {
        return remainder + b; // Ensure non-negative result
    } else {
        return remainder;
    }
}

fn init_array(length: i64, c: &mut [[i64; 50]; 50], w: &mut [[i64; 50]; 50]) {
    let mut i: i64 = 0;
    while i < length {
        let mut j: i64 = 0;
        while j < length {
            c[i][j] = modulo(i * j, 2);
            w[i][j] = ((i - j) / length);
            j += 1;
        }
        i += 1;
    }
}

fn kernel_dynprog(
    tsteps: i64,
    length: i64,
    c: &mut [[i64; 50]; 50],
    w: &[[i64; 50]; 50],
    sum_c: &mut [[[i64; 50]; 50]; 50],
) -> i64 {
    let mut out_l: i64 = 0;
    let mut t: i64 = 0;
    let mut i: i64 = 0;
    let mut j: i64 = 0;
    while t < tsteps {
        i = 0;
        while i < length {
            j = 0;
            while j < length {
                c[i][j] = 0;
                j += 1;
            }
            i += 1;
        }

        i = 0;
        while i < length - 1 {
            j = i + 1;
            while j < length {
                let mut k: i64 = i + 1;
                while k < j {
                    sum_c[i][j][k] = sum_c[i][j][k - 1] + c[i][k] + c[k][j];
                    k += 1;
                }
                j += 1;
            }
            i += 1;
        }
        out_l += c[0][length - 1];
        t += 1;
    }

    return out_l;
}

fn main() {
    let length: i64 = 50;
    let tsteps: i64 = 10000;

    // Variable declaration/allocation
    let mut sum_c: [[i64; 50]; 50] = [[[0; 50]; 50]; 50];
    let mut c: [[i64; 50]; 50] = [[0; 50]; 50];
    let mut w: [[i64; 50]; 50] = [[0; 50]; 50];

    // Initialize arrays
    init_array(length, &mut c, &mut w);

    // Run kernel
    let res: i64 = kernel_dynprog(tsteps, length, &mut c, &w, &mut sum_c);

    // Print result
    println!("{}", res);
}
