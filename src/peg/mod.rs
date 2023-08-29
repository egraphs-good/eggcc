//! Convert bril programs to PEGs.
//!
//! # References
//!
//! * ["Equality Saturation: A New Approach to Optimization"](https://arxiv.org/abs/1012.1802)
//! by Tate, Stepp, Tatlock, and Lerner

use crate::rvsdg::{Expr, Operand};

/// Define PEGs in terms of RVSDG units.
#[allow(dead_code)]
pub(crate) enum Peg {
    PureOp(Expr),
    Phi {
        pred: Operand,
        if_true: Operand,
        if_false: Operand,
    },
    Theta {
        init: Operand,
        loop_: Operand,
    },
}
