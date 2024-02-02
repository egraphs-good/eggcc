# Semantics of Tree Encoding with Inputs

## Comparison to previous "unique id" schemas

In previous schemas, we might write 
shared computation like this:
```
(Let id1 some_impure_computation
  (Add (Get (Arg id1) 0) (Get (Arg id1) 1)))
```

The reason for `id` is to give a context-specific equality relation
to this `Let`, allowing us to assume
the argument is equal to `some_impure_computation`.

`Input`s allow us to avoid this id by
allowing us to refer to the impure computation directly:

```
(Let
  (Add (Get (Input some_impure_computation) 0) (Get (Input some_impure_computation) 1)))
```

The semantics of an `Input` is that it is evaluated only once in its enclosing `Let`.
Each `Let` can only have one `Input`.

Now, no ids are necessary because the 
context is baked into the `Input` itself.
