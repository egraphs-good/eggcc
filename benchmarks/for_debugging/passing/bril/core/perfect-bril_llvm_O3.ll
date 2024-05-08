; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpvPg8yK/perfect-init.ll'
source_filename = "stdin"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"
target triple = "arm64-apple-macosx13.0.0"

@.str = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.str.1 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@.str.2 = private unnamed_addr constant [4 x i8] c"%ld\00", align 1
@.str.3 = private unnamed_addr constant [9 x i8] c"[object]\00", align 1
@.str.4 = private unnamed_addr constant [33 x i8] c"error: expected %d args, got %d\0A\00", align 1

; Function Attrs: nofree nounwind
declare dso_local noundef i32 @putchar(i32 noundef) local_unnamed_addr #0

; Function Attrs: nofree nounwind
declare dso_local noundef i32 @printf(ptr nocapture noundef readonly, ...) local_unnamed_addr #0

declare dso_local void @exit(i32) local_unnamed_addr

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read)
define dso_local i32 @btoi(ptr nocapture readonly %0) local_unnamed_addr #1 {
  %2 = load i8, ptr %0, align 1
  %3 = icmp eq i8 %2, 116
  %4 = zext i1 %3 to i32
  ret i32 %4
}

; Function Attrs: nofree nounwind
define dso_local void @print_bool(i1 %0) local_unnamed_addr #0 {
  %.str..str.1 = select i1 %0, ptr @.str, ptr @.str.1
  %2 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) %.str..str.1)
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @print_space() local_unnamed_addr #0 {
  %1 = tail call i32 @putchar(i32 32)
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @print_newline() local_unnamed_addr #0 {
  %1 = tail call i32 @putchar(i32 10)
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @print_int(i64 %0) local_unnamed_addr #0 {
  %2 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %0)
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @print_ptr(ptr nocapture readnone %0) local_unnamed_addr #0 {
  %2 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.3)
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__main() local_unnamed_addr #0 {
b0:
  br label %for.body.i.preheader

for.body.i.preheader:                             ; preds = %__orig_main.exit, %b0
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %__orig_main.exit ]
  br label %for.body.i

for.body.i:                                       ; preds = %for.body.i.preheader, %for.body.i
  %sum_13.i = phi i64 [ %sum_4.i, %for.body.i ], [ 1, %for.body.i.preheader ]
  %i_12.i = phi i64 [ %i_2.i, %for.body.i ], [ 2, %for.body.i.preheader ]
  %qut_0.i = sdiv i64 %loop_counter_11, %i_12.i
  %mpt_0.i = mul i64 %qut_0.i, %i_12.i
  %comp1_0.i = icmp eq i64 %mpt_0.i, %loop_counter_11
  %sum_2.i = add i64 %i_12.i, %sum_13.i
  %sum_3.i = add i64 %sum_2.i, %qut_0.i
  %sum_4.i = select i1 %comp1_0.i, i64 %sum_3.i, i64 %sum_13.i
  %i_2.i = add i64 %i_12.i, 1
  %ii_0.i = mul i64 %i_2.i, %i_2.i
  %comp_0.i = icmp sgt i64 %ii_0.i, %loop_counter_11
  br i1 %comp_0.i, label %__orig_main.exit, label %for.body.i

__orig_main.exit:                                 ; preds = %for.body.i
  %comp2_0.i = icmp ne i64 %sum_4.i, %loop_counter_11
  %spec.select.i = zext i1 %comp2_0.i to i64
  %0 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %spec.select.i)
  %1 = tail call i32 @putchar(i32 10)
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 1000000
  br i1 %exitcond.not, label %loop_done, label %for.body.i.preheader

loop_done:                                        ; preds = %__orig_main.exit
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %input) local_unnamed_addr #0 {
pre_entry:
  %comp_01 = icmp slt i64 %input, 4
  br i1 %comp_01, label %for.end, label %for.body

for.body:                                         ; preds = %pre_entry, %for.body
  %sum_13 = phi i64 [ %sum_4, %for.body ], [ 1, %pre_entry ]
  %i_12 = phi i64 [ %i_2, %for.body ], [ 2, %pre_entry ]
  %qut_0 = sdiv i64 %input, %i_12
  %mpt_0 = mul i64 %qut_0, %i_12
  %comp1_0 = icmp eq i64 %mpt_0, %input
  %sum_2 = add i64 %sum_13, %i_12
  %sum_3 = add i64 %sum_2, %qut_0
  %sum_4 = select i1 %comp1_0, i64 %sum_3, i64 %sum_13
  %i_2 = add i64 %i_12, 1
  %ii_0 = mul i64 %i_2, %i_2
  %comp_0 = icmp sgt i64 %ii_0, %input
  br i1 %comp_0, label %for.end, label %for.body

for.end:                                          ; preds = %for.body, %pre_entry
  %comp2_0 = icmp ne i64 %sum_4, %input
  %spec.select = zext i1 %comp2_0 to i64
  %0 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %spec.select)
  %1 = tail call i32 @putchar(i32 10)
  ret void
}

define dso_local noundef i32 @main(i32 %argc, ptr nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %for.body.i.preheader.i, label %2

2:                                                ; preds = %0
  %3 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.4, i32 0, i32 %1)
  tail call void @exit(i32 2)
  unreachable

for.body.i.preheader.i:                           ; preds = %0, %__orig_main.exit.i
  %loop_counter_11.i = phi i64 [ %loop_counter_2.i, %__orig_main.exit.i ], [ 10, %0 ]
  br label %for.body.i.i

for.body.i.i:                                     ; preds = %for.body.i.i, %for.body.i.preheader.i
  %sum_13.i.i = phi i64 [ %sum_4.i.i, %for.body.i.i ], [ 1, %for.body.i.preheader.i ]
  %i_12.i.i = phi i64 [ %i_2.i.i, %for.body.i.i ], [ 2, %for.body.i.preheader.i ]
  %qut_0.i.i = sdiv i64 %loop_counter_11.i, %i_12.i.i
  %mpt_0.i.i = mul i64 %qut_0.i.i, %i_12.i.i
  %comp1_0.i.i = icmp eq i64 %mpt_0.i.i, %loop_counter_11.i
  %sum_2.i.i = add i64 %i_12.i.i, %sum_13.i.i
  %sum_3.i.i = add i64 %sum_2.i.i, %qut_0.i.i
  %sum_4.i.i = select i1 %comp1_0.i.i, i64 %sum_3.i.i, i64 %sum_13.i.i
  %i_2.i.i = add i64 %i_12.i.i, 1
  %ii_0.i.i = mul i64 %i_2.i.i, %i_2.i.i
  %comp_0.i.i = icmp sgt i64 %ii_0.i.i, %loop_counter_11.i
  br i1 %comp_0.i.i, label %__orig_main.exit.i, label %for.body.i.i

__orig_main.exit.i:                               ; preds = %for.body.i.i
  %comp2_0.i.i = icmp ne i64 %sum_4.i.i, %loop_counter_11.i
  %spec.select.i.i = zext i1 %comp2_0.i.i to i64
  %4 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %spec.select.i.i)
  %5 = tail call i32 @putchar(i32 10)
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 1000000
  br i1 %exitcond.not.i, label %__main.exit, label %for.body.i.preheader.i

__main.exit:                                      ; preds = %__orig_main.exit.i
  ret i32 0
}

attributes #0 = { nofree nounwind }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read) }
