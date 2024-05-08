; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpCs57iN/postprocessed.ll'
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
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 400
  br i1 %loop_cond_0, label %loop_body, label %loop_done

loop_body:                                        ; preds = %loop_cond
  br label %inner_cond

inner_cond:                                       ; preds = %inner_body, %loop_body
  %inner_counter_1 = phi i64 [ %inner_counter_2, %inner_body ], [ 10, %loop_body ]
  %inner_cond_0 = icmp slt i64 %inner_counter_1, 400
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

define dso_local void @__orig_main(i64 %a, i64 %b) {
pre_entry:
  br label %for.outer.init

for.outer.init:                                   ; preds = %pre_entry
  %t1_0 = icmp slt i64 %a, 2
  br i1 %t1_0, label %true, label %false

true:                                             ; preds = %for.outer.init
  br label %for.outer.cond.preheader

false:                                            ; preds = %for.outer.init
  br label %for.outer.cond.preheader

for.outer.cond.preheader:                         ; preds = %true, %false
  %t2_2.ph = phi i64 [ 2, %true ], [ %a, %false ]
  br label %for.outer.cond

for.outer.cond:                                   ; preds = %for.outer.cond.preheader, %if.outer.end
  %t2_2 = phi i64 [ %t2_3, %if.outer.end ], [ %t2_2.ph, %for.outer.cond.preheader ]
  %t3_0.not = icmp sgt i64 %t2_2, %b
  br i1 %t3_0.not, label %for.outer.end, label %for.outer.body

for.outer.body:                                   ; preds = %for.outer.cond
  br label %for.inner.init

for.inner.init:                                   ; preds = %for.outer.body
  br label %for.inner.cond

for.inner.cond:                                   ; preds = %if.inner.end, %for.inner.init
  %t6_1 = phi i64 [ %t6_2, %if.inner.end ], [ 2, %for.inner.init ]
  %t8_0 = sdiv i64 %t2_2, 2
  %t9_0.not = icmp sgt i64 %t6_1, %t8_0
  br i1 %t9_0.not, label %for.inner.end.loopexit, label %for.inner.body

for.inner.body:                                   ; preds = %for.inner.cond
  %t10_0 = call i64 @__mod(i64 %t2_2, i64 %t6_1)
  %t12_0 = icmp eq i64 %t10_0, 0
  br i1 %t12_0, label %if.inner.body, label %if.inner.end

if.inner.body:                                    ; preds = %for.inner.body
  %t9_0.not.lcssa1 = phi i1 [ %t9_0.not, %for.inner.body ]
  br label %for.inner.end

if.inner.end:                                     ; preds = %for.inner.body
  %t6_2 = add i64 %t6_1, 1
  br label %for.inner.cond

for.inner.end.loopexit:                           ; preds = %for.inner.cond
  %t9_0.not.lcssa = phi i1 [ %t9_0.not, %for.inner.cond ]
  br label %for.inner.end

for.inner.end:                                    ; preds = %for.inner.end.loopexit, %if.inner.body
  %t9_0.not2 = phi i1 [ %t9_0.not.lcssa, %for.inner.end.loopexit ], [ %t9_0.not.lcssa1, %if.inner.body ]
  br i1 %t9_0.not2, label %if.outer.body, label %if.outer.end

if.outer.body:                                    ; preds = %for.inner.end
  call void @print_int(i64 %t2_2)
  call void @print_newline()
  br label %if.outer.end

if.outer.end:                                     ; preds = %if.outer.body, %for.inner.end
  %t2_3 = add i64 %t2_2, 1
  br label %for.outer.cond

for.outer.end:                                    ; preds = %for.outer.cond
  ret void
}

define dso_local i64 @__mod(i64 %a, i64 %b) {
pre_entry:
  %a.fr = freeze i64 %a
  %0 = srem i64 %a.fr, %b
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
