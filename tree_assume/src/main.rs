use tree_assume::egglog_test;
use tree_assume::interpreter::Value;
use tree_assume::Result;

fn main() -> Result {
    egglog_test("", "", vec![], Value::Tuple(vec![]), Value::Tuple(vec![]))
}
