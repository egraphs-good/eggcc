//! Convert tree programs to RVSDGs

use tree_in_context::schema::{BaseType, RcExpr, TreeProgram, Type};

use super::{RvsdgBody, RvsdgFunction, RvsdgProgram, RvsdgType};

struct TreeToRvsdg {
    nodes: Vec<RvsdgBody>,
}

pub(crate) fn tree_to_rvsdg(tree: TreeProgram) -> RvsdgProgram {
    let mut res = RvsdgProgram { functions: vec![] };
    for func in tree.functions {
        res.functions.push(tree_func_to_rvsdg(func));
    }
    res.functions.push(tree_func_to_rvsdg(tree.entry));
    res
}

fn bril_type_from_type(ty: Type) -> bril_rs::Type {
    match ty {
        Type::Base(base_ty) => match base_ty {
            BaseType::IntT => bril_rs::Type::Int,
            BaseType::BoolT => bril_rs::Type::Bool,
        },
        Type::PointerT(ty) => {
            let base_ty = bril_type_from_type(Type::Base(ty));
            bril_rs::Type::Pointer(Box::new(base_ty))
        }
        Type::TupleT(_) => panic!("Tuple types not supported in RVSDG"),
        Type::Unknown => panic!("Unknown type in tree_type_to_rvsdg_types"),
    }
}

fn rvsdg_types_from_tuple_type(ty: Type) -> Vec<RvsdgType> {
    let Type::TupleT(tys) = ty else {
        panic!("Expected tuple type in tree_type_to_rvsdg_types")
    };
    tys.into_iter()
        .map(|ty| RvsdgType::Bril(bril_type_from_type(ty)))
        .collect()
}

fn tree_func_to_rvsdg(func: RcExpr) -> RvsdgFunction {
    let types = func.func_output_ty().expect("Expected function types");

    let mut res = RvsdgFunction {
        name: func
            .func_name()
            .expect("Expected function in tree_func_to_rvsdg"),
        args: todo!(),
        nodes: todo!(),
        results: todo!(),
    };
}
