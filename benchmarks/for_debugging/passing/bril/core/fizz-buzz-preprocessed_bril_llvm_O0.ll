; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmp3Lfqmt/postprocessed.ll'
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

define dso_local void @__orig_main(i64 %input) {
pre_entry:
  br label %for.cond.0

for.cond.0:                                       ; preds = %endif.33, %pre_entry
  %index_1 = phi i64 [ %v43_0, %endif.33 ], [ 1, %pre_entry ]
  %v4_0 = icmp slt i64 %index_1, %input
  br i1 %v4_0, label %for.body.0, label %for.end.0

for.body.0:                                       ; preds = %for.cond.0
  %0 = srem i64 %index_1, 3
  %v12_0 = icmp eq i64 %0, 0
  %1 = srem i64 %index_1, 5
  %v20_0 = icmp eq i64 %1, 0
  br i1 %v12_0, label %then.21, label %else.21

then.21:                                          ; preds = %for.body.0
  br i1 %v20_0, label %then.23, label %else.23

then.23:                                          ; preds = %then.21
  call void @print_int(i64 -1)
  call void @print_newline()
  br label %endif.23

else.23:                                          ; preds = %then.21
  call void @print_int(i64 -2)
  call void @print_newline()
  br label %endif.23

endif.23:                                         ; preds = %else.23, %then.23
  br label %endif.33

else.21:                                          ; preds = %for.body.0
  br i1 %v20_0, label %then.33, label %else.33

then.33:                                          ; preds = %else.21
  call void @print_int(i64 -3)
  call void @print_newline()
  br label %endif.33

else.33:                                          ; preds = %else.21
  call void @print_int(i64 %index_1)
  call void @print_newline()
  br label %endif.33

endif.33:                                         ; preds = %else.33, %then.33, %endif.23
  %v43_0 = add i64 %index_1, 1
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
