#[test]
fn test_list_util() -> Result<(), egglog::Error> {

    let build = &*format!("
		(let id (Id 1))
		(let list (Cons (Num id 0) (Cons (Num id 1) (Cons (Num id 2) (Cons (Num id 3) (Cons (Num id 4) (Nil)))))))
		(let t (All (Sequential) list))
	");
	let check = &*format!("
		(check (= (ListExpr-ith list 1) (Num id 1)))
		(check (= (ListExpr-ith list 4) (Num id 4)))
		(check (= (ListExpr-length list) 5))
		
	");
	crate::run_test(build, check)
}