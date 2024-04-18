#![allow(unused_imports)]
#![allow(dead_code)]

use crate::schema::{BaseType, Type};
use crate::schema_helpers::{Constructor, ESort, Purpose};
use crate::{egglog_test, prologue};
use strum::IntoEnumIterator;

struct AssociationListImpl {
    key_lessthan: String,
    val_intersect: String,
    val_union: String,
}

fn listlike(tys: Vec<&str>, alist_impl: Option<AssociationListImpl>) -> String {
    assert!(!tys.is_empty());
    let datatype = format!("L<{}>", tys.join("+"));
    // Wrap the list so we only compute indices for the "top level"
    // (otherwise eager indices are quadratic)
    let wrapper = format!("List<{}>", tys.join("+"));
    let unwrap = format!("{wrapper}-to-{datatype}");
    let wrap = format!("{datatype}-to-{wrapper}");
    let tys_s = tys.join(" ");
    let el = (0..tys.len())
        .map(|i| format!("hd{i}"))
        .collect::<Vec<_>>()
        .join(" ");
    let val1 = (1..tys.len())
        .map(|i| format!("a{i}"))
        .collect::<Vec<_>>()
        .join(" ");
    let val2 = (1..tys.len())
        .map(|i| format!("b{i}"))
        .collect::<Vec<_>>()
        .join(" ");
    let alist_code = match alist_impl {
        None => "".to_string(),
        Some(AssociationListImpl {
            key_lessthan,
            val_intersect,
            val_union,
        }) => {
            assert!(tys.len() <= 2);
            format!(
                "
(function Union-{datatype} ({datatype} {datatype}) {datatype})
  ; The third argument of the helper is a WIP result map.
  ; Invariant: keys of the result map are not present in the first two and are in descending order
  (function UnionHelper-{datatype} ({datatype} {datatype} {datatype}) {datatype})
  (rewrite (Union-{datatype} m1 m2)
           (Rev-{datatype} (UnionHelper-{datatype} m1 m2 (Nil-{datatype})))
           :ruleset always-run)

  ; both m1 and m2 empty
  (rewrite (UnionHelper-{datatype} (Nil-{datatype}) (Nil-{datatype}) res)
           res
           :ruleset always-run)
  ; take from m1 when m2 empty and vice versa
  (rewrite
    (UnionHelper-{datatype}
      (Nil-{datatype})
      (Cons-{datatype} {el} tl)
      res)
    (UnionHelper-{datatype}
      (Nil-{datatype})
      tl
      (Cons-{datatype} {el} res))
    :ruleset always-run)
  (rewrite
    (UnionHelper-{datatype}
      (Cons-{datatype} {el} tl)
      (Nil-{datatype})
      res)
    (UnionHelper-{datatype}
      tl
      (Nil-{datatype})
      (Cons-{datatype} {el} res))
    :ruleset always-run)

  ; when both nonempty and smallest key different, take smaller key
  (rule ((= f (UnionHelper-{datatype} l1 l2 res))
         (= l1 (Cons-{datatype} k1 {val1} tl1))
         (= l2 (Cons-{datatype} k2 {val2} tl2))
         ({key_lessthan} k1 k2))
        ((union f
           (UnionHelper-{datatype} tl1 l2 (Cons-{datatype} k1 {val1} res))))
        :ruleset always-run)
  (rule ((= f (UnionHelper-{datatype} l1 l2 res))
         (= l1 (Cons-{datatype} k1 {val1} tl1))
         (= l2 (Cons-{datatype} k2 {val2} tl2))
         ({key_lessthan} k2 k1))
        ((union f
           (UnionHelper-{datatype} l1 tl2 (Cons-{datatype} k2 {val2} res))))
        :ruleset always-run)

  ; when shared smallest key, union interval
  (rule ((= f (UnionHelper-{datatype} l1 l2 res))
         (= l1 (Cons-{datatype} k {val1} tl1))
         (= l2 (Cons-{datatype} k {val2} tl2)))
        ((union f
           (UnionHelper-{datatype} tl1 tl2
             (Cons-{datatype} k ({val_union} {val1} {val2}) res))))
        :ruleset always-run)

(function Union-{wrapper} ({wrapper} {wrapper}) {wrapper})
(rewrite (Union-{wrapper} x y)
         ({wrap} (Union-{datatype} ({unwrap} x) ({unwrap} y)))
         :ruleset always-run)


(function Intersect-{datatype} ({datatype} {datatype}) {datatype})
  ; The third argument of the helper is a WIP result map.
  ; Invariant: keys of the result map are not present in the first two and are in descending order
  (function IntersectHelper-{datatype} ({datatype} {datatype} {datatype}) {datatype})
  (rewrite (Intersect-{datatype} m1 m2)
           (Rev-{datatype} (IntersectHelper-{datatype} m1 m2 (Nil-{datatype})))
           :ruleset always-run)

  ; m1 or m2 empty
  (rewrite (IntersectHelper-{datatype} (Nil-{datatype}) m2 res)
           res
           :ruleset always-run)
  (rewrite (IntersectHelper-{datatype} m1 (Nil-{datatype}) res)
           res
           :ruleset always-run)

  ; when both nonempty and smallest key different, drop smaller key
  (rule ((= f (IntersectHelper-{datatype} l1 l2 res))
         (= l1 (Cons-{datatype} k1 {val1} tl1))
         (= l2 (Cons-{datatype} k2 {val2} tl2))
         ({key_lessthan} k1 k2))
        ((union f (IntersectHelper-{datatype} tl1 l2 res)))
        :ruleset always-run)
  (rule ((= f (IntersectHelper-{datatype} l1 l2 res))
         (= l1 (Cons-{datatype} k1 {val1} tl1))
         (= l2 (Cons-{datatype} k2 {val2} tl2))
         ({key_lessthan} k2 k1))
        ((union f (IntersectHelper-{datatype} tl1 l2 res)))
        :ruleset always-run)

  ; when shared smallest key, intersect interval
  (rule ((= f (IntersectHelper-{datatype} l1 l2 res))
         (= l1 (Cons-{datatype} k {val1} tl1))
         (= l2 (Cons-{datatype} k {val2} tl2)))
        ((union f
           (IntersectHelper-{datatype} tl1 tl2
             (Cons-{datatype} k ({val_intersect} {val1} {val2}) res))))
        :ruleset always-run)

(function Intersect-{wrapper} ({wrapper} {wrapper}) {wrapper})
(rewrite (Intersect-{wrapper} x y)
         ({wrap} (Intersect-{datatype} ({unwrap} x) ({unwrap} y)))
         :ruleset always-run)
        "
            )
        }
    };
    format!(
        "
(datatype {datatype} 
  (Nil-{datatype})
  (Cons-{datatype} {tys_s} {datatype}))

(relation IsEmpty-{datatype} ({datatype}))
(rule ((= x (Nil-{datatype})))
      ((IsEmpty-{datatype} x))
      :ruleset always-run)

(relation IsNonEmpty-{datatype} ({datatype}))
(rule ((= x (Cons-{datatype} {el} tl)))
      ((IsNonEmpty-{datatype} x))
      :ruleset always-run)

(function RevConcat-{datatype} ({datatype} {datatype}) {datatype})
(rewrite (RevConcat-{datatype} (Nil-{datatype}) l)
         l
         :ruleset always-run)
(rewrite (RevConcat-{datatype} (Cons-{datatype} {el} tl) l)
         (RevConcat-{datatype} tl (Cons-{datatype} {el} l))
         :ruleset always-run)

(function Rev-{datatype} ({datatype}) {datatype})
(rewrite (Rev-{datatype} m)
         (RevConcat-{datatype} m (Nil-{datatype}))
         :ruleset always-run)

(function Concat-{datatype} ({datatype} {datatype}) {datatype})
(rewrite (Concat-{datatype} x y)
         (RevConcat-{datatype} (Rev-{datatype} x) y)
         :ruleset always-run)

(datatype {wrapper}
  ({wrap} {datatype}))
(function {unwrap} ({wrapper}) {datatype})
(rewrite ({unwrap} ({wrap} x))
         x
         :ruleset always-run)

(function Nil-{wrapper} () {wrapper})
(rewrite (Nil-{wrapper})
         ({wrap} (Nil-{datatype}))
         :ruleset always-run)

(function Cons-{wrapper} ({tys_s} {wrapper}) {wrapper})
(rewrite (Cons-{wrapper} {el} tl)
         ({wrap} (Cons-{datatype} {el} ({unwrap} tl)))
         :ruleset always-run)

(function Rev-{wrapper} ({wrapper}) {wrapper})
(rewrite (Rev-{wrapper} x)
         ({wrap} (Rev-{datatype} ({unwrap} x)))
         :ruleset always-run)

(function Concat-{wrapper} ({wrapper} {wrapper}) {wrapper})
(rewrite (Concat-{wrapper} x y)
         ({wrap} (Concat-{datatype} ({unwrap} x) ({unwrap} y)))
         :ruleset always-run)

(relation IsEmpty-{wrapper} ({wrapper}))
(rule ((IsEmpty-{datatype} x))
      ((IsEmpty-{wrapper} ({wrap} x)))
      :ruleset always-run)

(relation IsNonEmpty-{wrapper} ({wrapper}))
(rule ((IsNonEmpty-{datatype} x))
      ((IsNonEmpty-{wrapper} ({wrap} x)))
      :ruleset always-run)

; Only define SuffixAt and At for {wrapper} so it's O(N)
(relation SuffixAt-{wrapper} ({wrapper} i64 {datatype}))
(relation At-{wrapper} ({wrapper} i64 {tys_s}))
(rule ((= wrapped ({wrap} x)))
      ((SuffixAt-{wrapper} wrapped 0 x))
      :ruleset always-run)
(rule ((SuffixAt-{wrapper} wrapper i (Cons-{datatype} {el} tl)))
      ((SuffixAt-{wrapper} wrapper (+ i 1) tl)
       (At-{wrapper} wrapper i {el}))
      :ruleset always-run)

{alist_code}
    "
    )
}

/*
(interval-map-union map1 map2)
*/

#[allow(non_snake_case)]
pub(crate) fn rules() -> String {
    let Listi64IntInterval = listlike(
        vec!["i64", "IntInterval"],
        Some(AssociationListImpl {
            key_lessthan: "<".to_string(),
            val_intersect: "IntersectIntInterval".to_string(),
            val_union: "UnionIntInterval".to_string(),
        }),
    );
    let ListListi64IntInterval = listlike(vec!["List<i64+IntInterval>"], None);
    format!(
        "
(datatype IntOrInfinity
    (Infinity)
    (NegInfinity)
    (I i64))

(function MaxIntOrInfinity (IntOrInfinity IntOrInfinity) IntOrInfinity)
(rewrite (MaxIntOrInfinity (Infinity) _) (Infinity) :ruleset always-run)
(rewrite (MaxIntOrInfinity _ (Infinity)) (Infinity) :ruleset always-run)
(rewrite (MaxIntOrInfinity (NegInfinity) x) x :ruleset always-run)
(rewrite (MaxIntOrInfinity x (NegInfinity)) x :ruleset always-run)
(rewrite (MaxIntOrInfinity (I x) (I y)) (I (max x y)) :ruleset always-run)

(function MinIntOrInfinity (IntOrInfinity IntOrInfinity) IntOrInfinity)
(rewrite (MinIntOrInfinity (NegInfinity) _) (NegInfinity) :ruleset always-run)
(rewrite (MinIntOrInfinity _ (NegInfinity)) (NegInfinity) :ruleset always-run)
(rewrite (MinIntOrInfinity (Infinity) x) x :ruleset always-run)
(rewrite (MinIntOrInfinity x (Infinity)) x :ruleset always-run)
(rewrite (MinIntOrInfinity (I x) (I y)) (I (min x y)) :ruleset always-run)

(datatype IntInterval (MkIntInterval IntOrInfinity IntOrInfinity))

(function UnionIntInterval (IntInterval IntInterval) IntInterval)
(rewrite (UnionIntInterval (MkIntInterval lo1 hi1) (MkIntInterval lo2 hi2))
         (MkIntInterval (MinIntOrInfinity lo1 lo2) (MaxIntOrInfinity hi1 hi2))
         :ruleset always-run)

(function IntersectIntInterval (IntInterval IntInterval) IntInterval)
(rewrite (IntersectIntInterval (MkIntInterval lo1 hi1) (MkIntInterval lo2 hi2))
         (MkIntInterval (MaxIntOrInfinity lo1 lo2) (MinIntOrInfinity hi1 hi2))
         :ruleset always-run)

{Listi64IntInterval}
{ListListi64IntInterval}"
    )
}

#[cfg(test)]
use crate::schema::Constant;
#[cfg(test)]
use crate::Value;

#[test]
fn load_after_write() -> crate::Result {
    use crate::ast::*;
    // ptr = alloc int 1;
    // write ptr 2;
    // res = load ptr;
    // print res
    // =>
    // <some effects, but no load>
    // print 2;
    let one = int_ty(1, Type::Base(BaseType::IntT));
    let two = int_ty(2, Type::Base(BaseType::IntT));
    let orig_state = get(arg_ty(tuplet!(statet())), 0);
    let ptr_and_state = alloc(0, one, orig_state.clone(), pointert(intt()));
    let ptr = get(ptr_and_state.clone(), 0);
    let state = get(ptr_and_state, 1);
    let state = write(ptr.clone(), two.clone(), state);
    let val_and_state = load(ptr, state);
    let val = get(val_and_state.clone(), 0);
    let state = get(val_and_state, 1);
    let res = tprint(val, state);

    egglog_test(
        &format!("{res}"),
        &format!("(check (= {res} (Bop (Print) (Const (Int 2) (Base (IntT))) rest)))"),
        vec![],
        val_empty(),
        val_empty(),
        vec![],
    )
}

#[test]
fn load_after_write_without_alias() -> crate::Result {
    use crate::ast::*;
    // ptr1 = alloc int 1;
    // ptr2 = alloc int 1;
    // write ptr1 2;
    // write ptr2 3;
    // res = load ptr1;
    // print res
    // =>
    // <some effects, but no load>
    // print 2;
    //
    // This relies on the alias analysis to work.
    let one = int_ty(1, Type::Base(BaseType::IntT));
    let two = int_ty(2, Type::Base(BaseType::IntT));
    let three = int_ty(3, Type::Base(BaseType::IntT));
    let orig_state = get(arg_ty(tuplet!(statet())), 0);
    let ptr_and_state = alloc(0, one.clone(), orig_state.clone(), pointert(intt()));
    let ptr1 = get(ptr_and_state.clone(), 0);
    let state = get(ptr_and_state, 1);
    let ptr_and_state = alloc(1, one, state, pointert(intt()));
    let ptr2 = get(ptr_and_state.clone(), 0);
    let state = get(ptr_and_state, 1);
    let state = write(ptr1.clone(), two.clone(), state);
    let state = write(ptr2.clone(), three, state);
    let val_and_state = load(ptr1, state);
    let val = get(val_and_state.clone(), 0);
    let state = get(val_and_state, 1);
    let res = tprint(val, state);
    egglog_test(
        &format!("(DemandPointsToCells {res})"),
        &format!("(check (= {res} (Bop (Print) (Const (Int 2) (Base (IntT))) rest)))"),
        vec![],
        val_empty(),
        val_empty(),
        vec![],
    )
}

// #[test]
fn pqrs_loop_swap() -> crate::Result {
    use crate::ast::*;
    let concat_par3 = |x, y, z| concat_par(x, concat_par(y, z));
    let alloc_id = 1;
    let state = get(arg_ty(tuplet!(statet())), 0);
    let p_and_state = alloc(alloc_id, int(4), state, pointert(intt()));
    let p = get(p_and_state.clone(), 0);
    let state = get(p_and_state, 1);
    let loop1 = dowhile(
        parallel!(
            state,                     // state
            p.clone(),                 // p
            ptradd(p.clone(), int(1)), // q
            ptradd(p.clone(), int(2)), // r
            ptradd(p.clone(), int(3)), // s
        ),
        concat_par3(
            // single(call("f", getat(0))), // pred
            single(ttrue()), // pred
            dowhile(
                parallel!(
                    getat(0), // state
                    getat(1), // p
                    getat(2), // q
                ),
                parallel!(
                    // call("g", getat(0)), // pred
                    ttrue(),  // pred
                    getat(0), // state
                    getat(2), // q
                    getat(1), // p
                ),
            ),
            parallel!(
                getat(4), // s
                getat(3)  // r
            ),
        ),
    );
    let state = get(loop1.clone(), 0);
    let p = get(loop1.clone(), 1);
    let r = get(loop1.clone(), 3);
    let state = write(p.clone(), int(10), state);
    let state = write(r.clone(), int(20), state);
    let val_and_state = load(p, state);
    let val = get(val_and_state.clone(), 0);
    let state = get(val_and_state, 1);
    let res = tprint(val, state).with_arg_types(tuplet!(statet()), Type::Base(statet()));
    egglog_test(
        &format!("{res}"),
        &format!("(check (= {res} (Bop (Print) (Const (Int 10) (Base (IntT))) rest)))"),
        vec![],
        val_empty(),
        val_empty(),
        vec![],
    )
}
