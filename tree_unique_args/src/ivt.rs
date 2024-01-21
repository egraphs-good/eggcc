#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_ivt() {
        // do {
        //     if (x == 0) {
        //         (print 0);
        //     } else {
        //         (print 1);
        //     }
        // } while (x == 0);
        //
        // =>
        //
        // if (x == 0) {
        //     do {
        //         (print 0);
        //     } while (x == 0);
        // } else {
        //     (print 1)
        // }
    }
}
