; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpZBHOCE/sum-check-init.ll'
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
  br label %for.i.i.preheader

for.i.i.preheader:                                ; preds = %b0, %for.i.i.preheader
  %indvars.iv5 = phi i65 [ 72, %b0 ], [ %indvars.iv.next6, %for.i.i.preheader ]
  %indvars.iv3 = phi i65 [ 18, %b0 ], [ %indvars.iv.next4, %for.i.i.preheader ]
  %indvars.iv = phi i64 [ 19, %b0 ], [ %indvars.iv.next, %for.i.i.preheader ]
  %loop_counter_12 = phi i64 [ 10, %b0 ], [ %n_1_0.i.i, %for.i.i.preheader ]
  %0 = lshr exact i65 %indvars.iv5, 1
  %1 = trunc i65 %0 to i64
  %2 = add i64 %indvars.iv, %1
  %n_1_0.i.i = add nuw nsw i64 %loop_counter_12, 1
  %multi_0.i.i = mul nuw nsw i64 %n_1_0.i.i, %loop_counter_12
  %sum_0.i.i1 = lshr i64 %multi_0.i.i, 1
  %isSame_0.i = icmp eq i64 %2, %sum_0.i.i1
  %3 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %2)
  %4 = tail call i32 @putchar(i32 10)
  %5 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %sum_0.i.i1)
  %6 = tail call i32 @putchar(i32 10)
  %.str..str.1.i.i = select i1 %isSame_0.i, ptr @.str, ptr @.str.1
  %7 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) %.str..str.1.i.i)
  %8 = tail call i32 @putchar(i32 10)
  %indvars.iv.next = add nuw nsw i64 %indvars.iv, 2
  %indvars.iv.next4 = add nuw nsw i65 %indvars.iv3, 2
  %indvars.iv.next6 = add i65 %indvars.iv5, %indvars.iv3
  %exitcond.not = icmp eq i64 %n_1_0.i.i, 50000
  br i1 %exitcond.not, label %loop_done, label %for.i.i.preheader

loop_done:                                        ; preds = %for.i.i.preheader
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %n) local_unnamed_addr #0 {
pre_entry:
  %con_0.not1.i = icmp slt i64 %n, 1
  br i1 %con_0.not1.i, label %__sum_by_loop.exit, label %for.i

for.i:                                            ; preds = %pre_entry, %for.i
  %sum_13.i = phi i64 [ %sum_2.i, %for.i ], [ 0, %pre_entry ]
  %i_12.i = phi i64 [ %i_2.i, %for.i ], [ 1, %pre_entry ]
  %sum_2.i = add i64 %i_12.i, %sum_13.i
  %i_2.i = add i64 %i_12.i, 1
  %con_0.not.i = icmp sgt i64 %i_2.i, %n
  br i1 %con_0.not.i, label %__sum_by_loop.exit, label %for.i

__sum_by_loop.exit:                               ; preds = %for.i, %pre_entry
  %sum_1.lcssa.i = phi i64 [ 0, %pre_entry ], [ %sum_2.i, %for.i ]
  %n_1_0.i = add nsw i64 %n, 1
  %multi_0.i = mul i64 %n_1_0.i, %n
  %sum_0.i = sdiv i64 %multi_0.i, 2
  %isSame_0 = icmp eq i64 %sum_1.lcssa.i, %sum_0.i
  %0 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %sum_1.lcssa.i)
  %1 = tail call i32 @putchar(i32 10)
  %2 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %sum_0.i)
  %3 = tail call i32 @putchar(i32 10)
  %.str..str.1.i = select i1 %isSame_0, ptr @.str, ptr @.str.1
  %4 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) %.str..str.1.i)
  %5 = tail call i32 @putchar(i32 10)
  ret void
}

; Function Attrs: nofree norecurse nosync nounwind memory(none)
define dso_local i64 @__sum_by_loop(i64 %n) local_unnamed_addr #2 {
pre_entry:
  %con_0.not1 = icmp slt i64 %n, 1
  br i1 %con_0.not1, label %end, label %for

for:                                              ; preds = %pre_entry, %for
  %sum_13 = phi i64 [ %sum_2, %for ], [ 0, %pre_entry ]
  %i_12 = phi i64 [ %i_2, %for ], [ 1, %pre_entry ]
  %sum_2 = add i64 %sum_13, %i_12
  %i_2 = add i64 %i_12, 1
  %con_0.not = icmp sgt i64 %i_2, %n
  br i1 %con_0.not, label %end, label %for

end:                                              ; preds = %for, %pre_entry
  %sum_1.lcssa = phi i64 [ 0, %pre_entry ], [ %sum_2, %for ]
  ret i64 %sum_1.lcssa
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define dso_local i64 @__sum_by_formula(i64 %n) local_unnamed_addr #3 {
pre_entry:
  %n_1_0 = add i64 %n, 1
  %multi_0 = mul i64 %n_1_0, %n
  %sum_0 = sdiv i64 %multi_0, 2
  ret i64 %sum_0
}

define dso_local noundef i32 @main(i32 %argc, ptr nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %4, label %2

2:                                                ; preds = %0
  %3 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.4, i32 0, i32 %1)
  tail call void @exit(i32 2)
  unreachable

4:                                                ; preds = %0
  tail call void @__main()
  ret i32 0
}

attributes #0 = { nofree nounwind }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read) }
attributes #2 = { nofree norecurse nosync nounwind memory(none) }
attributes #3 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) }
