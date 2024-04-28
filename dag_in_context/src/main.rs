use dag_in_context::{prologue, schedule::mk_schedule};

fn main() {
    println!("{} \n {}", prologue(), mk_schedule());
}
