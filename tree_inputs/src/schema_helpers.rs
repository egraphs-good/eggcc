use std::fmt::{Display, Formatter};

use crate::schema::Constant;

impl Display for Constant {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Constant::Int(i) => write!(f, "{}", i),
            Constant::Bool(b) => write!(f, "{}", b),
        }
    }
}
