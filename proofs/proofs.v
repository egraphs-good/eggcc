(* Coq 8.19.2 *)

Require Import Arith Lia List String.

Inductive Expr : Type :=
| Arg: Expr
| Num: nat -> Expr
| Boolean: bool -> Expr
| Add: Expr -> Expr -> Expr
| LessThan: Expr -> Expr -> Expr
(* Tuples *)
| Single: Expr -> Expr
| Concat: Expr -> Expr -> Expr
| Get : Expr -> nat -> Expr
(* Effects *)
| Print : Expr -> Expr -> Expr
| Store : Expr -> Expr -> Expr
| Load : Expr -> Expr
(* pred in then else *)
| If : Expr -> Expr -> Expr -> Expr -> Expr
(* in pred out *)
| DoWhile : Expr -> Expr -> Expr -> Expr
| Ctx : Assumption -> Expr -> Expr
with Assumption : Type :=
| InThen : Expr -> Expr -> Assumption
| InElse : Expr -> Expr -> Assumption
| InDoWhile : Expr -> Expr -> Expr -> Assumption
.

Inductive Typ : Type :=
| TInt: Typ
| TState: Typ
| TTuple: list Type -> Typ.

Record State := 
{
  (* For now, only nats can be stored
    addr -> option nat *)
  Mem: list (nat * option nat);
  Log: list nat
}.

Inductive Value : Type :=
| VNum: nat -> Value
| VBoolean: bool -> Value
| VTuple: list Value -> Value
| VState: State -> Value.

Definition initstate :=
  (VState {| Mem := nil; Log := nil |}).

Inductive nth_element {A : Type} : nat -> list A -> A -> Prop :=
| nth_zero : forall x l, nth_element 0 (x :: l) x
| nth_S : forall n x y l, nth_element n l x -> nth_element (S n) (y :: l) x.

(* Big-step judgement *)
Inductive BS : Value -> Expr -> Value -> Prop :=
| EArg : forall a,
  BS a Arg a
| ENum : forall a n,
  BS a (Num n) (VNum n)
| EBoolean : forall a b,
  BS a (Boolean b) (VBoolean b)
| EAdd : forall a e1 e2 n1 n2,
  BS a e1 (VNum n1) ->
  BS a e2 (VNum n2) ->
  BS a (Add e1 e2) (VNum (n1 + n2))
| ELessThan : forall a e1 e2 n1 n2,
  BS a e1 (VNum n1) ->
  BS a e2 (VNum n2) ->
  BS a (LessThan e1 e2) (VBoolean (Nat.ltb n1 n2))
| ESingle : forall a e v,
  BS a e v ->
  BS a (Single e) (VTuple (cons v nil))
| EConcat: forall a e1 e2 t1 t2,
  BS a e1 (VTuple t1) ->
  BS a e2 (VTuple t2) ->
  BS a (Concat e1 e2) (VTuple (t1 ++ t2))
| EGet: forall a e vs i v,
  BS a e (VTuple vs) ->
  nth_element i vs v ->
  BS a (Get e i) v
| EPrint : forall a estate mem log e n,
  BS a estate (VState {| Mem := mem; Log := log |}) ->
  BS a e (VNum n) ->
  BS a (Print estate e) (VState {| Mem := mem; Log := cons n log|})
| EIfTrue : forall a ep ei et ee vi vt,
  BS a ep (VBoolean true) ->
  BS a ei vi ->
  BS vi et vt ->
  BS a (If ep ei et ee) vt
| EIfFalse : forall a ep ei et ee vi ve,
  BS a ep (VBoolean false) ->
  BS a ei vi ->
  BS vi ee ve ->
  BS a (If ep ei et ee) ve
| EDoWhileTrue : forall a ei ep eo vi vo vfin,
  BS a ei vi ->
  BS vi ep (VBoolean true) ->
  BS vi eo vo ->
  BS vo (DoWhile Arg ep eo) vfin ->
  BS a (DoWhile ei ep eo) vfin
| EDoWhileFalse : forall a ei ep eo vi vo,
  BS a ei vi ->
  BS vi ep (VBoolean false) ->
  BS vi eo vo ->
  BS a (DoWhile ei ep eo) vo
| ECtxInThen : forall a ei ep e v a0,
  BS a e v ->
  BS a0 ei a ->
  BS a0 ep (VBoolean true) ->
  BS a (Ctx (InThen ep ei) e) v
| ECtxInElse : forall a ei ep e v a0,
  BS a e v ->
  BS a0 ei a ->
  BS a0 ep (VBoolean false) ->
  BS a (Ctx (InElse ep ei) e) v
| ECtxInDoWhile : forall a ei ep eo e v,
  BS a e v ->
  WhileProducesArg ei ep eo a ->
  BS a (Ctx (InDoWhile ei ep eo) e) v
with WhileProducesArg : Expr -> Expr -> Expr -> Value -> Prop :=
| InitArg : forall a0 ei ep eo vi,
  BS a0 ei vi ->
  WhileProducesArg ei ep eo vi
| NextArg : forall a ei ep eo vi,
  WhileProducesArg ei ep eo a ->
  BS a ep (VBoolean true) ->
  BS a ei vi ->
  WhileProducesArg ei ep eo vi.

Notation "A ⊢ B ⇓ C" := (BS A B C) (at level 80).

Theorem introduce_if_ctx : forall a ep ei et ee v,
  a ⊢ If ep ei et ee ⇓ v
  <->
  a ⊢ If ep ei (Ctx (InThen ep ei) et) (Ctx (InElse ep ei) ee) ⇓ v
.
Proof.
  split. 
  - intros.
    * inversion H.
      + eapply EIfTrue; auto.
        -- apply H7.
        -- eapply ECtxInThen; auto.
          ** apply H7.
          ** auto.
      + eapply EIfFalse; auto.
        -- apply H7.
        -- eapply ECtxInElse; auto.
          ** apply H7.
          ** auto.
  - intros. inversion H.
    * eapply EIfTrue; auto.
      + apply H7.
      + inversion H8; auto.
    * eapply EIfFalse; auto.
      + apply H7.
      + inversion H8; auto.
Qed.

Theorem ctx_allows_const_union : forall a ep n v,
  a ⊢ Ctx (InThen ep (Num n)) Arg ⇓ v
  <->
  a ⊢ Ctx (InThen ep (Num n)) (Num n) ⇓ v
.
Proof.
  split.
  - intros. inversion H.
    * eapply ECtxInThen.
      + subst. inversion H6. subst. inversion H4. subst. apply ENum.
      + subst. inversion H6. subst. inversion H4. subst. apply ENum.
      + subst. apply H7.
  - intros. inversion H.
    * eapply ECtxInThen.
      + subst. inversion H6. subst. inversion H4. subst. apply EArg.
      + subst. inversion H6. subst. inversion H4. subst. apply ENum.
      + subst. apply H7. 
Qed.

Theorem ctx_push_down_add : forall assum a l r v,
  a ⊢ Ctx assum (Add l r) ⇓ v
  <->
  a ⊢ Add (Ctx assum l) (Ctx assum r) ⇓ v
.
Proof.
  split.
  - intros. inversion H; subst.
    * inversion H2. subst. eapply EAdd; eapply ECtxInThen; eauto.
    * inversion H2. subst. eapply EAdd; eapply ECtxInElse; eauto.
    * inversion H3. subst. eapply EAdd; eapply ECtxInDoWhile; eauto.
  - intros. inversion H. subst. inversion H3; inversion H5; subst.
    * eapply ECtxInThen; eauto. eapply EAdd; eauto.
    * inversion H9.
    * inversion H9.
    * inversion H9.
    * eapply ECtxInElse; eauto. eapply EAdd; eauto.
    * inversion H9.
    * inversion H8.
    * inversion H8.
    * eapply ECtxInDoWhile; eauto. eapply EAdd; eauto.
Qed.

Theorem ctx_push_down_if : forall assum a ep ei et ee v,
  a ⊢ Ctx assum (If ep ei et ee) ⇓ v
  <->
  a ⊢ If (Ctx assum ep) (Ctx assum ei) et ee ⇓ v
.
Proof.
  split.
  - intros. inversion H; subst.
    * inversion H2. subst.
      + eapply EIfTrue; eauto; eapply ECtxInThen; eauto.
      + eapply EIfFalse; eauto; eapply ECtxInThen; eauto.
    * inversion H2. subst.
      + eapply EIfTrue; eauto; eapply ECtxInElse; eauto.
      + eapply EIfFalse; eauto; eapply ECtxInElse; eauto.
    * inversion H3. subst.
      + eapply EIfTrue; eauto; eapply ECtxInDoWhile; eauto.
      + eapply EIfFalse; eauto; eapply ECtxInDoWhile; eauto.
  - intros. inversion H; subst; inversion H6; inversion H7; subst.
    * eapply ECtxInThen; eauto. eapply EIfTrue; eauto.
    * inversion H10.
    * inversion H10.
    * inversion H10.
    * eapply ECtxInElse; eauto. eapply EIfTrue; eauto.
    * inversion H10.
    * inversion H9.
    * inversion H9.
    * eapply ECtxInDoWhile; eauto. eapply EIfTrue; eauto.
    * eapply ECtxInThen; eauto. eapply EIfFalse; eauto.
    * inversion H10.
    * inversion H10.
    * inversion H10.
    * eapply ECtxInElse; eauto. eapply EIfFalse; eauto.
    * inversion H10.
    * inversion H9.
    * inversion H9.
    * eapply ECtxInDoWhile; eauto. eapply EIfFalse; eauto.
Qed.
