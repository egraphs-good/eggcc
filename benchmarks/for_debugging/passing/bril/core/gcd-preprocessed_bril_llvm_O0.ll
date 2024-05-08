; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmprAMk9o/postprocessed.ll'
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

define dso_local void @__orig_main(i64 %op1, i64 %op2) {
pre_entry:
  br label %cmp.val.outer

cmp.val.outer:                                    ; preds = %pre_entry, %if.2
  %v1_1.ph = phi i64 [ %op2, %pre_entry ], [ %v3_2.lcssa2, %if.2 ]
  %v0_1.ph = phi i64 [ %op1, %pre_entry ], [ %v0_1.lcssa1, %if.2 ]
  br label %cmp.val

cmp.val:                                          ; preds = %cmp.val.outer, %else.2
  %v0_1 = phi i64 [ %v3_2, %else.2 ], [ %v0_1.ph, %cmp.val.outer ]
  %v2_0 = icmp slt i64 %v0_1, %v1_1.ph
  br i1 %v2_0, label %if.1, label %else.1

if.1:                                             ; preds = %cmp.val
  %v3_0 = sub i64 %v1_1.ph, %v0_1
  br label %loop.bound

else.1:                                           ; preds = %cmp.val
  %v3_1 = sub i64 %v0_1, %v1_1.ph
  br label %loop.bound

loop.bound:                                       ; preds = %else.1, %if.1
  %v3_2 = phi i64 [ %v3_1, %else.1 ], [ %v3_0, %if.1 ]
  %v4_0 = icmp eq i64 %v3_2, 0
  br i1 %v4_0, label %program.end, label %update.val

update.val:                                       ; preds = %loop.bound
  br i1 %v2_0, label %if.2, label %else.2

if.2:                                             ; preds = %update.val
  %v3_2.lcssa2 = phi i64 [ %v3_2, %update.val ]
  %v0_1.lcssa1 = phi i64 [ %v0_1, %update.val ]
  br label %cmp.val.outer

else.2:                                           ; preds = %update.val
  br label %cmp.val

program.end:                                      ; preds = %loop.bound
  %v1_1.ph.lcssa = phi i64 [ %v1_1.ph, %loop.bound ]
  call void @print_int(i64 %v1_1.ph.lcssa)
  call void @print_newline()
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
