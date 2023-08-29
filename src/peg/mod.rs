//! Convert bril programs to PEGs.
//!
//! # References
//!
//! * ["Equality Saturation: A New Approach to Optimization"](https://arxiv.org/abs/1012.1802)
//! by Tate, Stepp, Tatlock, and Lerner

use crate::cfg::{ret_id, Annotation, BranchOp, Cfg, CondVal, Identifier};
