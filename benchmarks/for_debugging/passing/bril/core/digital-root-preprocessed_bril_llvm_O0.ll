; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpG3XPgN/postprocessed.ll'
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
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 100000
  br i1 %loop_cond_0, label %loop_body, label %loop_done

loop_body:                                        ; preds = %loop_cond
  call void @__orig_main(i64 %loop_counter_1)
  %loop_counter_2 = add i64 %loop_counter_1, 1
  br label %loop_cond

loop_done:                                        ; preds = %loop_cond
  ret void
}

define dso_local void @__orig_main(i64 %input) {
pre_entry:
  br label %begin

begin:                                            ; preds = %check_done, %pre_entry
  %result_1 = phi i64 [ %result_3.lcssa, %check_done ], [ 0, %pre_entry ]
  %input_1 = phi i64 [ %input_2, %check_done ], [ %input, %pre_entry ]
  %digit_0 = call i64 @__peel_last_digit(i64 %input_1)
  %input_2 = sdiv i64 %input_1, 10
  %result_2 = add i64 %result_1, %digit_0
  br label %check_result

check_result:                                     ; preds = %process_result, %begin
  %result_3 = phi i64 [ %result_5, %process_result ], [ %result_2, %begin ]
  call void @print_int(i64 %result_3)
  call void @print_newline()
  %processed_0 = call i1 @__is_single_digit(i64 %result_3)
  br i1 %processed_0, label %check_done, label %process_result

process_result:                                   ; preds = %check_result
  %r0_0 = call i64 @__peel_last_digit(i64 %result_3)
  %result_4 = sdiv i64 %result_3, 10
  %result_5 = add i64 %result_4, %r0_0
  br label %check_result

check_done:                                       ; preds = %check_result
  %result_3.lcssa = phi i64 [ %result_3, %check_result ]
  %input_1.off = add i64 %input_1, 9
  %done_0 = icmp ult i64 %input_1.off, 19
  br i1 %done_0, label %done, label %begin

done:                                             ; preds = %check_done
  %result_3.lcssa.lcssa = phi i64 [ %result_3.lcssa, %check_done ]
  call void @print_int(i64 %result_3.lcssa.lcssa)
  call void @print_newline()
  ret void
}

define dso_local i1 @__is_single_digit(i64 %input) {
pre_entry:
  %input.fr = freeze i64 %input
  %0 = srem i64 %input.fr, 10
  %result_0 = icmp eq i64 %input.fr, %0
  ret i1 %result_0
}

define dso_local i64 @__peel_last_digit(i64 %input) {
pre_entry:
  %input.fr = freeze i64 %input
  %0 = srem i64 %input.fr, 10
  ret i64 %0
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
