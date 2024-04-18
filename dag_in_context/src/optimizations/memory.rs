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

struct FnImpl {
    name: String,
    arg_tys: Vec<String>,
    resultty: String,
}

fn listlike(
    el_tys: Vec<&str>,
    el_functions: Vec<FnImpl>,
    el_relations: Vec<&str>,
    el_binops: Vec<FnImpl>,
    alist_impl: Option<AssociationListImpl>,
) -> String {
    assert!(!el_tys.is_empty());
    for f in el_functions {
        assert!(f.arg_tys == el_tys);
    }
    let datatype = format!("List<{}>", el_tys.join("+"));
    let tys_s = el_tys.join(" ");
    let el = (0..el_tys.len())
        .map(|i| format!("hd{i}"))
        .collect::<Vec<_>>()
        .join(" ");

    let mut res = vec![];

    res.push(format!(
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

; SuffixAt and At must be demanded, otherwise these are O(N^2)
(relation DemandAt-{datatype} ({datatype}))
(relation SuffixAt-{datatype} ({datatype} i64 {datatype}))
(relation At-{datatype} ({datatype} i64 {tys_s}))
(rule ((DemandAt-{datatype} x))
      ((SuffixAt-{datatype} x 0 x))
      :ruleset always-run)
(rule ((SuffixAt-{datatype} x i (Cons-{datatype} {el} tl)))
      ((SuffixAt-{datatype} x (+ i 1) tl)
       (At-{datatype} x i {el}))
      :ruleset always-run)"
    ));

    for el_relation in el_relations {
        res.push(format!(
            "
(relation All<{el_relation}> ({datatype}))
(rule ((= x (Nil-{datatype})))
      ((All<{el_relation}> x))
      :ruleset always-run)
(rule ((= x (Cons-{datatype} {el} tl))
       ({el_relation} {el})
       (All<{el_relation}> tl))
      ((All<{el_relation}> x))
      :ruleset always-run)
        "
        ));
    }

    for FnImpl {
        name,
        arg_tys,
        resultty,
    } in el_binops
    {
        let mut expected_tys = el_tys.clone();
        expected_tys.extend(el_tys.iter());
        assert!(arg_tys == expected_tys);
        let args1 = (0..el_tys.len())
            .map(|i| format!("x{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let args2 = (0..el_tys.len())
            .map(|i| format!("y{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let zipresultty = format!("List<{resultty}>");
        res.push(format!(
            "
(function Zip<{name}> ({datatype} {datatype}) {zipresultty})
(rewrite (Zip<{name}> (Nil-{datatype}) (Nil-{datatype}))
         (Nil-{zipresultty})
         :ruleset always-run)
(rewrite (Zip<{name}>
           (Cons-{datatype} {args1} tl1)
           (Cons-{datatype} {args2} tl2))
         (Cons-{zipresultty}
            ({name} {args1} {args2})
            (Zip<{name}> tl1 tl2))
         :ruleset always-run)
            "
        ))
    }

    if let Some(AssociationListImpl {
        key_lessthan,
        val_intersect,
        val_union,
    }) = alist_impl
    {
        assert!(el_tys.len() <= 2);
        let val1 = (1..el_tys.len())
            .map(|i| format!("a{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        let val2 = (1..el_tys.len())
            .map(|i| format!("b{i}"))
            .collect::<Vec<_>>()
            .join(" ");
        res.push(format!(
            "
; (function Union-{datatype} ({datatype} {datatype}) {datatype})
  ; ; The third argument of the helper is a WIP result map.
  ; ; Invariant: keys of the result map are not present in the first two and are in descending order
  ; (function UnionHelper-{datatype} ({datatype} {datatype} {datatype}) {datatype})
  ; (rewrite (Union-{datatype} m1 m2)
           ; (Rev-{datatype} (UnionHelper-{datatype} m1 m2 (Nil-{datatype})))
           ; :ruleset always-run)

  ; ; both m1 and m2 empty
  ; (rewrite (UnionHelper-{datatype} (Nil-{datatype}) (Nil-{datatype}) res)
           ; res
           ; :ruleset always-run)
  ; ; take from m1 when m2 empty and vice versa
  ; (rewrite
    ; (UnionHelper-{datatype}
      ; (Nil-{datatype})
      ; (Cons-{datatype} {el} tl)
      ; res)
    ; (UnionHelper-{datatype}
      ; (Nil-{datatype})
      ; tl
      ; (Cons-{datatype} {el} res))
    ; :ruleset always-run)
  ; (rewrite
    ; (UnionHelper-{datatype}
      ; (Cons-{datatype} {el} tl)
      ; (Nil-{datatype})
      ; res)
    ; (UnionHelper-{datatype}
      ; tl
      ; (Nil-{datatype})
      ; (Cons-{datatype} {el} res))
    ; :ruleset always-run)

  ; ; when both nonempty and smallest key different, take smaller key
  ; (rule ((= f (UnionHelper-{datatype} l1 l2 res))
         ; (= l1 (Cons-{datatype} k1 {val1} tl1))
         ; (= l2 (Cons-{datatype} k2 {val2} tl2))
         ; ({key_lessthan} k1 k2))
        ; ((union f
           ; (UnionHelper-{datatype} tl1 l2 (Cons-{datatype} k1 {val1} res))))
        ; :ruleset always-run)
  ; (rule ((= f (UnionHelper-{datatype} l1 l2 res))
         ; (= l1 (Cons-{datatype} k1 {val1} tl1))
         ; (= l2 (Cons-{datatype} k2 {val2} tl2))
         ; ({key_lessthan} k2 k1))
        ; ((union f
           ; (UnionHelper-{datatype} l1 tl2 (Cons-{datatype} k2 {val2} res))))
        ; :ruleset always-run)

  ; ; when shared smallest key, union interval
  ; (rule ((= f (UnionHelper-{datatype} l1 l2 res))
         ; (= l1 (Cons-{datatype} k {val1} tl1))
         ; (= l2 (Cons-{datatype} k {val2} tl2)))
        ; ((union f
           ; (UnionHelper-{datatype} tl1 tl2
             ; (Cons-{datatype} k ({val_union} {val1} {val2}) res))))
        ; :ruleset always-run)

; (function Intersect-{datatype} ({datatype} {datatype}) {datatype})
  ; ; The third argument of the helper is a WIP result map.
  ; ; Invariant: keys of the result map are not present in the first two and are in descending order
  ; (function IntersectHelper-{datatype} ({datatype} {datatype} {datatype}) {datatype})
  ; (rewrite (Intersect-{datatype} m1 m2)
           ; (Rev-{datatype} (IntersectHelper-{datatype} m1 m2 (Nil-{datatype})))
           ; :ruleset always-run)

  ; ; m1 or m2 empty
  ; (rewrite (IntersectHelper-{datatype} (Nil-{datatype}) m2 res)
           ; res
           ; :ruleset always-run)
  ; (rewrite (IntersectHelper-{datatype} m1 (Nil-{datatype}) res)
           ; res
           ; :ruleset always-run)

  ; ; when both nonempty and smallest key different, drop smaller key
  ; (rule ((= f (IntersectHelper-{datatype} l1 l2 res))
         ; (= l1 (Cons-{datatype} k1 {val1} tl1))
         ; (= l2 (Cons-{datatype} k2 {val2} tl2))
         ; ({key_lessthan} k1 k2))
        ; ((union f (IntersectHelper-{datatype} tl1 l2 res)))
        ; :ruleset always-run)
  ; (rule ((= f (IntersectHelper-{datatype} l1 l2 res))
         ; (= l1 (Cons-{datatype} k1 {val1} tl1))
         ; (= l2 (Cons-{datatype} k2 {val2} tl2))
         ; ({key_lessthan} k2 k1))
        ; ((union f (IntersectHelper-{datatype} tl1 l2 res)))
        ; :ruleset always-run)

  ; ; when shared smallest key, intersect interval
  ; (rule ((= f (IntersectHelper-{datatype} l1 l2 res))
         ; (= l1 (Cons-{datatype} k {val1} tl1))
         ; (= l2 (Cons-{datatype} k {val2} tl2)))
        ; ((union f
           ; (IntersectHelper-{datatype} tl1 tl2
             ; (Cons-{datatype} k ({val_intersect} {val1} {val2}) res))))
        ; :ruleset always-run)"
        ));
    }
    res.join("\n")
}

#[allow(non_snake_case)]
pub(crate) fn rules() -> String {
    let Listi64IntInterval = listlike(
        vec!["i64", "IntInterval"],
        vec![],
        vec![],
        vec![],
        Some(AssociationListImpl {
            key_lessthan: "<".to_string(),
            val_intersect: "IntersectIntInterval".to_string(),
            val_union: "UnionIntInterval".to_string(),
        }),
    );
    let ListListi64IntInterval = listlike(
        vec!["List<i64+IntInterval>"],
        vec![],
        vec!["IsEmpty-List<i64+IntInterval>"],
        vec![
            FnImpl {
                name: "Union-List<i64+IntInterval>".to_string(),
                arg_tys: vec![
                    "List<i64+IntInterval>".to_string(),
                    "List<i64+IntInterval>".to_string(),
                ],
                resultty: "List<i64+IntInterval>".to_string(),
            },
            FnImpl {
                name: "Intersect-List<i64+IntInterval>".to_string(),
                arg_tys: vec![
                    "List<i64+IntInterval>".to_string(),
                    "List<i64+IntInterval>".to_string(),
                ],
                resultty: "List<i64+IntInterval>".to_string(),
            },
        ],
        None,
    );
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

(function AddIntOrInfinity (IntOrInfinity IntOrInfinity) IntOrInfinity)
(rewrite (AddIntOrInfinity (Infinity) (Infinity)) (Infinity) :ruleset always-run)
(rewrite (AddIntOrInfinity (Infinity) (I _)) (Infinity) :ruleset always-run)
(rewrite (AddIntOrInfinity (I _) (Infinity)) (Infinity) :ruleset always-run)
(rewrite (AddIntOrInfinity (NegInfinity) (NegInfinity)) (NegInfinity) :ruleset always-run)
(rewrite (AddIntOrInfinity (NegInfinity) (I _)) (NegInfinity) :ruleset always-run)
(rewrite (AddIntOrInfinity (I _) (NegInfinity)) (NegInfinity) :ruleset always-run)
(rewrite (AddIntOrInfinity (I x) (I y)) (I (+ x y)) :ruleset always-run)

(datatype IntInterval (MkIntInterval IntOrInfinity IntOrInfinity))

(function UnionIntInterval (IntInterval IntInterval) IntInterval)
(rewrite (UnionIntInterval (MkIntInterval lo1 hi1) (MkIntInterval lo2 hi2))
         (MkIntInterval (MinIntOrInfinity lo1 lo2) (MaxIntOrInfinity hi1 hi2))
         :ruleset always-run)

(function IntersectIntInterval (IntInterval IntInterval) IntInterval)
(rewrite (IntersectIntInterval (MkIntInterval lo1 hi1) (MkIntInterval lo2 hi2))
         (MkIntInterval (MaxIntOrInfinity lo1 lo2) (MinIntOrInfinity hi1 hi2))
         :ruleset always-run)

(function AddIntInterval (IntInterval IntInterval) IntInterval)
(rewrite (AddIntInterval (MkIntInterval lo1 hi1) (MkIntInterval lo2 hi2))
         (MkIntInterval (AddIntOrInfinity lo1 lo2)
                        (AddIntOrInfinity hi1 hi2))
         :ruleset always-run)

{Listi64IntInterval}

(function Union-List<i64+IntInterval> (List<i64+IntInterval> List<i64+IntInterval>) List<i64+IntInterval>)
  ; The third argument of the helper is a WIP result map.
  ; Invariant: keys of the result map are not present in the first two and are in descending order
  (function UnionHelper-List<i64+IntInterval> (List<i64+IntInterval> List<i64+IntInterval> List<i64+IntInterval>) List<i64+IntInterval>)
  (rewrite (Union-List<i64+IntInterval> m1 m2)
           (Rev-List<i64+IntInterval> (UnionHelper-List<i64+IntInterval> m1 m2 (Nil-List<i64+IntInterval>)))
           :ruleset always-run)

  ; both m1 and m2 empty
  (rewrite (UnionHelper-List<i64+IntInterval> (Nil-List<i64+IntInterval>) (Nil-List<i64+IntInterval>) res)
           res
           :ruleset always-run)
  ; take from m1 when m2 empty and vice versa
  (rewrite
    (UnionHelper-List<i64+IntInterval>
      (Nil-List<i64+IntInterval>)
      (Cons-List<i64+IntInterval> hd0 hd1 tl)
      res)
    (UnionHelper-List<i64+IntInterval>
      (Nil-List<i64+IntInterval>)
      tl
      (Cons-List<i64+IntInterval> hd0 hd1 res))
    :ruleset always-run)
  (rewrite
    (UnionHelper-List<i64+IntInterval>
      (Cons-List<i64+IntInterval> hd0 hd1 tl)
      (Nil-List<i64+IntInterval>)
      res)
    (UnionHelper-List<i64+IntInterval>
      tl
      (Nil-List<i64+IntInterval>)
      (Cons-List<i64+IntInterval> hd0 hd1 res))
    :ruleset always-run)

  ; when both nonempty and smallest key different, take smaller key
  (rule ((= f (UnionHelper-List<i64+IntInterval> l1 l2 res))
         (= l1 (Cons-List<i64+IntInterval> k1 a1 tl1))
         (= l2 (Cons-List<i64+IntInterval> k2 b1 tl2))
         (< k1 k2))
        ((union f
           (UnionHelper-List<i64+IntInterval> tl1 l2 (Cons-List<i64+IntInterval> k1 a1 res))))
        :ruleset always-run)
  (rule ((= f (UnionHelper-List<i64+IntInterval> l1 l2 res))
         (= l1 (Cons-List<i64+IntInterval> k1 a1 tl1))
         (= l2 (Cons-List<i64+IntInterval> k2 b1 tl2))
         (< k2 k1))
        ((union f
           (UnionHelper-List<i64+IntInterval> l1 tl2 (Cons-List<i64+IntInterval> k2 b1 res))))
        :ruleset always-run)

  ; when shared smallest key, union interval
  (rule ((= f (UnionHelper-List<i64+IntInterval> l1 l2 res))
         (= l1 (Cons-List<i64+IntInterval> k a1 tl1))
         (= l2 (Cons-List<i64+IntInterval> k b1 tl2)))
        ((union f
           (UnionHelper-List<i64+IntInterval> tl1 tl2
             (Cons-List<i64+IntInterval> k (UnionIntInterval a1 b1) res))))
        :ruleset always-run)

(function Intersect-List<i64+IntInterval> (List<i64+IntInterval> List<i64+IntInterval>) List<i64+IntInterval>)
  ; The third argument of the helper is a WIP result map.
  ; Invariant: keys of the result map are not present in the first two and are in descending order
  (function IntersectHelper-List<i64+IntInterval> (List<i64+IntInterval> List<i64+IntInterval> List<i64+IntInterval>) List<i64+IntInterval>)
  (rewrite (Intersect-List<i64+IntInterval> m1 m2)
           (Rev-List<i64+IntInterval> (IntersectHelper-List<i64+IntInterval> m1 m2 (Nil-List<i64+IntInterval>)))
           :ruleset always-run)

  ; m1 or m2 empty
  (rewrite (IntersectHelper-List<i64+IntInterval> (Nil-List<i64+IntInterval>) m2 res)
           res
           :ruleset always-run)
  (rewrite (IntersectHelper-List<i64+IntInterval> m1 (Nil-List<i64+IntInterval>) res)
           res
           :ruleset always-run)

  ; when both nonempty and smallest key different, drop smaller key
  (rule ((= f (IntersectHelper-List<i64+IntInterval> l1 l2 res))
         (= l1 (Cons-List<i64+IntInterval> k1 a1 tl1))
         (= l2 (Cons-List<i64+IntInterval> k2 b1 tl2))
         (< k1 k2))
        ((union f (IntersectHelper-List<i64+IntInterval> tl1 l2 res)))
        :ruleset always-run)
  (rule ((= f (IntersectHelper-List<i64+IntInterval> l1 l2 res))
         (= l1 (Cons-List<i64+IntInterval> k1 a1 tl1))
         (= l2 (Cons-List<i64+IntInterval> k2 b1 tl2))
         (< k2 k1))
        ((union f (IntersectHelper-List<i64+IntInterval> tl1 l2 res)))
        :ruleset always-run)

(datatype MyBool (MyTrue) (MyFalse))

(function IntIntervalValid (IntInterval) MyBool)
(rewrite (IntIntervalValid (MkIntInterval (I lo) (I hi)))
         (MyTrue)
         :when ((<= lo hi))
         :ruleset always-run)
(rewrite (IntIntervalValid (MkIntInterval (I lo) (I hi)))
         (MyFalse)
         :when ((> lo hi))
         :ruleset always-run)
(rewrite (IntIntervalValid (MkIntInterval (NegInfinity) _))
         (MyTrue)
         :ruleset always-run)
(rewrite (IntIntervalValid (MkIntInterval _ (Infinity)))
         (MyTrue)
         :ruleset always-run)

(function ConsIfNonEmpty (i64 IntInterval List<i64+IntInterval>)
          List<i64+IntInterval>)
(rule ((ConsIfNonEmpty k v tl))
      ((IntIntervalValid v))
      :ruleset always-run)
(rule ((= f (ConsIfNonEmpty k v tl))
       (= (MyTrue) (IntIntervalValid v)))
      ((union f (Cons-List<i64+IntInterval> k v tl)))
      :ruleset always-run)
(rule ((= f (ConsIfNonEmpty k v tl))
       (= (MyFalse) (IntIntervalValid v)))
      ((union f tl))
      :ruleset always-run)

  ; when shared smallest key, intersect interval
  (rule ((= f (IntersectHelper-List<i64+IntInterval> l1 l2 res))
         (= l1 (Cons-List<i64+IntInterval> k a1 tl1))
         (= l2 (Cons-List<i64+IntInterval> k b1 tl2)))
        ((union f
           (IntersectHelper-List<i64+IntInterval> tl1 tl2
             (ConsIfNonEmpty k (IntersectIntInterval a1 b1) res))))
        :ruleset always-run)

(function AddIntIntervalToAll (IntInterval List<i64+IntInterval>)
                              List<i64+IntInterval>)
(rewrite (AddIntIntervalToAll _ (Nil-List<i64+IntInterval>))
         (Nil-List<i64+IntInterval>)
         :ruleset always-run)
(rewrite (AddIntIntervalToAll x (Cons-List<i64+IntInterval> allocid offset tl))
         (Cons-List<i64+IntInterval> allocid (AddIntInterval x offset)
           (AddIntIntervalToAll x tl))
         :ruleset always-run)

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

#[test]
fn simple_loop_swap() -> crate::Result {
    use crate::ast::*;
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
        ),
        parallel!(
            ttrue(),  // pred
            getat(0), // state
            getat(2), // q
            getat(1), // p
            getat(3), // r
        ),
    )
    .with_arg_types(
        tuplet!(statet()),
        tuplet!(
            statet(),
            pointert(intt()),
            pointert(intt()),
            pointert(intt())
        ),
    );
    let state = get(loop1.clone(), 0);
    let p = get(loop1.clone(), 1);
    let r = get(loop1.clone(), 3);
    let ten = int(10).with_arg_types(tuplet!(statet()), Type::Base(intt()));
    // let twenty = int(20).with_arg_types(tuplet!(statet()), Type::Base(intt()));
    let state = write(p.clone(), int(10), state);
    // let state_after_pwrite = state
    //     .clone()
    //     .with_arg_types(tuplet!(statet()), Type::Base(statet()));
    let state = write(r.clone(), int(20), state);
    let state_after_rwrite = state
        .clone()
        .with_arg_types(tuplet!(statet()), Type::Base(statet()));
    let val_and_state = load(p.clone(), state);
    let val = get(val_and_state.clone(), 0).with_arg_types(tuplet!(statet()), Type::Base(intt()));
    // let state = get(val_and_state, 1);
    // let res = tprint(val, state).with_arg_types(tuplet!(statet()), Type::Base(statet()));
    egglog_test(
        &format!("(DemandPointsToCells {val})"),
        &format!(
            "
; (extract (PointsToCells {loop1} (PointsAnywhere)))
; (extract (PointsToCells {p} (PointsAnywhere)))
; (extract (PointsToCells {r} (PointsAnywhere)))
; (extract (IntersectPointees (PointsToCells {p} (PointsAnywhere))
                            ; (PointsToCells {r} (PointsAnywhere))))
(check (PointsNowhere
         (IntersectPointees (PointsToCells {p} (PointsAnywhere))
                            (PointsToCells {r} (PointsAnywhere)))))

(check (DontAlias {p} {r} (PointsAnywhere)))
(check (= (PointsTo {state_after_rwrite} {p}) {ten}))

(check (= {val} {ten}))
"
        ),
        vec![],
        val_empty(),
        val_empty(),
        vec![],
    )
}

// #[test]
fn pqrs_deep_loop_swap() -> crate::Result {
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
    )
    .with_arg_types(
        tuplet!(statet()),
        tuplet!(
            statet(),
            pointert(intt()),
            pointert(intt()),
            pointert(intt()),
            pointert(intt())
        ),
    );
    let state = get(loop1.clone(), 0);
    let p = get(loop1.clone(), 1);
    let r = get(loop1.clone(), 3);
    let state = write(p.clone(), int(10), state);
    let state = write(r.clone(), int(20), state);
    let state_after_rwrite = state
        .clone()
        .with_arg_types(tuplet!(statet()), Type::Base(statet()));
    let val_and_state = load(p.clone(), state);
    let val = get(val_and_state.clone(), 0).with_arg_types(tuplet!(statet()), Type::Base(intt()));
    // let state = get(val_and_state, 1);
    let ten = int(10).with_arg_types(tuplet!(statet()), Type::Base(intt()));
    // let res = tprint(val, state).with_arg_types(tuplet!(statet()), Type::Base(statet()));
    egglog_test(
        &format!("(DemandPointsToCells {val})"),
        &format!(
            "
(check (PointsNowhere
         (IntersectPointees (PointsToCells {p} (PointsAnywhere))
                            (PointsToCells {r} (PointsAnywhere)))))

(check (DontAlias {p} {r} (PointsAnywhere)))
(check (= (PointsTo {state_after_rwrite} {p}) {ten}))
(check (= {val} {ten}))
            "
        ),
        vec![],
        val_empty(),
        val_empty(),
        vec![],
    )
}
