use tree_in_context::prologue;

fn main() {
    println!("{}\n{}", prologue(), include_str!("schedule.egg"));
}
