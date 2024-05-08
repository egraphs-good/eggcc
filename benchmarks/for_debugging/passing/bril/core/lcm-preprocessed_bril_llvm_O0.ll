; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpdcN1sJ/postprocessed.ll'
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
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 250
  br i1 %loop_cond_0, label %loop_body, label %loop_done

loop_body:                                        ; preds = %loop_cond
  br label %inner_cond

inner_cond:                                       ; preds = %inner_body, %loop_body
  %inner_counter_1 = phi i64 [ %inner_counter_2, %inner_body ], [ 10, %loop_body ]
  %inner_cond_0 = icmp slt i64 %inner_counter_1, 250
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

define dso_local void @__orig_main(i64 %x, i64 %y) {
pre_entry:
  %v4_0 = icmp sgt i64 %x, %y
  br i1 %v4_0, label %then.1, label %else.1.preheader

then.1:                                           ; preds = %pre_entry
  br label %else.1.preheader

else.1.preheader:                                 ; preds = %pre_entry, %then.1
  %greater_2.ph = phi i64 [ %y, %pre_entry ], [ %x, %then.1 ]
  br label %else.1

else.1:                                           ; preds = %else.1.preheader, %else.2
  %greater_2 = phi i64 [ %greater_3, %else.2 ], [ %greater_2.ph, %else.1.preheader ]
  %modX_0 = call i64 @__getMod(i64 %greater_2, i64 %x)
  %modY_0 = call i64 @__getMod(i64 %greater_2, i64 %y)
  %0 = or i64 %modX_0, %modY_0
  %bothZero_0 = icmp eq i64 %0, 0
  br i1 %bothZero_0, label %then.2, label %else.2

then.2:                                           ; preds = %else.1
  %greater_2.lcssa = phi i64 [ %greater_2, %else.1 ]
  call void @print_int(i64 %greater_2.lcssa)
  call void @print_newline()
  br label %loopend

else.2:                                           ; preds = %else.1
  %greater_3 = add i64 %greater_2, 1
  br label %else.1

loopend:                                          ; preds = %then.2
  ret void
}

define dso_local i64 @__getMod(i64 %val, i64 %mod) {
pre_entry:
  %val.fr = freeze i64 %val
  %0 = srem i64 %val.fr, %mod
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
