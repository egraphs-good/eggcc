; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpP9f5ak/loopfact-init.ll'
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
  br label %for.body.2.i.preheader

for.body.2.i.preheader:                           ; preds = %b0, %__orig_main.exit
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %__orig_main.exit ]
  %min.iters.check = icmp ult i64 %loop_counter_11, 2
  br i1 %min.iters.check, label %for.body.2.i, label %vector.ph

vector.ph:                                        ; preds = %for.body.2.i.preheader
  %n.vec = and i64 %loop_counter_11, 9223372036854775806
  %ind.end = and i64 %loop_counter_11, 1
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 1, %vector.ph ], [ %1, %vector.body ]
  %vec.phi2 = phi i64 [ 1, %vector.ph ], [ %2, %vector.body ]
  %offset.idx = sub i64 %loop_counter_11, %index
  %0 = add i64 %offset.idx, -1
  %1 = mul i64 %offset.idx, %vec.phi
  %2 = mul i64 %0, %vec.phi2
  %index.next = add nuw i64 %index, 2
  %3 = icmp eq i64 %index.next, %n.vec
  br i1 %3, label %middle.block, label %vector.body, !llvm.loop !0

middle.block:                                     ; preds = %vector.body
  %bin.rdx = mul i64 %2, %1
  %cmp.n = icmp eq i64 %loop_counter_11, %n.vec
  br i1 %cmp.n, label %__orig_main.exit, label %for.body.2.i

for.body.2.i:                                     ; preds = %middle.block, %for.body.2.i.preheader
  %result_13.i.ph = phi i64 [ 1, %for.body.2.i.preheader ], [ %bin.rdx, %middle.block ]
  %i_12.i.ph = phi i64 [ %loop_counter_11, %for.body.2.i.preheader ], [ %ind.end, %middle.block ]
  %v9_0.i = mul nuw i64 %i_12.i.ph, %result_13.i.ph
  br label %__orig_main.exit

__orig_main.exit:                                 ; preds = %for.body.2.i, %middle.block
  %v9_0.i.lcssa = phi i64 [ %bin.rdx, %middle.block ], [ %v9_0.i, %for.body.2.i ]
  %4 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %v9_0.i.lcssa)
  %5 = tail call i32 @putchar(i32 10)
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 50000
  br i1 %exitcond.not, label %loop_done, label %for.body.2.i.preheader

loop_done:                                        ; preds = %__orig_main.exit
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %input) local_unnamed_addr #0 {
pre_entry:
  %v6_01 = icmp sgt i64 %input, 0
  br i1 %v6_01, label %for.body.2.preheader, label %for.end.2

for.body.2.preheader:                             ; preds = %pre_entry
  %min.iters.check = icmp ult i64 %input, 4
  br i1 %min.iters.check, label %for.body.2, label %vector.ph

vector.ph:                                        ; preds = %for.body.2.preheader
  %n.vec = and i64 %input, 9223372036854775804
  %ind.end = and i64 %input, 3
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 1, %vector.ph ], [ %3, %vector.body ]
  %vec.phi4 = phi i64 [ 1, %vector.ph ], [ %4, %vector.body ]
  %vec.phi5 = phi i64 [ 1, %vector.ph ], [ %5, %vector.body ]
  %vec.phi6 = phi i64 [ 1, %vector.ph ], [ %6, %vector.body ]
  %offset.idx = sub i64 %input, %index
  %0 = add i64 %offset.idx, -1
  %1 = add i64 %offset.idx, -2
  %2 = add i64 %offset.idx, -3
  %3 = mul i64 %vec.phi, %offset.idx
  %4 = mul i64 %vec.phi4, %0
  %5 = mul i64 %vec.phi5, %1
  %6 = mul i64 %vec.phi6, %2
  %index.next = add nuw i64 %index, 4
  %7 = icmp eq i64 %index.next, %n.vec
  br i1 %7, label %middle.block, label %vector.body, !llvm.loop !3

middle.block:                                     ; preds = %vector.body
  %bin.rdx = mul i64 %4, %3
  %bin.rdx7 = mul i64 %5, %bin.rdx
  %bin.rdx8 = mul i64 %6, %bin.rdx7
  %cmp.n = icmp eq i64 %n.vec, %input
  br i1 %cmp.n, label %for.end.2, label %for.body.2

for.body.2:                                       ; preds = %middle.block, %for.body.2.preheader
  %result_13.ph = phi i64 [ 1, %for.body.2.preheader ], [ %bin.rdx8, %middle.block ]
  %i_12.ph = phi i64 [ %input, %for.body.2.preheader ], [ %ind.end, %middle.block ]
  %v9_0 = mul i64 %result_13.ph, %i_12.ph
  %v6_0 = icmp ugt i64 %i_12.ph, 1
  br i1 %v6_0, label %for.body.2.1, label %for.end.2, !llvm.loop !4

for.body.2.1:                                     ; preds = %for.body.2
  %v12_0 = add nsw i64 %i_12.ph, -1
  %v9_0.1 = mul i64 %v9_0, %v12_0
  %v6_0.1 = icmp ugt i64 %v12_0, 1
  br i1 %v6_0.1, label %for.body.2.2, label %for.end.2, !llvm.loop !4

for.body.2.2:                                     ; preds = %for.body.2.1
  %v12_0.1 = add nsw i64 %i_12.ph, -2
  %v9_0.2 = mul i64 %v9_0.1, %v12_0.1
  br label %for.end.2

for.end.2:                                        ; preds = %for.body.2, %for.body.2.1, %for.body.2.2, %middle.block, %pre_entry
  %result_1.lcssa = phi i64 [ 1, %pre_entry ], [ %bin.rdx8, %middle.block ], [ %v9_0, %for.body.2 ], [ %v9_0.1, %for.body.2.1 ], [ %v9_0.2, %for.body.2.2 ]
  %8 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %result_1.lcssa)
  %9 = tail call i32 @putchar(i32 10)
  ret void
}

define dso_local noundef i32 @main(i32 %argc, ptr nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %for.body.2.i.preheader.i, label %2

2:                                                ; preds = %0
  %3 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.4, i32 0, i32 %1)
  tail call void @exit(i32 2)
  unreachable

for.body.2.i.preheader.i:                         ; preds = %0, %__orig_main.exit.i
  %loop_counter_11.i = phi i64 [ %loop_counter_2.i, %__orig_main.exit.i ], [ 10, %0 ]
  %min.iters.check = icmp ult i64 %loop_counter_11.i, 2
  br i1 %min.iters.check, label %for.body.2.i.i, label %vector.ph

vector.ph:                                        ; preds = %for.body.2.i.preheader.i
  %n.vec = and i64 %loop_counter_11.i, 9223372036854775806
  %ind.end = and i64 %loop_counter_11.i, 1
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 1, %vector.ph ], [ %5, %vector.body ]
  %vec.phi2 = phi i64 [ 1, %vector.ph ], [ %6, %vector.body ]
  %offset.idx = sub i64 %loop_counter_11.i, %index
  %4 = add i64 %offset.idx, -1
  %5 = mul i64 %offset.idx, %vec.phi
  %6 = mul i64 %4, %vec.phi2
  %index.next = add nuw i64 %index, 2
  %7 = icmp eq i64 %index.next, %n.vec
  br i1 %7, label %middle.block, label %vector.body, !llvm.loop !5

middle.block:                                     ; preds = %vector.body
  %bin.rdx = mul i64 %6, %5
  %cmp.n = icmp eq i64 %loop_counter_11.i, %n.vec
  br i1 %cmp.n, label %__orig_main.exit.i, label %for.body.2.i.i

for.body.2.i.i:                                   ; preds = %middle.block, %for.body.2.i.preheader.i
  %result_13.i.i.ph = phi i64 [ 1, %for.body.2.i.preheader.i ], [ %bin.rdx, %middle.block ]
  %i_12.i.i.ph = phi i64 [ %loop_counter_11.i, %for.body.2.i.preheader.i ], [ %ind.end, %middle.block ]
  %v9_0.i.i = mul nuw i64 %i_12.i.i.ph, %result_13.i.i.ph
  br label %__orig_main.exit.i

__orig_main.exit.i:                               ; preds = %for.body.2.i.i, %middle.block
  %v9_0.i.i.lcssa = phi i64 [ %bin.rdx, %middle.block ], [ %v9_0.i.i, %for.body.2.i.i ]
  %8 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %v9_0.i.i.lcssa)
  %9 = tail call i32 @putchar(i32 10)
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 50000
  br i1 %exitcond.not.i, label %__main.exit, label %for.body.2.i.preheader.i

__main.exit:                                      ; preds = %__orig_main.exit.i
  ret i32 0
}

attributes #0 = { nofree nounwind }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read) }

!0 = distinct !{!0, !1, !2}
!1 = !{!"llvm.loop.isvectorized", i32 1}
!2 = !{!"llvm.loop.unroll.runtime.disable"}
!3 = distinct !{!3, !1, !2}
!4 = distinct !{!4, !1}
!5 = distinct !{!5, !1, !2}
