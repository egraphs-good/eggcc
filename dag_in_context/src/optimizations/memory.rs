// Signature of an egglog function, for metaprogramming
fn listlike(el_tys: Vec<&str>, el_relations: Vec<&str>) -> String {
    assert!(!el_tys.is_empty());
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

(function Length-{datatype} ({datatype}) i64)
(rule ((= x (Nil-{datatype})))
      ((set (Length-{datatype} x) 0))
      :ruleset always-run)
(rule ((= x (Cons-{datatype} {el} tl))
       (= l (Length-{datatype} tl)))
      ((set (Length-{datatype} x) (+ l 1)))
      :ruleset always-run)
(rule ((= x (Nil-{datatype})))
      ((set (Length-{datatype} x) 0))
      :ruleset memory-helpers)
(rule ((= x (Cons-{datatype} {el} tl))
       (= l (Length-{datatype} tl)))
      ((set (Length-{datatype} x) (+ l 1)))
      :ruleset memory-helpers)

(relation IsEmpty-{datatype} ({datatype}))
(rule ((= x (Nil-{datatype})))
      ((IsEmpty-{datatype} x))
      :ruleset always-run)

(relation IsNonEmpty-{datatype} ({datatype}))
(rule ((= x (Cons-{datatype} {el} tl)))
      ((IsNonEmpty-{datatype} x))
      :ruleset always-run)

(function RevConcat-{datatype} ({datatype} {datatype}) {datatype} :cost 1000)
(rewrite (RevConcat-{datatype} (Nil-{datatype}) l)
         l
         :ruleset always-run)
(rewrite (RevConcat-{datatype} (Cons-{datatype} {el} tl) l)
         (RevConcat-{datatype} tl (Cons-{datatype} {el} l))
         :ruleset always-run)

(function Rev-{datatype} ({datatype}) {datatype} :cost 1000)
(rewrite (Rev-{datatype} m)
         (RevConcat-{datatype} m (Nil-{datatype}))
         :ruleset always-run)

(function Concat-{datatype} ({datatype} {datatype}) {datatype} :cost 1000)
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
    res.join("\n")
}

#[allow(non_snake_case)]
pub(crate) fn rules() -> String {
    let Listi64IntInterval = listlike(vec!["i64", "IntInterval"], vec![]);
    let ListPointees = listlike(vec!["PtrPointees"], vec!["PointsNowhere-PtrPointees"]);
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
          List<i64+IntInterval>
          :cost 100)
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

(datatype PtrPointees
  (PointsTo List<i64+IntInterval>)
  (PointsAnywhere))

(function AddIntIntervalToPtrPointees (IntInterval PtrPointees) PtrPointees)
(rewrite (AddIntIntervalToPtrPointees interval (PointsAnywhere))
         (PointsAnywhere)
         :ruleset always-run)
(rewrite (AddIntIntervalToPtrPointees interval (PointsTo l))
         (PointsTo (AddIntIntervalToAll interval l))
         :ruleset always-run)

(function Union-PtrPointees (PtrPointees PtrPointees) PtrPointees)
(rewrite (Union-PtrPointees (PointsAnywhere) _)
         (PointsAnywhere)
         :ruleset always-run)
(rewrite (Union-PtrPointees _ (PointsAnywhere))
         (PointsAnywhere)
         :ruleset always-run)
(rewrite (Union-PtrPointees (PointsTo x) (PointsTo y))
         (PointsTo (Union-List<i64+IntInterval> x y))
         :ruleset always-run)
(function Intersect-PtrPointees (PtrPointees PtrPointees) PtrPointees)
(rewrite (Intersect-PtrPointees (PointsAnywhere) x)
         x
         :ruleset always-run)
(rewrite (Intersect-PtrPointees x (PointsAnywhere))
         x
         :ruleset always-run)
(rewrite (Intersect-PtrPointees (PointsTo x) (PointsTo y))
         (PointsTo (Intersect-List<i64+IntInterval> x y))
         :ruleset always-run)

(relation PointsNowhere-PtrPointees (PtrPointees))
(rule ((= f (PointsTo x))
       (IsEmpty-List<i64+IntInterval> x))
      ((PointsNowhere-PtrPointees f))
      :ruleset always-run)

{ListPointees}


(function Zip<Union-PtrPointees> (List<PtrPointees> List<PtrPointees>) List<PtrPointees> :cost 1000)
(rewrite (Zip<Union-PtrPointees> (Nil-List<PtrPointees>) (Nil-List<PtrPointees>))
         (Nil-List<PtrPointees>)
         :ruleset always-run)
(rewrite (Zip<Union-PtrPointees>
           (Cons-List<PtrPointees> x0 tl1)
           (Cons-List<PtrPointees> y0 tl2))
         (Cons-List<PtrPointees>
            (Union-PtrPointees x0 y0)
            (Zip<Union-PtrPointees> tl1 tl2))
         :when ((= (Length-List<PtrPointees> tl1) (Length-List<PtrPointees> tl2)))
         :ruleset always-run)

(function Zip<Intersect-PtrPointees> (List<PtrPointees> List<PtrPointees>) List<PtrPointees> :cost 1000)
(rewrite (Zip<Intersect-PtrPointees> (Nil-List<PtrPointees>) (Nil-List<PtrPointees>))
         (Nil-List<PtrPointees>)
         :ruleset always-run)
(rewrite (Zip<Intersect-PtrPointees>
           (Cons-List<PtrPointees> x0 tl1)
           (Cons-List<PtrPointees> y0 tl2))
         (Cons-List<PtrPointees>
            (Intersect-PtrPointees x0 y0)
            (Zip<Intersect-PtrPointees> tl1 tl2))
         :ruleset always-run)

"
    )
}

#[cfg(test)]
use crate::egglog_test;

#[cfg(test)]
use crate::schema::TreeProgram;
#[cfg(test)]
use crate::schema::{BaseType, Type};
#[cfg(test)]
use crate::Value;
#[cfg(test)]
use main_error::MainError;

#[cfg(test)]
// TODO we don't use memory in the main schedule yet
// so here enable it for tests
fn memory_egglog_test(
    build: &str,
    check: &str,
    progs: Vec<TreeProgram>,
    input: Value,
    expected: Value,
    expected_log: Vec<String>,
) -> Result<(), MainError> {
    egglog_test(
        build,
        &format!(
            "
    ;; TODO we don't run memory in the main loop right now
    (run-schedule
        (repeat 6
        (saturate
            always-run
            memory-helpers)
        memory))        
        {check}"
        ),
        progs,
        input,
        expected,
        expected_log,
    )
}

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
    let two = int(2).with_arg_types(tuplet!(statet()), Type::Base(intt()));
    let orig_state = get(arg_ty(tuplet!(statet())), 0);
    let ptr_and_state = alloc(0, one, orig_state.clone(), pointert(intt()));
    let ptr = get(ptr_and_state.clone(), 0);
    let state = get(ptr_and_state, 1);
    let state = write(ptr.clone(), two.clone(), state);
    let val_and_state = load(ptr, state);
    let val = get(val_and_state.clone(), 0);
    let state = get(val_and_state, 1);
    let res = tprint(val, state);

    memory_egglog_test(
        &format!("{res}"),
        &format!(
            "
        (check (= {res} (Bop (Print) {two} rest)))"
        ),
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
    let one = int(1);
    let two = int(2).with_arg_types(tuplet!(statet()), Type::Base(intt()));
    let three = int(3);
    let orig_state = getat(0);
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
    let res = tprint(val, state).with_arg_types(tuplet!(statet()), Type::Base(statet()));
    let f = function("main", tuplet!(statet()), Type::Base(statet()), res.clone())
        .func_with_arg_types();
    memory_egglog_test(
        &format!("{f}"),
        &format!(
            "
        ;; TODO we don't run memory in the main loop right now
        (run-schedule
          (repeat 6
            (saturate
                always-run
                memory-helpers)
            memory))
        
        (check (= {res} (Bop (Print) {two} rest)))"
        ),
        vec![],
        val_empty(),
        val_empty(),
        vec![],
    )
}

#[test]
fn simple_loop_swap() -> crate::Result {
    // p = alloc(alloc_id, 4, int*)
    // q = ptradd(p, 1)
    // r = ptradd(p, 2)
    // // (p, r) don't alias
    // do {
    //   p, q = q, p
    // } while true;
    // // (p, r) still shouldn't alias
    use crate::ast::*;
    let alloc_id = 1;
    let state = getat(0);
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
    let state = write(p.clone(), int(10), state);
    let state = write(r.clone(), int(20), state);
    let val_and_state = load(p.clone(), state);
    let val = get(val_and_state.clone(), 0).with_arg_types(tuplet!(statet()), Type::Base(intt()));
    let ten = int(10).with_arg_types(tuplet!(statet()), Type::Base(intt()));
    let f =
        function("main", tuplet!(statet()), Type::Base(intt()), val.clone()).func_with_arg_types();
    memory_egglog_test(
        &format!("{f}"),
        &format!(
            "
        ;; TODO we don't run memory in the main loop right now
        (run-schedule
          (repeat 6
            (saturate
                always-run
                memory-helpers)
            memory))
        (let ten {ten}) (let val {val}) (check (= val ten))"
        ),
        vec![],
        val_empty(),
        val_empty(),
        vec![],
    )
}

#[test]
fn pqrs_deep_loop_swap() -> crate::Result {
    // p = alloc(alloc_id, 4, int*)
    // q = ptradd(p, 1)
    // r = ptradd(p, 2)
    // s = ptradd(p, 3)
    // // (p, r), (p, s), (q, r), (q, s) don't alias
    // do {
    //   do {
    //     p, q = q, p
    //   } while true;
    //   r, s = r, s
    // } while true;
    // // (p, r), (p, s), (q, r), (q, s) still shouldn't alias
    use crate::ast::*;
    let concat3 = |x, y, z| concat(x, concat(y, z));
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
        concat3(
            single(ttrue()), // pred
            dowhile(
                parallel!(
                    getat(0), // state
                    getat(1), // p
                    getat(2), // q
                ),
                parallel!(
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
    let val_and_state = load(p.clone(), state);
    let val = get(val_and_state.clone(), 0).with_arg_types(tuplet!(statet()), Type::Base(intt()));
    let ten = int(10).with_arg_types(tuplet!(statet()), Type::Base(intt()));
    let f =
        function("main", tuplet!(statet()), Type::Base(intt()), val.clone()).func_with_arg_types();
    memory_egglog_test(
        &format!("{f}"),
        &format!(
            "
        (let ten {ten}) (let val {val}) (check (= val ten))"
        ),
        vec![],
        val_empty(),
        val_empty(),
        vec![],
    )
}

#[test]
fn redundant_load_elim() -> crate::Result {
    use crate::ast::*;
    let alloc_id = 1;
    let state = getat(0);
    let p_and_state = alloc(alloc_id, int(4), state, pointert(intt()));
    let p = get(p_and_state.clone(), 0);
    // if (Arg 1) { *p = 2 } else { *p = 3 }
    let state = tif(
        getat(1),
        p_and_state,
        write(getat(0), int(2), getat(1)),
        write(getat(0), int(3), getat(1)),
    );
    // load p
    let val_and_state = load(p.clone(), state);
    let load1_val = get(val_and_state.clone(), 0)
        .with_arg_types(tuplet!(statet(), boolt()), Type::Base(intt()));
    let state = get(val_and_state, 1);
    // load p again and print
    let val_and_state = load(p.clone(), state);
    let val = get(val_and_state.clone(), 0);
    let state = get(val_and_state, 1);
    let res = tprint(val, state).with_arg_types(tuplet!(statet(), boolt()), Type::Base(statet()));
    let f = function(
        "main",
        tuplet!(statet(), boolt()),
        Type::Base(statet()),
        res.clone(),
    )
    .func_with_arg_types();
    memory_egglog_test(
        &format!("{f}"),
        &format!(
            "
        ;; TODO we don't run memory in the main loop right now
        (run-schedule
          (repeat 6
            (saturate
                always-run
                memory-helpers)
            memory))
        (print-function PointsToExpr 1000)
        (check (= {res} (Bop (Print) {load1_val} rest)))"
        ),
        vec![],
        val_empty(),
        val_empty(),
        vec![],
    )
}
