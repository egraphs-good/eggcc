; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpKCkiMq/postprocessed.ll'
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

define dso_local i64 @__dot_product(ptr %vectorA, ptr %vectorB, i64 %size) {
pre_entry:
  br label %loop

loop:                                             ; preds = %loop, %pre_entry
  %answer_1 = phi i64 [ %answer_2, %loop ], [ 0, %pre_entry ]
  %index_1 = phi i64 [ %index_2, %loop ], [ 0, %pre_entry ]
  %ptrA_0 = getelementptr inbounds i64, ptr %vectorA, i64 %index_1
  %ptrB_0 = getelementptr inbounds i64, ptr %vectorB, i64 %index_1
  %valA_0 = load i64, ptr %ptrA_0, align 8
  %valB_0 = load i64, ptr %ptrB_0, align 8
  %tmp_0 = mul i64 %valA_0, %valB_0
  %answer_2 = add i64 %answer_1, %tmp_0
  %index_2 = add i64 %index_1, 1
  %cond_0 = icmp slt i64 %index_2, %size
  br i1 %cond_0, label %loop, label %done

done:                                             ; preds = %loop
  %answer_2.lcssa = phi i64 [ %answer_2, %loop ]
  ret i64 %answer_2.lcssa
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

define dso_local void @__orig_main(i64 %x) {
pre_entry:
  %z1 = call ptr @malloc(i64 40)
  store i64 25, ptr %z1, align 8
  %indexPtr_1 = getelementptr inbounds i64, ptr %z1, i64 1
  store i64 50, ptr %indexPtr_1, align 8
  %indexPtr_2 = getelementptr inbounds i64, ptr %z1, i64 2
  store i64 100, ptr %indexPtr_2, align 8
  %indexPtr_3 = getelementptr inbounds i64, ptr %z1, i64 3
  store i64 150, ptr %indexPtr_3, align 8
  %indexPtr_4 = getelementptr inbounds i64, ptr %z1, i64 4
  store i64 250, ptr %indexPtr_4, align 8
  %z3 = call ptr @malloc(i64 40)
  store i64 2, ptr %z3, align 8
  %indexPtr_6 = getelementptr inbounds i64, ptr %z3, i64 1
  store i64 10, ptr %indexPtr_6, align 8
  %indexPtr_7 = getelementptr inbounds i64, ptr %z3, i64 2
  store i64 20, ptr %indexPtr_7, align 8
  %indexPtr_8 = getelementptr inbounds i64, ptr %z3, i64 3
  store i64 30, ptr %indexPtr_8, align 8
  %indexPtr_9 = getelementptr inbounds i64, ptr %z3, i64 4
  store i64 40, ptr %indexPtr_9, align 8
  %val_0 = call i64 @__dot_product(ptr nonnull %z1, ptr nonnull %z3, i64 5)
  %val_1 = add i64 %val_0, %x
  call void @print_int(i64 %val_1)
  call void @print_newline()
  call void @free(ptr nonnull %z1)
  call void @free(ptr nonnull %z3)
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
