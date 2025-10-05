use dag_in_context::{prologue, schedule::parallel_schedule, EggccConfig};

fn main() {
    println!(
        "{} \n {}",
        prologue(),
        parallel_schedule(&EggccConfig::default())
            .iter()
            .map(|s| s.egglog_schedule().to_string())
            .collect::<Vec<String>>()
            .join("\n")
    );
}
