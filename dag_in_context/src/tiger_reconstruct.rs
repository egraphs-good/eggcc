use crate::schema::{RcExpr, TreeProgram};
use crate::tiger_extractor_core::TigerExtractor;
use crate::tiger_extractor_types::{TigerExtraction, TigerExtractionResult};
use crate::tiger_format::TigerEGraph;
use egraph_serialize::EGraph;
use indexmap::IndexMap;
use std::rc::Rc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TigerReconstructError {
    #[error("missing extraction for function root {0}")]
    MissingExtraction(String),
    #[error("no root index in extraction for function root {0}")]
    MissingRoot(String),
    #[error("unexpected: child index out of range in extraction graph")]
    ChildIndexRange,
    #[error("unsupported operation or arity: {0}")]
    UnsupportedHead(String),
}

fn build_expr_from_extraction(
    serialized: &EGraph,
    tiger: &TigerEGraph,
    extraction: &TigerExtraction,
) -> Result<RcExpr, TigerReconstructError> {
    use crate::schema::Expr::*;
    use crate::schema::{
        Assumption, BaseType, BinaryOp, Constant as SchemaConstant, TernaryOp, Type as SchemaType,
        UnaryOp,
    };
    use ordered_float::OrderedFloat;

    #[derive(Clone, Debug)]
    enum BuiltValue {
        Expr(RcExpr),
        Type(SchemaType),
        BaseType(BaseType),
        TypeList(Vec<BaseType>),
        ListExpr(Vec<RcExpr>),
        Assumption(Assumption),
        Constant(SchemaConstant),
        BinaryOp(BinaryOp),
        UnaryOp(UnaryOp),
        TernaryOp(TernaryOp),
        Int(i64),
        Bool(bool),
        Float(OrderedFloat<f64>),
        String(String),
    }

    fn expect_value<'a>(
        values: &'a [Option<BuiltValue>],
        idx: usize,
    ) -> Result<&'a BuiltValue, TigerReconstructError> {
        values
            .get(idx)
            .and_then(|v| v.as_ref())
            .ok_or(TigerReconstructError::ChildIndexRange)
    }

    fn expect_expr(
        values: &[Option<BuiltValue>],
        idx: usize,
    ) -> Result<RcExpr, TigerReconstructError> {
        match expect_value(values, idx)? {
            BuiltValue::Expr(expr) => Ok(expr.clone()),
            other => Err(TigerReconstructError::UnsupportedHead(format!(
                "expected Expr child at index {idx}, found {other:?}",
            ))),
        }
    }

    fn expect_type(
        values: &[Option<BuiltValue>],
        idx: usize,
    ) -> Result<SchemaType, TigerReconstructError> {
        match expect_value(values, idx)? {
            BuiltValue::Type(ty) => Ok(ty.clone()),
            other => Err(TigerReconstructError::UnsupportedHead(format!(
                "expected Type child at index {idx}, found {other:?}",
            ))),
        }
    }

    fn expect_base_type(
        values: &[Option<BuiltValue>],
        idx: usize,
    ) -> Result<BaseType, TigerReconstructError> {
        match expect_value(values, idx)? {
            BuiltValue::BaseType(bt) => Ok(bt.clone()),
            other => Err(TigerReconstructError::UnsupportedHead(format!(
                "expected BaseType child at index {idx}, found {other:?}",
            ))),
        }
    }

    fn expect_type_list(
        values: &[Option<BuiltValue>],
        idx: usize,
    ) -> Result<Vec<BaseType>, TigerReconstructError> {
        match expect_value(values, idx)? {
            BuiltValue::TypeList(list) => Ok(list.clone()),
            other => Err(TigerReconstructError::UnsupportedHead(format!(
                "expected TypeList child at index {idx}, found {other:?}",
            ))),
        }
    }

    fn expect_list_expr(
        values: &[Option<BuiltValue>],
        idx: usize,
    ) -> Result<Vec<RcExpr>, TigerReconstructError> {
        match expect_value(values, idx)? {
            BuiltValue::ListExpr(list) => Ok(list.clone()),
            other => Err(TigerReconstructError::UnsupportedHead(format!(
                "expected ListExpr child at index {idx}, found {other:?}",
            ))),
        }
    }

    fn expect_assumption(
        values: &[Option<BuiltValue>],
        idx: usize,
    ) -> Result<Assumption, TigerReconstructError> {
        match expect_value(values, idx)? {
            BuiltValue::Assumption(a) => Ok(a.clone()),
            other => Err(TigerReconstructError::UnsupportedHead(format!(
                "expected Assumption child at index {idx}, found {other:?}",
            ))),
        }
    }

    fn expect_constant(
        values: &[Option<BuiltValue>],
        idx: usize,
    ) -> Result<SchemaConstant, TigerReconstructError> {
        match expect_value(values, idx)? {
            BuiltValue::Constant(c) => Ok(c.clone()),
            other => Err(TigerReconstructError::UnsupportedHead(format!(
                "expected Constant child at index {idx}, found {other:?}",
            ))),
        }
    }

    fn expect_binary_op(
        values: &[Option<BuiltValue>],
        idx: usize,
    ) -> Result<BinaryOp, TigerReconstructError> {
        match expect_value(values, idx)? {
            BuiltValue::BinaryOp(op) => Ok(op.clone()),
            other => Err(TigerReconstructError::UnsupportedHead(format!(
                "expected BinaryOp child at index {idx}, found {other:?}",
            ))),
        }
    }

    fn expect_unary_op(
        values: &[Option<BuiltValue>],
        idx: usize,
    ) -> Result<UnaryOp, TigerReconstructError> {
        match expect_value(values, idx)? {
            BuiltValue::UnaryOp(op) => Ok(op.clone()),
            other => Err(TigerReconstructError::UnsupportedHead(format!(
                "expected UnaryOp child at index {idx}, found {other:?}",
            ))),
        }
    }

    fn expect_ternary_op(
        values: &[Option<BuiltValue>],
        idx: usize,
    ) -> Result<TernaryOp, TigerReconstructError> {
        match expect_value(values, idx)? {
            BuiltValue::TernaryOp(op) => Ok(op.clone()),
            other => Err(TigerReconstructError::UnsupportedHead(format!(
                "expected TernaryOp child at index {idx}, found {other:?}",
            ))),
        }
    }

    fn expect_int(values: &[Option<BuiltValue>], idx: usize) -> Result<i64, TigerReconstructError> {
        match expect_value(values, idx)? {
            BuiltValue::Int(v) => Ok(*v),
            other => Err(TigerReconstructError::UnsupportedHead(format!(
                "expected i64 child at index {idx}, found {other:?}",
            ))),
        }
    }

    fn expect_bool(
        values: &[Option<BuiltValue>],
        idx: usize,
    ) -> Result<bool, TigerReconstructError> {
        match expect_value(values, idx)? {
            BuiltValue::Bool(v) => Ok(*v),
            other => Err(TigerReconstructError::UnsupportedHead(format!(
                "expected bool child at index {idx}, found {other:?}",
            ))),
        }
    }

    fn expect_float(
        values: &[Option<BuiltValue>],
        idx: usize,
    ) -> Result<OrderedFloat<f64>, TigerReconstructError> {
        match expect_value(values, idx)? {
            BuiltValue::Float(v) => Ok(*v),
            other => Err(TigerReconstructError::UnsupportedHead(format!(
                "expected f64 child at index {idx}, found {other:?}",
            ))),
        }
    }

    fn expect_string(
        values: &[Option<BuiltValue>],
        idx: usize,
    ) -> Result<String, TigerReconstructError> {
        match expect_value(values, idx)? {
            BuiltValue::String(s) => Ok(s.clone()),
            other => Err(TigerReconstructError::UnsupportedHead(format!(
                "expected String child at index {idx}, found {other:?}",
            ))),
        }
    }

    fn parse_binary_op(op: &str) -> Option<BinaryOp> {
        use BinaryOp::*;
        Some(match op {
            "Add" => Add,
            "Sub" => Sub,
            "Mul" => Mul,
            "Div" => Div,
            "Eq" => Eq,
            "LessThan" => LessThan,
            "GreaterThan" => GreaterThan,
            "LessEq" => LessEq,
            "GreaterEq" => GreaterEq,
            "Smax" => Smax,
            "Smin" => Smin,
            "Shl" => Shl,
            "Shr" => Shr,
            "FAdd" => FAdd,
            "FSub" => FSub,
            "FMul" => FMul,
            "FDiv" => FDiv,
            "FEq" => FEq,
            "FLessThan" => FLessThan,
            "FGreaterThan" => FGreaterThan,
            "FLessEq" => FLessEq,
            "FGreaterEq" => FGreaterEq,
            "Fmax" => Fmax,
            "Fmin" => Fmin,
            "And" => And,
            "Or" => Or,
            "PtrAdd" => PtrAdd,
            "Load" => Load,
            "Print" => Print,
            "Free" => Free,
            "Bitand" => Bitand,
            _ => return None,
        })
    }

    fn parse_unary_op(op: &str) -> Option<UnaryOp> {
        use UnaryOp::*;
        Some(match op {
            "Abs" => Abs,
            "Not" => Not,
            "Neg" => Neg,
            _ => return None,
        })
    }

    fn parse_ternary_op(op: &str) -> Option<TernaryOp> {
        use TernaryOp::*;
        Some(match op {
            "Write" => Write,
            "Select" => Select,
            _ => return None,
        })
    }

    fn parse_string_literal(raw: &str) -> String {
        if raw.starts_with('"') && raw.ends_with('"') && raw.len() >= 2 {
            let inner = &raw[1..raw.len() - 1];
            let mut result = String::with_capacity(inner.len());
            let mut chars = inner.chars();
            while let Some(ch) = chars.next() {
                if ch == '\\' {
                    if let Some(next) = chars.next() {
                        result.push(match next {
                            '\\' => '\\',
                            '"' => '"',
                            'n' => '\n',
                            't' => '\t',
                            other => other,
                        });
                    }
                } else {
                    result.push(ch);
                }
            }
            result
        } else {
            raw.to_string()
        }
    }

    fn usize_from_i64(v: i64, ctx: &str) -> Result<usize, TigerReconstructError> {
        if v < 0 {
            Err(TigerReconstructError::UnsupportedHead(format!(
                "expected non-negative index for {ctx}, found {v}"
            )))
        } else {
            Ok(v as usize)
        }
    }

    let mut values: Vec<Option<BuiltValue>> = vec![None; extraction.nodes.len()];
    for (idx, en) in extraction.nodes.iter().enumerate() {
        let tiger_idx = *tiger
            .class_index
            .get(&en.eclass)
            .expect("eclass missing in tiger graph");
        let ten = &tiger.eclasses[tiger_idx].enodes[en.enode_index];
        let op = ten.head.as_str();
        let sort = serialized
            .class_data
            .get(&en.eclass)
            .and_then(|d| d.typ.as_deref())
            .unwrap_or("Expr");

        let built_value = match sort {
            "String" => BuiltValue::String(parse_string_literal(op)),
            "i64" => {
                let value = op.parse::<i64>().map_err(|_| {
                    TigerReconstructError::UnsupportedHead(format!("invalid i64 literal '{op}'"))
                })?;
                BuiltValue::Int(value)
            }
            "bool" => {
                let value = match op {
                    "true" => true,
                    "false" => false,
                    other => {
                        return Err(TigerReconstructError::UnsupportedHead(format!(
                            "invalid bool literal '{other}'"
                        )))
                    }
                };
                BuiltValue::Bool(value)
            }
            "f64" => {
                let value = op.parse::<f64>().map_err(|_| {
                    TigerReconstructError::UnsupportedHead(format!("invalid f64 literal '{op}'"))
                })?;
                BuiltValue::Float(OrderedFloat(value))
            }
            "BinaryOp" => BuiltValue::BinaryOp(parse_binary_op(op).ok_or_else(|| {
                TigerReconstructError::UnsupportedHead(format!("unknown BinaryOp variant '{op}'"))
            })?),
            "UnaryOp" => BuiltValue::UnaryOp(parse_unary_op(op).ok_or_else(|| {
                TigerReconstructError::UnsupportedHead(format!("unknown UnaryOp variant '{op}'"))
            })?),
            "TernaryOp" => BuiltValue::TernaryOp(parse_ternary_op(op).ok_or_else(|| {
                TigerReconstructError::UnsupportedHead(format!("unknown TernaryOp variant '{op}'"))
            })?),
            "BaseType" => {
                use BaseType::*;
                let value = match op {
                    "IntT" => IntT,
                    "BoolT" => BoolT,
                    "FloatT" => FloatT,
                    "StateT" => StateT,
                    "PointerT" => {
                        if en.children.len() != 1 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "PointerT arity {} != 1",
                                en.children.len()
                            )));
                        }
                        PointerT(Box::new(expect_base_type(&values, en.children[0])?))
                    }
                    other => {
                        return Err(TigerReconstructError::UnsupportedHead(format!(
                            "unknown BaseType variant '{other}'"
                        )))
                    }
                };
                BuiltValue::BaseType(value)
            }
            "TypeList" => {
                let value = match op {
                    "TNil" => Vec::new(),
                    "TCons" => {
                        if en.children.len() != 2 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "TCons arity {} != 2",
                                en.children.len()
                            )));
                        }
                        let head = expect_base_type(&values, en.children[0])?;
                        let mut tail = expect_type_list(&values, en.children[1])?;
                        tail.insert(0, head);
                        tail
                    }
                    "TLConcat" => {
                        if en.children.len() != 2 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "TLConcat arity {} != 2",
                                en.children.len()
                            )));
                        }
                        let mut left = expect_type_list(&values, en.children[0])?;
                        let right = expect_type_list(&values, en.children[1])?;
                        left.extend(right);
                        left
                    }
                    other => {
                        return Err(TigerReconstructError::UnsupportedHead(format!(
                            "unknown TypeList variant '{other}'"
                        )))
                    }
                };
                BuiltValue::TypeList(value)
            }
            "Type" => {
                use SchemaType::*;
                let value = match op {
                    "Base" => {
                        if en.children.len() != 1 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "Base arity {} != 1",
                                en.children.len()
                            )));
                        }
                        Base(expect_base_type(&values, en.children[0])?)
                    }
                    "TupleT" => {
                        if en.children.len() != 1 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "TupleT arity {} != 1",
                                en.children.len()
                            )));
                        }
                        TupleT(expect_type_list(&values, en.children[0])?)
                    }
                    "TmpType" => Unknown,
                    other => {
                        return Err(TigerReconstructError::UnsupportedHead(format!(
                            "unknown Type variant '{other}'"
                        )))
                    }
                };
                BuiltValue::Type(value)
            }
            "Constant" => {
                use SchemaConstant::*;
                let value = match op {
                    "Int" => {
                        if en.children.len() != 1 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "Constant::Int arity {} != 1",
                                en.children.len()
                            )));
                        }
                        Int(expect_int(&values, en.children[0])?)
                    }
                    "Bool" => {
                        if en.children.len() != 1 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "Constant::Bool arity {} != 1",
                                en.children.len()
                            )));
                        }
                        Bool(expect_bool(&values, en.children[0])?)
                    }
                    "Float" => {
                        if en.children.len() != 1 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "Constant::Float arity {} != 1",
                                en.children.len()
                            )));
                        }
                        Float(expect_float(&values, en.children[0])?)
                    }
                    other => {
                        return Err(TigerReconstructError::UnsupportedHead(format!(
                            "unknown Constant variant '{other}'"
                        )))
                    }
                };
                BuiltValue::Constant(value)
            }
            "Assumption" => {
                use Assumption::*;
                let value = match op {
                    "InFunc" => {
                        if en.children.len() != 1 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "InFunc arity {} != 1",
                                en.children.len()
                            )));
                        }
                        InFunc(expect_string(&values, en.children[0])?)
                    }
                    "InLoop" => {
                        if en.children.len() != 2 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "InLoop arity {} != 2",
                                en.children.len()
                            )));
                        }
                        InLoop(
                            expect_expr(&values, en.children[0])?,
                            expect_expr(&values, en.children[1])?,
                        )
                    }
                    "InIf" => {
                        if en.children.len() != 3 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "InIf arity {} != 3",
                                en.children.len()
                            )));
                        }
                        InIf(
                            expect_bool(&values, en.children[0])?,
                            expect_expr(&values, en.children[1])?,
                            expect_expr(&values, en.children[2])?,
                        )
                    }
                    "InSwitch" => {
                        if en.children.len() != 3 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "InSwitch arity {} != 3",
                                en.children.len()
                            )));
                        }
                        InSwitch(
                            expect_int(&values, en.children[0])?,
                            expect_expr(&values, en.children[1])?,
                            expect_expr(&values, en.children[2])?,
                        )
                    }
                    "WildCard" => {
                        if en.children.len() != 1 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "WildCard arity {} != 1",
                                en.children.len()
                            )));
                        }
                        WildCard(expect_string(&values, en.children[0])?)
                    }
                    other => {
                        return Err(TigerReconstructError::UnsupportedHead(format!(
                            "unknown Assumption variant '{other}'"
                        )))
                    }
                };
                BuiltValue::Assumption(value)
            }
            "ListExpr" => {
                let value = match op {
                    "Nil" => Vec::new(),
                    "Cons" => {
                        if en.children.len() != 2 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "Cons arity {} != 2",
                                en.children.len()
                            )));
                        }
                        let head = expect_expr(&values, en.children[0])?;
                        let mut tail = expect_list_expr(&values, en.children[1])?;
                        tail.insert(0, head);
                        tail
                    }
                    other => {
                        return Err(TigerReconstructError::UnsupportedHead(format!(
                            "unknown ListExpr variant '{other}'"
                        )))
                    }
                };
                BuiltValue::ListExpr(value)
            }
            "Expr" => {
                let value = match op {
                    "Const" => {
                        if en.children.len() != 3 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "Const arity {} != 3",
                                en.children.len()
                            )));
                        }
                        Rc::new(Const(
                            expect_constant(&values, en.children[0])?,
                            expect_type(&values, en.children[1])?,
                            expect_assumption(&values, en.children[2])?,
                        ))
                    }
                    "Arg" => {
                        if en.children.len() != 2 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "Arg arity {} != 2",
                                en.children.len()
                            )));
                        }
                        Rc::new(Arg(
                            expect_type(&values, en.children[0])?,
                            expect_assumption(&values, en.children[1])?,
                        ))
                    }
                    "Empty" => {
                        if en.children.len() != 2 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "Empty arity {} != 2",
                                en.children.len()
                            )));
                        }
                        Rc::new(Empty(
                            expect_type(&values, en.children[0])?,
                            expect_assumption(&values, en.children[1])?,
                        ))
                    }
                    "Bop" => {
                        if en.children.len() != 3 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "Bop arity {} != 3",
                                en.children.len()
                            )));
                        }
                        Rc::new(Bop(
                            expect_binary_op(&values, en.children[0])?,
                            expect_expr(&values, en.children[1])?,
                            expect_expr(&values, en.children[2])?,
                        ))
                    }
                    "Uop" => {
                        if en.children.len() != 2 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "Uop arity {} != 2",
                                en.children.len()
                            )));
                        }
                        Rc::new(Uop(
                            expect_unary_op(&values, en.children[0])?,
                            expect_expr(&values, en.children[1])?,
                        ))
                    }
                    "Top" => {
                        if en.children.len() != 4 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "Top arity {} != 4",
                                en.children.len()
                            )));
                        }
                        Rc::new(Top(
                            expect_ternary_op(&values, en.children[0])?,
                            expect_expr(&values, en.children[1])?,
                            expect_expr(&values, en.children[2])?,
                            expect_expr(&values, en.children[3])?,
                        ))
                    }
                    "Get" => {
                        if en.children.len() != 2 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "Get arity {} != 2",
                                en.children.len()
                            )));
                        }
                        let idx_val = expect_int(&values, en.children[1])?;
                        Rc::new(Get(
                            expect_expr(&values, en.children[0])?,
                            usize_from_i64(idx_val, "Get")?,
                        ))
                    }
                    "Alloc" => {
                        if en.children.len() != 4 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "Alloc arity {} != 4",
                                en.children.len()
                            )));
                        }
                        Rc::new(Alloc(
                            expect_int(&values, en.children[0])?,
                            expect_expr(&values, en.children[1])?,
                            expect_expr(&values, en.children[2])?,
                            expect_base_type(&values, en.children[3])?,
                        ))
                    }
                    "Call" => {
                        if en.children.len() != 2 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "Call arity {} != 2",
                                en.children.len()
                            )));
                        }
                        Rc::new(Call(
                            expect_string(&values, en.children[0])?,
                            expect_expr(&values, en.children[1])?,
                        ))
                    }
                    "Single" => {
                        if en.children.len() != 1 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "Single arity {} != 1",
                                en.children.len()
                            )));
                        }
                        Rc::new(Single(expect_expr(&values, en.children[0])?))
                    }
                    "Concat" => {
                        if en.children.len() != 2 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "Concat arity {} != 2",
                                en.children.len()
                            )));
                        }
                        Rc::new(Concat(
                            expect_expr(&values, en.children[0])?,
                            expect_expr(&values, en.children[1])?,
                        ))
                    }
                    "If" => {
                        if en.children.len() != 4 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "If arity {} != 4",
                                en.children.len()
                            )));
                        }
                        Rc::new(If(
                            expect_expr(&values, en.children[0])?,
                            expect_expr(&values, en.children[1])?,
                            expect_expr(&values, en.children[2])?,
                            expect_expr(&values, en.children[3])?,
                        ))
                    }
                    "Switch" => {
                        if en.children.len() != 3 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "Switch arity {} != 3",
                                en.children.len()
                            )));
                        }
                        Rc::new(Switch(
                            expect_expr(&values, en.children[0])?,
                            expect_expr(&values, en.children[1])?,
                            expect_list_expr(&values, en.children[2])?,
                        ))
                    }
                    "DoWhile" => {
                        if en.children.len() != 2 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "DoWhile arity {} != 2",
                                en.children.len()
                            )));
                        }
                        Rc::new(DoWhile(
                            expect_expr(&values, en.children[0])?,
                            expect_expr(&values, en.children[1])?,
                        ))
                    }
                    "Function" => {
                        if en.children.len() != 4 {
                            return Err(TigerReconstructError::UnsupportedHead(format!(
                                "Function arity {} != 4",
                                en.children.len()
                            )));
                        }
                        Rc::new(Function(
                            expect_string(&values, en.children[0])?,
                            expect_type(&values, en.children[1])?,
                            expect_type(&values, en.children[2])?,
                            expect_expr(&values, en.children[3])?,
                        ))
                    }
                    "Symbolic" => {
                        if en.children.is_empty() {
                            return Err(TigerReconstructError::UnsupportedHead(
                                "Symbolic arity 0".into(),
                            ));
                        }
                        let name = expect_string(&values, en.children[0])?;
                        let ty = if en.children.len() > 1 {
                            Some(expect_type(&values, en.children[1])?)
                        } else {
                            None
                        };
                        Rc::new(Symbolic(name, ty))
                    }
                    other => {
                        return Err(TigerReconstructError::UnsupportedHead(format!(
                            "unknown Expr constructor '{other}'"
                        )))
                    }
                };
                BuiltValue::Expr(value)
            }
            other => {
                return Err(TigerReconstructError::UnsupportedHead(format!(
                    "unsupported sort '{other}' for op '{op}'"
                )))
            }
        };

        values[idx] = Some(built_value);
    }

    let ridx = extraction
        .root_index
        .ok_or_else(|| TigerReconstructError::MissingRoot("<unknown>".into()))?;
    match values.get(ridx).and_then(|v| v.as_ref()) {
        Some(BuiltValue::Expr(expr)) => Ok(expr.clone()),
        Some(other) => Err(TigerReconstructError::UnsupportedHead(format!(
            "root index did not resolve to Expr; found {other:?}"
        ))),
        None => Err(TigerReconstructError::ChildIndexRange),
    }
}

pub fn reconstruct_program_from_tiger(
    original_prog: &TreeProgram,
    serialized: &EGraph,
    tiger: &TigerEGraph,
    batch: &[String],
    tiger_res: &TigerExtractionResult,
    extractor: &TigerExtractor,
) -> Result<TreeProgram, TigerReconstructError> {
    use crate::schema::Expr;
    let mut new_funcs: IndexMap<String, RcExpr> = IndexMap::new();
    for f in &original_prog.functions {
        if let Expr::Function(name, _, _, _) = &**f {
            new_funcs.insert(name.clone(), f.clone());
        }
    }
    if let Expr::Function(name, _, _, _) = &*original_prog.entry {
        new_funcs.insert(name.clone(), original_prog.entry.clone());
    }
    for fname in batch {
        let Some(root_cid) = tiger_res.function_roots.get(fname) else {
            return Err(TigerReconstructError::MissingExtraction(fname.clone()));
        };
        let Some(tex) = tiger_res.extractions.get(root_cid) else {
            return Err(TigerReconstructError::MissingExtraction(fname.clone()));
        };

        // check that it's a valid extraction with a root index
        eprintln!(
            "[tiger reconstruct] Reconstructing function {fname} from extraction with {} nodes, root_cid = {root_cid}, root_index = {:?}",
            tex.nodes.len(),
            tex.root_index
        );
        for (idx, node) in tex.nodes.iter().enumerate() {
            let sort = serialized
                .class_data
                .get(&node.eclass)
                .and_then(|d| d.typ.as_deref())
                .unwrap_or("<unknown>");
            eprintln!(
                "  [extraction] idx={idx} eclass={} sort={} enode_index={} children={:?} original_node={:?}",
                node.eclass,
                sort,
                node.enode_index,
                node.children,
                node.original_node
            );
        }
        assert!(extractor.valid_extraction(tex, root_cid));
        let mut body = build_expr_from_extraction(serialized, tiger, tex)?;
        // If extraction gave us a full function, unwrap its body.
        if let Expr::Function(_, _in_ty, _out_ty, inner) = body.as_ref() {
            eprintln!("[tiger reconstruct] Unwrapping extracted Function node for {fname} and using its body");
            body = inner.clone();
        }
        if let Some(orig_fn) = new_funcs.get(fname).cloned() {
            if let Expr::Function(_, arg_ty, ret_ty, _) = &*orig_fn {
                let replaced = Rc::new(Expr::Function(
                    fname.clone(),
                    arg_ty.clone(),
                    ret_ty.clone(),
                    body,
                ));
                new_funcs.insert(fname.clone(), replaced);
            }
        }
    }
    let orig_entry_name = if let Expr::Function(n, _, _, _) = &*original_prog.entry {
        n.clone()
    } else {
        "main".to_string()
    };
    let mut others: Vec<RcExpr> = Vec::new();
    let mut entry: Option<RcExpr> = None;
    for (_name, fexpr) in new_funcs.into_iter() {
        if let Expr::Function(n, _, _, _) = &*fexpr {
            if *n == orig_entry_name {
                entry = Some(fexpr.clone());
            } else {
                others.push(fexpr.clone());
            }
        }
    }
    if entry.is_none() {
        entry = Some(original_prog.entry.clone());
    }
    let prog = TreeProgram {
        entry: entry.unwrap(),
        functions: others,
    };
    // Fill in argument types.
    Ok(prog.with_arg_types())
}
