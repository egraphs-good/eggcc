//! Opt-in structural disequality for the existing ExpectType diagnostic.
//!
//! Contract: goals/eggcc-combined-encoding/findings/type-contract.md in the study
//! workspace. Ground finite free types only; helpers retain their normalization.
use clap::ValueEnum;
use egglog::{ast::Literal, EGraph, Term};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum StructuralTypeMode {
    #[default]
    Off,
    Manual,
    Combined,
    AnnotationsOff,
    /// Keep the same consumer, but consult explicit Type tag witnesses only.
    QueryOff,
}

pub(crate) fn schema(mode: StructuralTypeMode) -> String {
    let mut schema = include_str!("schema.egg").to_owned();
    if matches!(
        mode,
        StructuralTypeMode::Off | StructuralTypeMode::AnnotationsOff
    ) {
        return schema;
    }
    for signature in [
        "(PointerT BaseType)",
        "(Base BaseType)",
        "(TupleT TypeList)",
        "(constructor TCons (BaseType TypeList) TypeList)",
    ] {
        assert_eq!(schema.matches(signature).count(), 1);
        let marked = format!("{} :injective)", &signature[..signature.len() - 1]);
        schema = schema.replace(signature, &marked);
    }
    schema
}

pub(crate) fn type_analysis(mode: StructuralTypeMode) -> String {
    let original = include_str!("type_analysis.egg");
    if mode == StructuralTypeMode::Off {
        return original.to_owned();
    }
    const GUARD: &str = "(!= expected actual) ;; not okay unless we saturate type helpers.";
    assert_eq!(original.matches(GUARD).count(), 1);
    // Shared declarations must precede the existing diagnostic that uses them.
    // Its producers and panic action stay unchanged.
    let (declarations, preparation) = include_str!("structural_types.egg")
        .split_once("(StructuralGroundBase (IntT))")
        .unwrap();
    let analysis = original.replace(GUARD, "(StructuralTypeMismatch expected actual)");
    let manual = if mode == StructuralTypeMode::Manual {
        include_str!("structural_types_manual.egg")
    } else {
        ""
    };
    let predicate = match mode {
        StructuralTypeMode::Manual => "(StructuralManualTypeDQ expected actual)",
        StructuralTypeMode::QueryOff => "(StructuralTypeTagDQ expected actual)",
        StructuralTypeMode::Combined | StructuralTypeMode::AnnotationsOff => {
            "(known-disequal? expected actual)"
        }
        StructuralTypeMode::Off => unreachable!(),
    };
    format!(
        "{declarations}\n{analysis}\n(StructuralGroundBase (IntT)){preparation}\n{manual}
         (rule ((ExpectType e expected message) (HasType e actual)
                (StructuralGroundType expected) (StructuralGroundType actual)
                {predicate})
               ((StructuralTypeMismatch expected actual))
               :ruleset structural-type-consume :naive)"
    )
}

pub(crate) fn schedule(mode: StructuralTypeMode, schedule: &str) -> String {
    if mode == StructuralTypeMode::Off {
        return schedule.to_owned();
    }
    // Helper closure commits ground rows and tag assertions before the query.
    assert!(schedule.contains("error-checking"));
    schedule.replace(
        "error-checking",
        "(saturate structural-type-consume) error-checking",
    )
}

/// Audit every actual alternative before serialization. Query unknown and
/// unsupported forms are explicit errors, including in feature-removal arms.
pub(crate) fn validate(graph: &EGraph) -> Result<usize, egglog::Error> {
    let mut obligations = 0;
    for (table, accepted) in [
        ("StructuralTypeHasActual", 1),
        ("StructuralTypeCoverage", 2),
    ] {
        let (rows, outputs, dag) = graph.function_to_dag(table, usize::MAX, true)?;
        let outputs = outputs.expect("coverage functions return outputs");
        if table == "StructuralTypeCoverage" {
            obligations = rows.len();
        }
        for (row, output) in rows.into_iter().zip(outputs) {
            let Term::Lit(Literal::Int(status)) = dag.get(output) else {
                unreachable!("coverage functions return i64")
            };
            if *status != accepted {
                let reason = match (table, *status) {
                    ("StructuralTypeHasActual", _) => "missing-actual",
                    (_, 0) => "unsupported",
                    _ => "coverage-unknown",
                };
                return Err(egglog::Error::BackendError(format!(
                    "structural-type-{reason}: {}",
                    dag.to_string(row)
                )));
            }
        }
    }
    if obligations == 0 {
        return Err(egglog::Error::BackendError(
            "structural-type-no-obligations: no ExpectType consumer was reached".into(),
        ));
    }
    Ok(obligations)
}

#[cfg(test)]
mod tests;
