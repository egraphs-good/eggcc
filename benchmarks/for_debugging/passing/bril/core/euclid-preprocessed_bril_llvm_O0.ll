; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpEEK7BV/postprocessed.ll'
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

loop_cond:                                        ; preds = %inner_done, %b0
  %loop_counter_1 = phi i64 [ %loop_counter_2, %inner_done ], [ 10, %b0 ]
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 1000
  br i1 %loop_cond_0, label %loop_body, label %loop_done

loop_body:                                        ; preds = %loop_cond
  br label %inner_cond

inner_cond:                                       ; preds = %inner_body, %loop_body
  %inner_counter_1 = phi i64 [ %inner_counter_2, %inner_body ], [ 10, %loop_body ]
  %inner_cond_0 = icmp slt i64 %inner_counter_1, 1000
  br i1 %inner_cond_0, label %inner_body, label %inner_done

inner_body:                                       ; preds = %inner_cond
  call void @__orig_main(i64 %loop_counter_1, i64 %inner_counter_1)
  %inner_counter_2 = add i64 %inner_counter_1, 1
  br label %inner_cond

inner_done:                                       ; preds = %inner_cond
  %loop_counter_2 = add i64 %loop_counter_1, 1
  br label %loop_cond

loop_done:                                        ; preds = %loop_cond
  ret void
}

define dso_local void @__orig_main(i64 %v0, i64 %v1) {
pre_entry:
  %f_0 = call i64 @__gcd(i64 %v0, i64 %v1)
  call void @print_int(i64 %f_0)
  call void @print_newline()
  ret void
}

define dso_local i64 @__mod(i64 %r, i64 %s) {
pre_entry:
  %r.fr = freeze i64 %r
  %0 = srem i64 %r.fr, %s
  ret i64 %0
}

define dso_local i64 @__gcd(i64 %a, i64 %b) {
pre_entry:
  br label %for.cond.5

for.cond.5:                                       ; preds = %for.body.5, %pre_entry
  %b_1 = phi i64 [ %v10_0, %for.body.5 ], [ %b, %pre_entry ]
  %a_1 = phi i64 [ %b_1, %for.body.5 ], [ %a, %pre_entry ]
  %cond_1.not = icmp eq i64 %b_1, 0
  br i1 %cond_1.not, label %for.end.5, label %for.body.5

for.body.5:                                       ; preds = %for.cond.5
  %v10_0 = call i64 @__mod(i64 %a_1, i64 %b_1)
  br label %for.cond.5

for.end.5:                                        ; preds = %for.cond.5
  %a_1.lcssa = phi i64 [ %a_1, %for.cond.5 ]
  ret i64 %a_1.lcssa
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
