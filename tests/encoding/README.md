# Restricted exact local address experiment

`--exact-local-addresses off|keys|query|manual` defaults to `off`. Enabled modes
require `--with-context` and reject `--ablate`. They apply only to state-only function inputs and
supported positive constant-sized integer allocations, bounded constant offsets,
and constant-integer writes with live origins. Other effects have no propagation
rule; existing source equalities or AST inlining can expose eligible forms.
Static allocation ids do not establish freshness. No annotation is put on Expr,
Const or other runtime constructors, and witnesses are not exported to global
NoAlias.

The existing source passes finish normally. Before each serialization, the
feature runs audited single-step key propagation, its selected consumer, existing
memory forwarding and type/indexing. The audit runs inside ordinal saturation and
before/after consumers; key, ordinal and injectivity conflicts abort before
native extraction. This is an ordinary-mode experiment, with no new proof support.

`keys` derives analysis and audits without new memory rewrites. `query` uses the
structural disequality witness. `manual` uses state-scoped primitive key-field
comparisons with the same eligibility contract. All enabled modes share the same
query-based consistency audit; manual is not query-free. Off-to-keys also adds
memory/type fixed-point closure after the source schedule, so it is not a pure
annotation-cost contrast. Query and manual share that closure. This candidate has no
asserted disequality workload, so NEE/EE assertion-storage costs are not isolated.

`fresh-allocations.bril` comes from the semantic study's source-derived two-store
example (`memory.rs::load_after_write_without_alias`): the load must return 17,
not 99. Off/keys retain that load; query/manual remove it through the real native
pipeline. `same-pointer.bril` and `zero-offset.bril` must both return the later 99.

Focused checks, from the compiler root:

```sh
cargo +1.91.0 test --locked --release --manifest-path dag_in_context/Cargo.toml --lib exact_local_addresses_
cargo +1.91.0 test --locked --release --no-default-features --test exact_local_addresses
cargo +1.91.0 test --locked --release --no-default-features --bin eggcc exact_address_cli_tests
```

The rejection tests exercise pointer/key/count/cyclic seeded violations in every
enabled mode and require rejection within 2 seconds before a serialization sentinel.
These are deliberate invariant violations, not discovered compiler bugs. The
native test uses all 3 default passes, contexts, greedy Tiger, production
non_weakly_linear=true and both interpreters. It checks non-null expected outputs,
off/keys source identity and actual load removal. CLI correctness additionally
requires checking both non-null JSON interpreter fields: `--interp` alone does
not assert agreement.
