; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmp7cz6oZ/postprocessed.ll'
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

define dso_local ptr @__pack(i64 %size, i64 %n1, i64 %n2, i64 %n3, i64 %n4, i64 %n5) {
pre_entry:
  %z0 = shl i64 %size, 3
  %z1 = call ptr @malloc(i64 %z0)
  store i64 %n1, ptr %z1, align 8
  %loc_1 = getelementptr inbounds i64, ptr %z1, i64 1
  store i64 %n2, ptr %loc_1, align 8
  %loc_2 = getelementptr inbounds i64, ptr %z1, i64 2
  store i64 %n3, ptr %loc_2, align 8
  %loc_3 = getelementptr inbounds i64, ptr %z1, i64 3
  store i64 %n4, ptr %loc_3, align 8
  %loc_4 = getelementptr inbounds i64, ptr %z1, i64 4
  store i64 %n5, ptr %loc_4, align 8
  ret ptr %z1
}

define dso_local void @__print_array(ptr %array, i64 %size) {
pre_entry:
  br label %loop

loop:                                             ; preds = %loop_end, %pre_entry
  %i_1 = phi i64 [ %i_2, %loop_end ], [ 0, %pre_entry ]
  %cond_0 = icmp slt i64 %i_1, %size
  br i1 %cond_0, label %body, label %done

body:                                             ; preds = %loop
  %loc_0 = getelementptr inbounds i64, ptr %array, i64 %i_1
  %val_0 = load i64, ptr %loc_0, align 8
  call void @print_int(i64 %val_0)
  call void @print_newline()
  br label %loop_end

loop_end:                                         ; preds = %body
  %i_2 = add i64 %i_1, 1
  br label %loop

done:                                             ; preds = %loop
  ret void
}

define dso_local i64 @__binary_search(ptr %array, i64 %target, i64 %left, i64 %right) {
pre_entry:
  br label %check_valid

check_valid:                                      ; preds = %pre_entry
  %end_cond_0 = icmp sgt i64 %left, %right
  br i1 %end_cond_0, label %end, label %body

body:                                             ; preds = %check_valid
  %mid_0 = add i64 %left, %right
  %mid_1 = sdiv i64 %mid_0, 2
  br label %check_equal

check_equal:                                      ; preds = %body
  %midloc_0 = getelementptr inbounds i64, ptr %array, i64 %mid_1
  %midval_0 = load i64, ptr %midloc_0, align 8
  %equal_cond_0 = icmp eq i64 %midval_0, %target
  br i1 %equal_cond_0, label %found, label %check_gt

found:                                            ; preds = %check_equal
  ret i64 %mid_1

check_gt:                                         ; preds = %check_equal
  %gt_cond_0 = icmp slt i64 %midval_0, %target
  br i1 %gt_cond_0, label %call_gt, label %call_lt

call_gt:                                          ; preds = %check_gt
  %newleft_0 = add i64 %left, 1
  %output_0 = call i64 @__binary_search(ptr %array, i64 %target, i64 %newleft_0, i64 %right)
  ret i64 %output_0

call_lt:                                          ; preds = %check_gt
  %newright_0 = add i64 %right, -1
  %output_1 = call i64 @__binary_search(ptr %array, i64 %target, i64 %left, i64 %newright_0)
  ret i64 %output_1

end:                                              ; preds = %check_valid
  ret i64 -1
}

define dso_local void @__orig_main(i64 %e1, i64 %e2, i64 %e3, i64 %e4, i64 %e5) {
pre_entry:
  %array_0 = call ptr @__pack(i64 5, i64 %e1, i64 %e2, i64 %e3, i64 %e4, i64 %e5)
  %output_0 = call i64 @__binary_search(ptr %array_0, i64 7, i64 0, i64 4)
  call void @print_int(i64 %output_0)
  call void @print_newline()
  call void @free(ptr %array_0)
  ret void
}

define dso_local void @__main() {
b0:
  br label %loop_cond

loop_cond:                                        ; preds = %loop2_done, %b0
  %loop_counter_1 = phi i64 [ %loop_counter_2, %loop2_done ], [ 10, %b0 ]
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 25
  br i1 %loop_cond_0, label %loop_body, label %loop_done

loop_body:                                        ; preds = %loop_cond
  br label %loop2_cond

loop2_cond:                                       ; preds = %loop3_done, %loop_body
  %loop2_counter_1 = phi i64 [ %loop2_counter_2, %loop3_done ], [ 10, %loop_body ]
  %loop2_cond_0 = icmp slt i64 %loop2_counter_1, 25
  br i1 %loop2_cond_0, label %loop2_body, label %loop2_done

loop2_body:                                       ; preds = %loop2_cond
  br label %loop3_cond

loop3_cond:                                       ; preds = %loop4_done, %loop2_body
  %loop3_counter_1 = phi i64 [ %loop3_counter_2, %loop4_done ], [ 10, %loop2_body ]
  %loop3_cond_0 = icmp slt i64 %loop3_counter_1, 25
  br i1 %loop3_cond_0, label %loop3_body, label %loop3_done

loop3_body:                                       ; preds = %loop3_cond
  br label %loop4_cond

loop4_cond:                                       ; preds = %loop5_done, %loop3_body
  %loop4_counter_1 = phi i64 [ %loop4_counter_2, %loop5_done ], [ 10, %loop3_body ]
  %loop4_cond_0 = icmp slt i64 %loop4_counter_1, 25
  br i1 %loop4_cond_0, label %loop4_body, label %loop4_done

loop4_body:                                       ; preds = %loop4_cond
  br label %loop5_cond

loop5_cond:                                       ; preds = %loop5_body, %loop4_body
  %loop5_counter_1 = phi i64 [ %loop5_counter_2, %loop5_body ], [ 10, %loop4_body ]
  %loop5_cond_0 = icmp slt i64 %loop5_counter_1, 25
  br i1 %loop5_cond_0, label %loop5_body, label %loop5_done

loop5_body:                                       ; preds = %loop5_cond
  call void @__orig_main(i64 %loop_counter_1, i64 %loop2_counter_1, i64 %loop3_counter_1, i64 %loop4_counter_1, i64 %loop5_counter_1)
  %loop5_counter_2 = add i64 %loop5_counter_1, 1
  br label %loop5_cond

loop5_done:                                       ; preds = %loop5_cond
  %loop4_counter_2 = add i64 %loop4_counter_1, 1
  br label %loop4_cond

loop4_done:                                       ; preds = %loop4_cond
  %loop3_counter_2 = add i64 %loop3_counter_1, 1
  br label %loop3_cond

loop3_done:                                       ; preds = %loop3_cond
  %loop2_counter_2 = add i64 %loop2_counter_1, 1
  br label %loop2_cond

loop2_done:                                       ; preds = %loop2_cond
  %loop_counter_2 = add i64 %loop_counter_1, 1
  br label %loop_cond

loop_done:                                        ; preds = %loop_cond
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
