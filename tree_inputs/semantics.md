;; Input schema idea

;; current schema:
(Let id1 some_input
  (Add (Get (Arg id1) 0) (Get (Arg id1) 1)))

=>

;; input appears twice but evaluated only once
;; "dag semantics for inputs"
(Let
  (Add (Get (Input some_input) 0) (Get (Input some_input) 1)))


(Let
  (Add (Get (Input inner) 0) (Get (Input inner) 1)))


(Loop
  (Add (Input some_loop_inputs) (Input some_loop_inputs)))

