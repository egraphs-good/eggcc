#lang racket

(provide (all-defined-out))

;; Configurations
(define ignored-relations (list 'TmpCtx))
(define ignored-rulesets (list "debug-deletes"))
;; Spurious dependencies
(define ignored-computes (list
                          (cons "subst" 'HasArgType)
                          (cons "always-run-postprocess" 'tuple-length)

                          )
  )
(define grouping `(
                    ; (("type-analysis" "type-helpers" "always-run")
                    ;  .
                    ;  "type-analysis*\ntype-helpers*\nalways-run*"
                    ;  )
                   (("subst" "apply-subst-unions" "cleanup-subst")
                    .
                    "subst*\napply-subst-unions\ncleanup-subst")
                   (("drop" "apply-drop-unions" "cleanup-drop")
                    .
                    "drop*\napply-drop-unions\ncleanup-drop")
                   (("memory-helpers" "memory") . "(saturate\n(saturate memory-helpers)\n(saturate memory))")
                   (("loop-iters-analysis" "loop-unroll" "loop-peel") . "(saturate loop-iters-analysis)\nloop-unroll-and-unroll")

                   (("loop-unroll"
                     "switch_rewrite"
                     "loop-inv-motion"
                     "loop-strength-reduction"
                     "loop-peel"
                     "loop-inversion"
                     "loop-simplify"
                     "peepholes") . "all-optimizations")

                   )
  )

;; Getting all the rules
(define file "out.egg")
(define programs (file->list file))

(define (rule? e)
  (if (or (eq? (car e) 'rule)
          (eq? (car e) 'rewrite)
          (eq? (car e) 'birewrite))
      (match e
        [`(rule ,lhs ,rhs :ruleset ,ruleset) #t]
        [`(rewrite ,lhs ,rhs :ruleset ,ruleset) #t]
        [`(rewrite ,lhs ,rhs :when ,when :ruleset ,ruleset) #t]
        [else (error "unrecognized rule!" e)])
      #f)
  )
(define rules (filter rule? programs))

;; Getting all the rulesets
(define (rule-ruleset rule)
  (match rule
    [`(rule ,query ,action :ruleset ,ruleset) ruleset]
    [`(rewrite ,lhs ,rhs :ruleset ,ruleset) ruleset]
    [`(rewrite ,lhs ,rhs :when ,when :ruleset ,ruleset) ruleset]))
(define (rule-lhs-atoms rule)
  (match rule
    [`(rule ,query ,action :ruleset ,ruleset) query]
    [`(rewrite ,lhs ,rhs :ruleset ,ruleset) (list lhs)]
    [`(rewrite ,lhs ,rhs :when ,when :ruleset ,ruleset) (cons lhs when)]))
(define (rule-rhs-atoms rule)
  (match rule
    [`(rule ,query ,action :ruleset ,ruleset) action]
    [`(rewrite ,lhs ,rhs :ruleset ,ruleset) (list rhs)]
    [`(rewrite ,lhs ,rhs :when ,when :ruleset ,ruleset) (list rhs)]))

;; Getting the dependencies
(define (get-deps-from-atom atom acc)
  (if (list? atom)
      (let ([acc+ (cons (car atom) acc)])
        (foldr get-deps-from-atom acc+ (cdr atom)))
      acc))

;; Special handling of subsume and delete
(define (get-deps-from-action-atom atom acc)
  (if (list? atom)
      (if (member (car atom) '(subsume delete))
          (foldr get-deps-from-atom acc (cdr (second atom)))
          (let ([acc+ (cons (car atom) acc)])
            (foldr get-deps-from-atom acc+ (cdr atom))))
      acc))

(define AST (let ()
              (define schema (file->list "dag_in_context/src/schema.egg"))
              (define funcs (map second (filter (lambda (x) (eq? (car x) 'function)) schema)))
              (define datatypes (filter (lambda (x) (eq? (car x) 'datatype)) schema))
              (define (get-funcs-from-datatype dt) (map first (cddr dt)))
              (append funcs (append* (map get-funcs-from-datatype datatypes)))))
(define primitives (list '+ '- '* '/ 'max 'min '> '>= '< '% '<= '!= 'and 'or 'bool-<))

(define (get-query-deps rule)
  (define keywords (append (list '=) primitives AST ignored-relations))
  (define query (rule-lhs-atoms rule))
  (filter (lambda (ruleset) (not (member ruleset keywords)))
          (remove-duplicates (foldr get-deps-from-atom '() query))))

(define (get-action-deps rule)
  (define keywords (append (list 'union 'delete 'subsume 'set 'let 'extract 'panic) primitives AST ignored-relations))
  (define action (rule-rhs-atoms rule))
  (if (list? action)
      (filter (lambda (ruleset) (not (member ruleset keywords)))
              (remove-duplicates (foldr get-deps-from-action-atom '() action)))
      '()))

(struct dep (
             ruleset
             requires
             computes)
  #:transparent)

(define dependencies (make-hash))

(define (lookup-group-from-ruleset r)
  (define ruleset (if (symbol? r) (symbol->string r) r))
  (define res (findf (lambda (p)
                       (member ruleset (car p))) grouping))
  (if res
      (cdr res)
      ruleset))

(for ([rule rules])
  (define ruleset (lookup-group-from-ruleset (rule-ruleset rule)))
  (define query-deps (get-query-deps rule))
  (define action-deps (get-action-deps rule))
  (define d (dep ruleset query-deps action-deps))
  (hash-update! dependencies ruleset (lambda (x) (cons d x)) '())
  )

(for ([(ruleset deps) (in-hash dependencies)])
  (define query-deps (remove-duplicates (append* (map dep-requires deps))))
  (define action-deps (remove-duplicates (append* (map dep-computes deps))))
  (hash-set! dependencies ruleset (dep ruleset query-deps action-deps))
  )

(for ([ignored-ruleset ignored-rulesets])
  (hash-remove! dependencies ignored-ruleset))
(for ([ignored-compute ignored-computes])
  (define d (hash-ref dependencies (lookup-group-from-ruleset (car ignored-compute))))
  (define d+ (dep
              (dep-ruleset d)
              (dep-requires d)
              (remove (cdr ignored-compute) (dep-computes d))))
  (hash-set! dependencies (dep-ruleset d) d+)
  )



;; analyses
(define rulesets (hash-keys dependencies))
(define (list-intersect l1 l2)
  (filter (lambda (x) (member x l2)) l1))

; (for ([d (hash-values dependencies)])
;   (define recursive-nodes (list-intersect (dep-requires d) (dep-computes d)))
;   (when (not (null? recursive-nodes))
;     (displayln (dep-ruleset d))
;     (displayln recursive-nodes)
;     )
;   )

(define (generate-dot-from-dependencies dependencies file)
  (define rulesets (hash-keys dependencies))
  (with-output-to-file file
    (thunk
     (displayln "digraph G {")
     (for ([r rulesets])
       (displayln (format "  \"~a\" [shape=box];" (string-replace r "-" "_")))
       )

     (for* ([r1 rulesets]
            [r2 rulesets])
       (define r1-requires (dep-requires (hash-ref dependencies r1)))
       (define r2-computes (dep-computes (hash-ref dependencies r2)))
       (define isect (list-intersect r1-requires r2-computes))
       (when (not (null? isect))
         (define r1+ (string-replace r1 "-" "_"))
         (define r2+ (string-replace r2 "-" "_"))
         (define label (if (> (length isect) 3) (format "~a relations" (length isect)) (string-join (map symbol->string isect) ",")))
         (displayln (format "  \"~a\" -> \"~a\" [label=\"~a\"];" r2+ r1+ label))
         )
       )
     (displayln "}")
     )
    #:exists 'replace))

(define (generate-yaml-from-dependencies dependencies file)
  (with-output-to-file file
    (thunk
     (displayln "rulesets:")
     (for ([ruleset rulesets])
       (define d (hash-ref dependencies ruleset))
       (define (j strs)
         (string-join (map symbol->string strs) ", " #:before-first "[" #:after-last "]"))
       (displayln (format "  - name: \"~a\"" (string-replace ruleset "\n" "\\n")))
       (displayln (format "    requires: ~a" (j (dep-requires d))))
       (displayln (format "    computes: ~a" (j (dep-computes d))))
       )
     )
    #:exists 'replace))

(generate-dot-from-dependencies dependencies "dependencies.dot")
(generate-yaml-from-dependencies dependencies "dependencies.yaml")


(define helper-rulesets
  (list
   "type-helpers"
   "error-checking"
   "state-edge-passthrough"
   "drop"
   "apply-drop-unions"
   "cleanup-drop"
   "subst"
   "apply-subst-unions"
   "cleanup-subst"
   "subsume-after-helpers"
   "boundary-analysis"
   "always-run"
   "passthrough"
   "canon"
   "type-analysis"
   "context"
   "interval-analysis"
   "memory-helpers"
   "always-switch-rewrite"
   "loop-iters-analysis"
   "is-resolved"
   ))
(define helper-dependencies
  (for/hash ([r helper-rulesets])
    (define ruleset (lookup-group-from-ruleset r))
    (define d (hash-ref dependencies ruleset))
    (values ruleset d)
    ))
(generate-dot-from-dependencies helper-dependencies "helper-dependencies.dot")

; (displayln (dep-computes (hash-ref dependencies "always-run")))

(displayln (hash-ref dependencies "interval-analysis"))
