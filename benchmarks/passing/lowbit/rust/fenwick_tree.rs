// ARGS: 200001

fn main(n : i64) {
    let mut x : [i64; 200001] = [0; 200001];
    let mut i : i64 = 1;
    while i < n {
        let mut j : i64 = i;
        while j < n {
            x[j as usize] = x[j as usize] + 1;
            j = j + lowbit_naive(j);
        }
        i = i + 1;
    }
    let mut i : i64 = n - 1;
    let mut ans : i64 = 0;
    while i > 0 {
        let mut j : i64 = i;
        let mut sum : i64 = 0;
        while j > 0 {
            sum = sum + x[j as usize];
            j = j - lowbit_naive(j);
        }
        i = i - 1;
        ans = ans + sum;
    }
    drop(x);
    println!("{:}", ans);
}

fn lowbit_naive(a : i64) -> i64 {
    let mut n : i64 = a;
    let mut lb : i64 = 1;
    while n == n / 2 * 2 {
        n = n / 2;
        lb = lb * 2;
    }
    return lb;
}

fn lowbit(n : i64) -> i64 {
    let lb : i64 = n & (-n);
    return lb;
}