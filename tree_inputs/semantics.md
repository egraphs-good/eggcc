# Semantics of Tree Encoding with Inputs

## Comparison to previous "unique id" schemas

In previous schemas, we might write 
shared computation like this:
```scheme
(Let id1 some_impure_computation
  (Add (Get (Arg id1) 0) (Get (Arg id1) 1)))
```

The reason for `id` is to give a context-specific equality relation
to this `Let`, allowing us to assume
the argument is equal to `some_impure_computation`.

`Input`s allow us to avoid this id by
allowing us to refer to the impure computation directly:

```scheme
(Let
  (Add (Get (Input some_impure_computation) 0) (Get (Input some_impure_computation) 1)))
```

The semantics of an `Input` is that it is evaluated only once in its enclosing `Let`.
Each `Let` can only have one `Input`.

Now, no ids are necessary because the 
context is baked into the `Input` itself.


## Regions

`Function`, `Loop`, and `Let` all
create regions.
`Function` and `Loop` only refer to `Arg` and sub-regions.
`Let` only refers to `Input` and
sub-regions.


For a valid program, there is a 1 to 1 correspondance between a `Let`
and its `Input`.


Here are some examples.
First lets define a couple constants:
```
(let zero (Const (Global) (Int 0)))
(let one (Const (Global) (Int 1)))
```


This program is valid. The outer `Let` has `(Input one)` and
the inner `Let` has `(Input zero)`:
```scheme
(Let
  (Add (Let (Neg (Input zero)))
       (Input one)))
```

This program is invalid, since
the `Let` has two unique `Input`s:
```
(Let
  (Add (Input zero)
       (Input one)))
```

An `Input` can also have sub-regions.
The following valid program reads from address 0, doubles it, and negates the result twice:
```
(Let
  (Neg
    (Input
      (Neg
        (Let
          (Add (Input (Read zero (IntT)))
               (Input (Read zero (IntT)))))))))
```


A `Let` evaluates its input only once.
This program prints zero twice:
```
(Let
  (All (Sequential)
       (Cons
         (Print zero)
         (Cons
           (Input (Print zero))
           (Input (Print zero))))))
```


Inputs are evaluated before anything else.
This program prints zero, then one:
```
(Let
  (All (Sequential)
       (Cons
         (Print one)
         (Cons
           (Input (Print zero))
           (Input (Print zero))))))
```

