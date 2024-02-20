use tree_assume::prologue;

fn main() {
    println!("{}\n{}", prologue(), include_str!("schedule.egg"));
}
