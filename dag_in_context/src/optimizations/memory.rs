#![allow(unused_imports)]
#![allow(dead_code)]

use crate::schema_helpers::{Constructor, ESort, Purpose};
use strum::IntoEnumIterator;

pub(crate) fn rules() -> Vec<String> {
    // ESort::iter()
    //     .map(|sort| "(relation BodyContains* (Expr *))".replace('*', sort.name()))
    //     .chain(Constructor::iter().filter_map(captured_expr_rule_for_ctor))
    //     .chain(Constructor::iter().filter_map(subexpr_rule_for_ctor))
    //     .collect::<Vec<_>>()
    vec![]
}

#[cfg(test)]
use crate::ast::*;
#[cfg(test)]
use crate::schema::Constant;
#[cfg(test)]
use crate::Value;

#[test]
fn test_memory() -> crate::Result {
    Ok(())
}
