use dag_in_context::{prologue, schedule::parallel_schedule};

fn main() {
    println!("{} \n {}", prologue(), parallel_schedule().join("\n"));
}
