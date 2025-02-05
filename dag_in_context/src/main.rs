use dag_in_context::{prologue, schedule::parallel_schedule};

fn main() {
    println!(
        "{} \n {}",
        prologue(),
        parallel_schedule(true /* with_lowering_peepholes */)
            .iter()
            .map(|s| s.egglog_schedule().to_string())
            .collect::<Vec<String>>()
            .join("\n")
    );
}
