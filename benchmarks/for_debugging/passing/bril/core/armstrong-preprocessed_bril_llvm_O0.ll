; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpjbXm9x/postprocessed.ll'
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
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 1000000
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
  %digits_0 = call i64 @__getDigits(i64 %input)
  br label %loop

loop:                                             ; preds = %body, %pre_entry
  %tmp_1 = phi i64 [ %tmp_2, %body ], [ %input, %pre_entry ]
  %sum_1 = phi i64 [ %sum_2, %body ], [ 0, %pre_entry ]
  %b_0 = icmp sgt i64 %tmp_1, 0
  br i1 %b_0, label %body, label %done

body:                                             ; preds = %loop
  %digit_0 = call i64 @__mod(i64 %tmp_1, i64 10)
  %pow_0 = call i64 @__power(i64 %digit_0, i64 %digits_0)
  %sum_2 = add i64 %sum_1, %pow_0
  %tmp_2 = udiv i64 %tmp_1, 10
  br label %loop

done:                                             ; preds = %loop
  %sum_1.lcssa = phi i64 [ %sum_1, %loop ]
  %res_0 = icmp eq i64 %sum_1.lcssa, %input
  call void @print_bool(i1 %res_0)
  call void @print_newline()
  ret void
}

define dso_local i64 @__getDigits(i64 %n) {
pre_entry:
  %n.off = add i64 %n, 9
  %cond_0 = icmp ult i64 %n.off, 19
  br i1 %cond_0, label %then, label %else

then:                                             ; preds = %pre_entry
  ret i64 1

else:                                             ; preds = %pre_entry
  %div_0 = sdiv i64 %n, 10
  %rec_0 = call i64 @__getDigits(i64 %div_0)
  %res_0 = add i64 %rec_0, 1
  ret i64 %res_0
}

define dso_local i64 @__mod(i64 %a, i64 %b) {
pre_entry:
  %a.fr = freeze i64 %a
  %0 = srem i64 %a.fr, %b
  ret i64 %0
}

define dso_local i64 @__power(i64 %base, i64 %exp) {
pre_entry:
  br label %loop

loop:                                             ; preds = %body, %pre_entry
  %res_1 = phi i64 [ %res_2, %body ], [ 1, %pre_entry ]
  %exp_1 = phi i64 [ %exp_2, %body ], [ %exp, %pre_entry ]
  %b_0 = icmp eq i64 %exp_1, 0
  br i1 %b_0, label %done, label %body

body:                                             ; preds = %loop
  %res_2 = mul i64 %res_1, %base
  %exp_2 = add i64 %exp_1, -1
  br label %loop

done:                                             ; preds = %loop
  %res_1.lcssa = phi i64 [ %res_1, %loop ]
  ret i64 %res_1.lcssa
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
