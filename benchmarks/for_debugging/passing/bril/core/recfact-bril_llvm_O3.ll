; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpWndA1s/recfact-init.ll'
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
  br label %else.0.i.i.preheader

else.0.i.i.preheader:                             ; preds = %__orig_main.exit, %b0
  %indvar = phi i64 [ %indvar.next, %__orig_main.exit ], [ 0, %b0 ]
  %loop_counter_11 = phi i64 [ %loop_counter_2, %__orig_main.exit ], [ 10, %b0 ]
  %0 = add i64 %indvar, 9
  %min.iters.check = icmp ult i64 %0, 2
  br i1 %min.iters.check, label %else.0.i.i.preheader3, label %vector.ph

vector.ph:                                        ; preds = %else.0.i.i.preheader
  %n.vec = and i64 %0, -2
  %ind.end = sub i64 %loop_counter_11, %n.vec
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 1, %vector.ph ], [ %2, %vector.body ]
  %vec.phi2 = phi i64 [ 1, %vector.ph ], [ %3, %vector.body ]
  %offset.idx = sub i64 %loop_counter_11, %index
  %1 = add i64 %offset.idx, -1
  %2 = mul i64 %vec.phi, %offset.idx
  %3 = mul i64 %vec.phi2, %1
  %index.next = add nuw i64 %index, 2
  %4 = icmp eq i64 %index.next, %n.vec
  br i1 %4, label %middle.block, label %vector.body, !llvm.loop !0

middle.block:                                     ; preds = %vector.body
  %bin.rdx = mul i64 %3, %2
  %cmp.n = icmp eq i64 %0, %n.vec
  br i1 %cmp.n, label %__orig_main.exit, label %else.0.i.i.preheader3

else.0.i.i.preheader3:                            ; preds = %else.0.i.i.preheader, %middle.block
  %x.tr3.i.i.ph = phi i64 [ %loop_counter_11, %else.0.i.i.preheader ], [ %ind.end, %middle.block ]
  %accumulator.tr2.i.i.ph = phi i64 [ 1, %else.0.i.i.preheader ], [ %bin.rdx, %middle.block ]
  br label %else.0.i.i

else.0.i.i:                                       ; preds = %else.0.i.i.preheader3, %else.0.i.i
  %x.tr3.i.i = phi i64 [ %v8_0.i.i, %else.0.i.i ], [ %x.tr3.i.i.ph, %else.0.i.i.preheader3 ]
  %accumulator.tr2.i.i = phi i64 [ %v10_0.i.i, %else.0.i.i ], [ %accumulator.tr2.i.i.ph, %else.0.i.i.preheader3 ]
  %v8_0.i.i = add nsw i64 %x.tr3.i.i, -1
  %v10_0.i.i = mul i64 %accumulator.tr2.i.i, %x.tr3.i.i
  %v3_0.i.i = icmp ult i64 %x.tr3.i.i, 3
  br i1 %v3_0.i.i, label %__orig_main.exit, label %else.0.i.i, !llvm.loop !3

__orig_main.exit:                                 ; preds = %else.0.i.i, %middle.block
  %v10_0.i.i.lcssa = phi i64 [ %bin.rdx, %middle.block ], [ %v10_0.i.i, %else.0.i.i ]
  %5 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %v10_0.i.i.lcssa)
  %6 = tail call i32 @putchar(i32 10)
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 20000
  %indvar.next = add i64 %indvar, 1
  br i1 %exitcond.not, label %loop_done, label %else.0.i.i.preheader

loop_done:                                        ; preds = %__orig_main.exit
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %input) local_unnamed_addr #0 {
pre_entry:
  %v3_01.i = icmp slt i64 %input, 2
  br i1 %v3_01.i, label %__fac.exit, label %else.0.i.preheader

else.0.i.preheader:                               ; preds = %pre_entry
  %0 = add nsw i64 %input, -1
  %min.iters.check = icmp ult i64 %input, 5
  br i1 %min.iters.check, label %else.0.i.preheader6, label %vector.ph

vector.ph:                                        ; preds = %else.0.i.preheader
  %n.vec = and i64 %0, -4
  %ind.end = sub i64 %input, %n.vec
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 1, %vector.ph ], [ %4, %vector.body ]
  %vec.phi1 = phi i64 [ 1, %vector.ph ], [ %5, %vector.body ]
  %vec.phi2 = phi i64 [ 1, %vector.ph ], [ %6, %vector.body ]
  %vec.phi3 = phi i64 [ 1, %vector.ph ], [ %7, %vector.body ]
  %offset.idx = sub i64 %input, %index
  %1 = add i64 %offset.idx, -1
  %2 = add i64 %offset.idx, -2
  %3 = add i64 %offset.idx, -3
  %4 = mul i64 %vec.phi, %offset.idx
  %5 = mul i64 %vec.phi1, %1
  %6 = mul i64 %vec.phi2, %2
  %7 = mul i64 %vec.phi3, %3
  %index.next = add nuw i64 %index, 4
  %8 = icmp eq i64 %index.next, %n.vec
  br i1 %8, label %middle.block, label %vector.body, !llvm.loop !4

middle.block:                                     ; preds = %vector.body
  %bin.rdx = mul i64 %5, %4
  %bin.rdx4 = mul i64 %6, %bin.rdx
  %bin.rdx5 = mul i64 %7, %bin.rdx4
  %cmp.n = icmp eq i64 %0, %n.vec
  br i1 %cmp.n, label %__fac.exit, label %else.0.i.preheader6

else.0.i.preheader6:                              ; preds = %else.0.i.preheader, %middle.block
  %x.tr3.i.ph = phi i64 [ %input, %else.0.i.preheader ], [ %ind.end, %middle.block ]
  %accumulator.tr2.i.ph = phi i64 [ 1, %else.0.i.preheader ], [ %bin.rdx5, %middle.block ]
  br label %else.0.i

else.0.i:                                         ; preds = %else.0.i.preheader6, %else.0.i
  %x.tr3.i = phi i64 [ %v8_0.i, %else.0.i ], [ %x.tr3.i.ph, %else.0.i.preheader6 ]
  %accumulator.tr2.i = phi i64 [ %v10_0.i, %else.0.i ], [ %accumulator.tr2.i.ph, %else.0.i.preheader6 ]
  %v8_0.i = add nsw i64 %x.tr3.i, -1
  %v10_0.i = mul i64 %accumulator.tr2.i, %x.tr3.i
  %v3_0.i = icmp ult i64 %x.tr3.i, 3
  br i1 %v3_0.i, label %__fac.exit, label %else.0.i, !llvm.loop !5

__fac.exit:                                       ; preds = %else.0.i, %middle.block, %pre_entry
  %accumulator.tr.lcssa.i = phi i64 [ 1, %pre_entry ], [ %bin.rdx5, %middle.block ], [ %v10_0.i, %else.0.i ]
  %9 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %accumulator.tr.lcssa.i)
  %10 = tail call i32 @putchar(i32 10)
  ret void
}

; Function Attrs: nofree norecurse nosync nounwind memory(none)
define dso_local i64 @__fac(i64 %x) local_unnamed_addr #2 {
pre_entry:
  %v3_01 = icmp slt i64 %x, 2
  br i1 %v3_01, label %common.ret, label %else.0.preheader

else.0.preheader:                                 ; preds = %pre_entry
  %0 = add nsw i64 %x, -1
  %min.iters.check = icmp ult i64 %x, 5
  br i1 %min.iters.check, label %else.0.preheader9, label %vector.ph

vector.ph:                                        ; preds = %else.0.preheader
  %n.vec = and i64 %0, -4
  %ind.end = sub i64 %x, %n.vec
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 1, %vector.ph ], [ %4, %vector.body ]
  %vec.phi4 = phi i64 [ 1, %vector.ph ], [ %5, %vector.body ]
  %vec.phi5 = phi i64 [ 1, %vector.ph ], [ %6, %vector.body ]
  %vec.phi6 = phi i64 [ 1, %vector.ph ], [ %7, %vector.body ]
  %offset.idx = sub i64 %x, %index
  %1 = add i64 %offset.idx, -1
  %2 = add i64 %offset.idx, -2
  %3 = add i64 %offset.idx, -3
  %4 = mul i64 %offset.idx, %vec.phi
  %5 = mul i64 %1, %vec.phi4
  %6 = mul i64 %2, %vec.phi5
  %7 = mul i64 %3, %vec.phi6
  %index.next = add nuw i64 %index, 4
  %8 = icmp eq i64 %index.next, %n.vec
  br i1 %8, label %middle.block, label %vector.body, !llvm.loop !6

middle.block:                                     ; preds = %vector.body
  %bin.rdx = mul i64 %5, %4
  %bin.rdx7 = mul i64 %6, %bin.rdx
  %bin.rdx8 = mul i64 %7, %bin.rdx7
  %cmp.n = icmp eq i64 %0, %n.vec
  br i1 %cmp.n, label %common.ret, label %else.0.preheader9

else.0.preheader9:                                ; preds = %else.0.preheader, %middle.block
  %x.tr3.ph = phi i64 [ %x, %else.0.preheader ], [ %ind.end, %middle.block ]
  %accumulator.tr2.ph = phi i64 [ 1, %else.0.preheader ], [ %bin.rdx8, %middle.block ]
  br label %else.0

common.ret:                                       ; preds = %else.0, %middle.block, %pre_entry
  %accumulator.tr.lcssa = phi i64 [ 1, %pre_entry ], [ %bin.rdx8, %middle.block ], [ %v10_0, %else.0 ]
  ret i64 %accumulator.tr.lcssa

else.0:                                           ; preds = %else.0.preheader9, %else.0
  %x.tr3 = phi i64 [ %v8_0, %else.0 ], [ %x.tr3.ph, %else.0.preheader9 ]
  %accumulator.tr2 = phi i64 [ %v10_0, %else.0 ], [ %accumulator.tr2.ph, %else.0.preheader9 ]
  %v8_0 = add nsw i64 %x.tr3, -1
  %v10_0 = mul i64 %x.tr3, %accumulator.tr2
  %v3_0 = icmp ult i64 %x.tr3, 3
  br i1 %v3_0, label %common.ret, label %else.0, !llvm.loop !7
}

define dso_local noundef i32 @main(i32 %argc, ptr nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %else.0.i.i.preheader.i, label %2

2:                                                ; preds = %0
  %3 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.4, i32 0, i32 %1)
  tail call void @exit(i32 2)
  unreachable

else.0.i.i.preheader.i:                           ; preds = %0, %__orig_main.exit.i
  %indvar = phi i64 [ %indvar.next, %__orig_main.exit.i ], [ 0, %0 ]
  %loop_counter_11.i = phi i64 [ %loop_counter_2.i, %__orig_main.exit.i ], [ 10, %0 ]
  %4 = add i64 %indvar, 9
  %min.iters.check = icmp ult i64 %4, 2
  br i1 %min.iters.check, label %else.0.i.i.i.preheader, label %vector.ph

vector.ph:                                        ; preds = %else.0.i.i.preheader.i
  %n.vec = and i64 %4, -2
  %ind.end = sub i64 %loop_counter_11.i, %n.vec
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 1, %vector.ph ], [ %6, %vector.body ]
  %vec.phi2 = phi i64 [ 1, %vector.ph ], [ %7, %vector.body ]
  %offset.idx = sub i64 %loop_counter_11.i, %index
  %5 = add i64 %offset.idx, -1
  %6 = mul i64 %vec.phi, %offset.idx
  %7 = mul i64 %vec.phi2, %5
  %index.next = add nuw i64 %index, 2
  %8 = icmp eq i64 %index.next, %n.vec
  br i1 %8, label %middle.block, label %vector.body, !llvm.loop !8

middle.block:                                     ; preds = %vector.body
  %bin.rdx = mul i64 %7, %6
  %cmp.n = icmp eq i64 %4, %n.vec
  br i1 %cmp.n, label %__orig_main.exit.i, label %else.0.i.i.i.preheader

else.0.i.i.i.preheader:                           ; preds = %else.0.i.i.preheader.i, %middle.block
  %x.tr3.i.i.i.ph = phi i64 [ %loop_counter_11.i, %else.0.i.i.preheader.i ], [ %ind.end, %middle.block ]
  %accumulator.tr2.i.i.i.ph = phi i64 [ 1, %else.0.i.i.preheader.i ], [ %bin.rdx, %middle.block ]
  br label %else.0.i.i.i

else.0.i.i.i:                                     ; preds = %else.0.i.i.i.preheader, %else.0.i.i.i
  %x.tr3.i.i.i = phi i64 [ %v8_0.i.i.i, %else.0.i.i.i ], [ %x.tr3.i.i.i.ph, %else.0.i.i.i.preheader ]
  %accumulator.tr2.i.i.i = phi i64 [ %v10_0.i.i.i, %else.0.i.i.i ], [ %accumulator.tr2.i.i.i.ph, %else.0.i.i.i.preheader ]
  %v8_0.i.i.i = add nsw i64 %x.tr3.i.i.i, -1
  %v10_0.i.i.i = mul i64 %accumulator.tr2.i.i.i, %x.tr3.i.i.i
  %v3_0.i.i.i = icmp ult i64 %x.tr3.i.i.i, 3
  br i1 %v3_0.i.i.i, label %__orig_main.exit.i, label %else.0.i.i.i, !llvm.loop !9

__orig_main.exit.i:                               ; preds = %else.0.i.i.i, %middle.block
  %v10_0.i.i.i.lcssa = phi i64 [ %bin.rdx, %middle.block ], [ %v10_0.i.i.i, %else.0.i.i.i ]
  %9 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %v10_0.i.i.i.lcssa)
  %10 = tail call i32 @putchar(i32 10)
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 20000
  %indvar.next = add i64 %indvar, 1
  br i1 %exitcond.not.i, label %__main.exit, label %else.0.i.i.preheader.i

__main.exit:                                      ; preds = %__orig_main.exit.i
  ret i32 0
}

attributes #0 = { nofree nounwind }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read) }
attributes #2 = { nofree norecurse nosync nounwind memory(none) }

!0 = distinct !{!0, !1, !2}
!1 = !{!"llvm.loop.isvectorized", i32 1}
!2 = !{!"llvm.loop.unroll.runtime.disable"}
!3 = distinct !{!3, !1}
!4 = distinct !{!4, !1, !2}
!5 = distinct !{!5, !1}
!6 = distinct !{!6, !1, !2}
!7 = distinct !{!7, !1}
!8 = distinct !{!8, !1, !2}
!9 = distinct !{!9, !1}
