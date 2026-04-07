use eggcc::util::{Run, RunMode, TestProgram};
use std::collections::HashMap;
use std::io::Write;

const INFINITE_LOOP_BRIL: &str = include_str!("infinite_loop_preserved.bril");

/// Returns true if the Bril text has a jump/branch target to an earlier label.
fn has_back_edge(bril_text: &str) -> bool {
    let mut label_line = HashMap::<String, usize>::new();
    let mut pending_edges = Vec::<(usize, String)>::new();

    for (line_no, raw_line) in bril_text.lines().enumerate() {
        let line = raw_line.trim();

        if let Some(label) = line.strip_prefix('.') {
            if let Some(name) = label.strip_suffix(':') {
                label_line.insert(name.to_string(), line_no);
            }
            continue;
        }

        if let Some(target) = line.strip_prefix("jmp .") {
            if let Some(name) = target.strip_suffix(';') {
                pending_edges.push((line_no, name.to_string()));
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("br ") {
            let mut tokens = rest.split_whitespace();
            let _cond = tokens.next();
            for tok in tokens {
                if let Some(label) = tok.strip_prefix('.') {
                    let cleaned = label.trim_end_matches(';').to_string();
                    pending_edges.push((line_no, cleaned));
                }
            }
        }
    }

    pending_edges.into_iter().any(|(src_line, target)| {
        label_line
            .get(&target)
            .is_some_and(|dst_line| *dst_line < src_line)
    })
}

#[test]
fn infinite_loop_not_optimized_away() {
    let mut input = tempfile::NamedTempFile::new().expect("create temp bril file");
    input
        .write_all(INFINITE_LOOP_BRIL.as_bytes())
        .expect("write bril program");

    let run = Run::new(
        TestProgram::BrilFile(input.path().to_path_buf()).read_program(),
        RunMode::Optimize,
    );
    let result = run.run().expect("optimize program");

    let optimized = &result
        .visualizations
        .first()
        .expect("expected optimized bril visualization")
        .result;

    assert!(
        has_back_edge(optimized),
        "optimized program should still contain a loop/back-edge, got:\n{optimized}"
    );
}
