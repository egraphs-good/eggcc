; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpmks8qS/postprocessed.ll'
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
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 50000
  br i1 %loop_cond_0, label %loop_body, label %loop_done

loop_body:                                        ; preds = %loop_cond
  call void @__orig_main(i64 %loop_counter_1)
  %loop_counter_2 = add i64 %loop_counter_1, 1
  br label %loop_cond

loop_done:                                        ; preds = %loop_cond
  ret void
}

define dso_local void @__orig_main(i64 %size) {
pre_entry:
  %z0 = shl i64 %size, 3
  %z1 = call ptr @malloc(i64 %z0)
  call void @__fill_array(ptr %z1, i64 %size)
  %checksum_0 = call i64 @__adler32(ptr %z1, i64 %size)
  call void @print_int(i64 %checksum_0)
  call void @print_newline()
  call void @free(ptr %z1)
  ret void
}

define dso_local i64 @__mod(i64 %r, i64 %s) {
pre_entry:
  %r.fr = freeze i64 %r
  %0 = srem i64 %r.fr, %s
  ret i64 %0
}

define dso_local void @__fill_array(ptr %arr, i64 %size) {
pre_entry:
  br label %loop

loop:                                             ; preds = %loop, %pre_entry
  %loc_1 = phi ptr [ %loc_2, %loop ], [ %arr, %pre_entry ]
  %curr_1 = phi i64 [ %curr_2, %loop ], [ 0, %pre_entry ]
  store i64 %curr_1, ptr %loc_1, align 8
  %loc_2 = getelementptr inbounds i64, ptr %loc_1, i64 1
  %curr_2 = add i64 %curr_1, 1
  %continue_0 = icmp slt i64 %curr_2, %size
  br i1 %continue_0, label %loop, label %exit

exit:                                             ; preds = %loop
  ret void
}

define dso_local i64 @__bitwise_or(i64 %x, i64 %y) {
pre_entry:
  br label %loop

loop:                                             ; preds = %false, %pre_entry
  %result_1 = phi i64 [ %result_3, %false ], [ 0, %pre_entry ]
  %val_1 = phi i64 [ %val_2, %false ], [ 1, %pre_entry ]
  %y_1 = phi i64 [ %y_2, %false ], [ %y, %pre_entry ]
  %x_1 = phi i64 [ %x_2, %false ], [ %x, %pre_entry ]
  %xmod2_0 = call i64 @__mod(i64 %x_1, i64 2)
  %ymod2_0 = call i64 @__mod(i64 %y_1, i64 2)
  %xodd_0 = icmp eq i64 %xmod2_0, 1
  %yodd_0 = icmp eq i64 %ymod2_0, 1
  %cond_0 = or i1 %xodd_0, %yodd_0
  br i1 %cond_0, label %true, label %false

true:                                             ; preds = %loop
  %result_2 = add i64 %result_1, %val_1
  br label %false

false:                                            ; preds = %true, %loop
  %result_3 = phi i64 [ %result_2, %true ], [ %result_1, %loop ]
  %x_2 = sdiv i64 %x_1, 2
  %y_2 = sdiv i64 %y_1, 2
  %xpos_0 = icmp sgt i64 %x_1, 1
  %ypos_0 = icmp sgt i64 %y_1, 1
  %val_2 = shl i64 %val_1, 1
  %continue_0 = or i1 %xpos_0, %ypos_0
  br i1 %continue_0, label %loop, label %exit

exit:                                             ; preds = %false
  %result_3.lcssa = phi i64 [ %result_3, %false ]
  ret i64 %result_3.lcssa
}

define dso_local i64 @__adler32(ptr %arr, i64 %size) {
pre_entry:
  br label %loop

loop:                                             ; preds = %loop, %pre_entry
  %loc_1 = phi ptr [ %loc_2, %loop ], [ %arr, %pre_entry ]
  %curr_1 = phi i64 [ %curr_2, %loop ], [ 0, %pre_entry ]
  %b_1 = phi i64 [ %b_2, %loop ], [ 0, %pre_entry ]
  %a_1 = phi i64 [ %a_2, %loop ], [ 1, %pre_entry ]
  %val_0 = load i64, ptr %loc_1, align 8
  %a_2 = add i64 %a_1, %val_0
  %b_2 = add i64 %b_1, %a_2
  %loc_2 = getelementptr inbounds i64, ptr %loc_1, i64 1
  %curr_2 = add i64 %curr_1, 1
  %continue_0 = icmp slt i64 %curr_2, %size
  br i1 %continue_0, label %loop, label %exit

exit:                                             ; preds = %loop
  %a_2.lcssa = phi i64 [ %a_2, %loop ]
  %b_2.lcssa = phi i64 [ %b_2, %loop ]
  %a_3 = call i64 @__mod(i64 %a_2.lcssa, i64 65521)
  %b_3 = call i64 @__mod(i64 %b_2.lcssa, i64 65521)
  %b_4 = shl i64 %b_3, 16
  %result_0 = call i64 @__bitwise_or(i64 %b_4, i64 %a_3)
  ret i64 %result_0
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
