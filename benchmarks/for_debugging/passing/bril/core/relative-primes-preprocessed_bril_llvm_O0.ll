; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpohXJlf/postprocessed.ll'
source_filename = "stdin"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"
target triple = "arm64-apple-macosx13.0.0"

@.str = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.str.1 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@.str.2 = private unnamed_addr constant [4 x i8] c"%ld\00", align 1
@.str.3 = private unnamed_addr constant [9 x i8] c"[object]\00", align 1
@.str.4 = private unnamed_addr constant [33 x i8] c"error: expected %d args, got %d\0A\00", align 1

declare dso_local i32 @putchar(i32)

declare dso_local i32 @printf(ptr, ...)

declare dso_local void @exit(i32)

declare dso_local i64 @atol(ptr)

declare dso_local noalias ptr @malloc(i64)

declare dso_local void @free(ptr)

define dso_local i32 @btoi(ptr %0) {
  %2 = load i8, ptr %0, align 1
  %3 = icmp eq i8 %2, 116
  %4 = zext i1 %3 to i32
  ret i32 %4
}

define dso_local void @print_bool(i1 %0) {
  br i1 %0, label %2, label %4

2:                                                ; preds = %1
  %3 = call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str)
  br label %6

4:                                                ; preds = %1
  %5 = call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str.1)
  br label %6

6:                                                ; preds = %4, %2
  ret void
}

define dso_local void @print_space() {
  %1 = call i32 @putchar(i32 32)
  ret void
}

define dso_local void @print_newline() {
  %1 = call i32 @putchar(i32 10)
  ret void
}

define dso_local void @print_int(i64 %0) {
  %2 = call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str.2, i64 %0)
  ret void
}

define dso_local void @print_ptr(ptr %0) {
  %2 = call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str.3)
  ret void
}

define dso_local void @__main() {
b0:
  br label %loop_cond

loop_cond:                                        ; preds = %loop_body, %b0
  %loop_counter_1 = phi i64 [ %loop_counter_2, %loop_body ], [ 10, %b0 ]
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 3000
  br i1 %loop_cond_0, label %loop_body, label %loop_done

loop_body:                                        ; preds = %loop_cond
  call void @__orig_main(i64 %loop_counter_1)
  %loop_counter_2 = add i64 %loop_counter_1, 1
  br label %loop_cond

loop_done:                                        ; preds = %loop_cond
  ret void
}

define dso_local void @__orig_main(i64 %v0) {
pre_entry:
  call void @__relative_primes(i64 %v0)
  ret void
}

define dso_local i64 @__mod(i64 %a, i64 %b) {
pre_entry:
  %a.fr = freeze i64 %a
  %0 = srem i64 %a.fr, %b
  ret i64 %0
}

define dso_local i64 @__gcd(i64 %a, i64 %b) {
pre_entry:
  %v3_0 = icmp sgt i64 %b, %a
  br i1 %v3_0, label %then.0, label %else.0

then.0:                                           ; preds = %pre_entry
  br label %else.0

else.0:                                           ; preds = %then.0, %pre_entry
  %b_2 = phi i64 [ %a, %then.0 ], [ %b, %pre_entry ]
  %a_2 = phi i64 [ %b, %then.0 ], [ %a, %pre_entry ]
  %v10_0 = icmp eq i64 %a_2, 0
  br i1 %v10_0, label %then.7, label %else.7

then.7:                                           ; preds = %else.0
  ret i64 %b_2

b4:                                               ; No predecessors!
  br label %else.12

else.7:                                           ; preds = %else.0
  %v15 = icmp eq i64 %b, 0
  br i1 %v15, label %then.12, label %else.12

then.12:                                          ; preds = %else.7
  ret i64 %b

b7:                                               ; No predecessors!
  br label %else.12

else.12:                                          ; preds = %b7, %else.7, %b4
  %remainder = call i64 @__mod(i64 %a, i64 %b)
  %g = call i64 @__gcd(i64 %b, i64 %remainder)
  ret i64 %g
}

define dso_local void @__relative_primes(i64 %a) {
pre_entry:
  br label %for.cond.0

for.cond.0:                                       ; preds = %else.7, %pre_entry
  %b_1 = phi i64 [ %v15_0, %else.7 ], [ %a, %pre_entry ]
  %v4_0 = icmp sgt i64 %b_1, 0
  br i1 %v4_0, label %for.body.0, label %for.end.0

for.body.0:                                       ; preds = %for.cond.0
  %g_0 = call i64 @__gcd(i64 %a, i64 %b_1)
  %v10_0 = icmp eq i64 %g_0, 1
  br i1 %v10_0, label %then.7, label %else.7

then.7:                                           ; preds = %for.body.0
  call void @print_int(i64 %b_1)
  call void @print_newline()
  br label %else.7

else.7:                                           ; preds = %then.7, %for.body.0
  %v15_0 = add nsw i64 %b_1, -1
  br label %for.cond.0

for.end.0:                                        ; preds = %for.cond.0
  ret void
}

define dso_local i32 @main(i32 %argc, ptr %argv) {
  %.not = icmp eq i32 %argc, 1
  br i1 %.not, label %4, label %1

1:                                                ; preds = %0
  %2 = add nsw i32 %argc, -1
  %3 = call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str.4, i32 0, i32 %2)
  call void @exit(i32 2)
  unreachable

4:                                                ; preds = %0
  call void @__main()
  ret i32 0
}
