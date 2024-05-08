; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmppFOXgD/dot-product-init.ll'
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

; Function Attrs: nofree norecurse nosync nounwind memory(argmem: read)
define dso_local i64 @__dot_product(ptr nocapture readonly %vectorA, ptr nocapture readonly %vectorB, i64 %size) local_unnamed_addr #2 {
pre_entry:
  %smax = tail call i64 @llvm.smax.i64(i64 %size, i64 1)
  %min.iters.check = icmp slt i64 %size, 4
  br i1 %min.iters.check, label %loop.preheader, label %vector.ph

vector.ph:                                        ; preds = %pre_entry
  %n.vec = and i64 %smax, 9223372036854775804
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 0, %vector.ph ], [ %23, %vector.body ]
  %vec.phi1 = phi i64 [ 0, %vector.ph ], [ %24, %vector.body ]
  %vec.phi2 = phi i64 [ 0, %vector.ph ], [ %25, %vector.body ]
  %vec.phi3 = phi i64 [ 0, %vector.ph ], [ %26, %vector.body ]
  %0 = or disjoint i64 %index, 1
  %1 = or disjoint i64 %index, 2
  %2 = or disjoint i64 %index, 3
  %3 = getelementptr inbounds i64, ptr %vectorA, i64 %index
  %4 = getelementptr inbounds i64, ptr %vectorA, i64 %0
  %5 = getelementptr inbounds i64, ptr %vectorA, i64 %1
  %6 = getelementptr inbounds i64, ptr %vectorA, i64 %2
  %7 = getelementptr inbounds i64, ptr %vectorB, i64 %index
  %8 = getelementptr inbounds i64, ptr %vectorB, i64 %0
  %9 = getelementptr inbounds i64, ptr %vectorB, i64 %1
  %10 = getelementptr inbounds i64, ptr %vectorB, i64 %2
  %11 = load i64, ptr %3, align 8
  %12 = load i64, ptr %4, align 8
  %13 = load i64, ptr %5, align 8
  %14 = load i64, ptr %6, align 8
  %15 = load i64, ptr %7, align 8
  %16 = load i64, ptr %8, align 8
  %17 = load i64, ptr %9, align 8
  %18 = load i64, ptr %10, align 8
  %19 = mul i64 %15, %11
  %20 = mul i64 %16, %12
  %21 = mul i64 %17, %13
  %22 = mul i64 %18, %14
  %23 = add i64 %19, %vec.phi
  %24 = add i64 %20, %vec.phi1
  %25 = add i64 %21, %vec.phi2
  %26 = add i64 %22, %vec.phi3
  %index.next = add nuw i64 %index, 4
  %27 = icmp eq i64 %index.next, %n.vec
  br i1 %27, label %middle.block, label %vector.body, !llvm.loop !0

middle.block:                                     ; preds = %vector.body
  %bin.rdx = add i64 %24, %23
  %bin.rdx4 = add i64 %25, %bin.rdx
  %bin.rdx5 = add i64 %26, %bin.rdx4
  %cmp.n = icmp eq i64 %smax, %n.vec
  br i1 %cmp.n, label %done, label %loop.preheader

loop.preheader:                                   ; preds = %pre_entry, %middle.block
  %answer_1.ph = phi i64 [ 0, %pre_entry ], [ %bin.rdx5, %middle.block ]
  %index_1.ph = phi i64 [ 0, %pre_entry ], [ %n.vec, %middle.block ]
  br label %loop

loop:                                             ; preds = %loop.preheader, %loop
  %answer_1 = phi i64 [ %answer_2, %loop ], [ %answer_1.ph, %loop.preheader ]
  %index_1 = phi i64 [ %index_2, %loop ], [ %index_1.ph, %loop.preheader ]
  %ptrA_0 = getelementptr inbounds i64, ptr %vectorA, i64 %index_1
  %ptrB_0 = getelementptr inbounds i64, ptr %vectorB, i64 %index_1
  %valA_0 = load i64, ptr %ptrA_0, align 8
  %valB_0 = load i64, ptr %ptrB_0, align 8
  %tmp_0 = mul i64 %valB_0, %valA_0
  %answer_2 = add i64 %tmp_0, %answer_1
  %index_2 = add nuw nsw i64 %index_1, 1
  %exitcond.not = icmp eq i64 %index_2, %smax
  br i1 %exitcond.not, label %done, label %loop, !llvm.loop !3

done:                                             ; preds = %loop, %middle.block
  %answer_2.lcssa = phi i64 [ %bin.rdx5, %middle.block ], [ %answer_2, %loop ]
  ret i64 %answer_2.lcssa
}

; Function Attrs: nofree nounwind
define dso_local void @__main() local_unnamed_addr #0 {
b0:
  br label %loop_body

loop_body:                                        ; preds = %b0, %loop_body
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %loop_body ]
  %val_1.i = add nuw nsw i64 %loop_counter_11, 17050
  %0 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %val_1.i)
  %1 = tail call i32 @putchar(i32 10)
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 1000000
  br i1 %exitcond.not, label %loop_done, label %loop_body

loop_done:                                        ; preds = %loop_body
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %x) local_unnamed_addr #0 {
pre_entry:
  %val_1 = add i64 %x, 17050
  %0 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %val_1)
  %1 = tail call i32 @putchar(i32 10)
  ret void
}

define dso_local noundef i32 @main(i32 %argc, ptr nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %loop_body.i, label %2

2:                                                ; preds = %0
  %3 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.4, i32 0, i32 %1)
  tail call void @exit(i32 2)
  unreachable

loop_body.i:                                      ; preds = %0, %loop_body.i
  %loop_counter_11.i = phi i64 [ %loop_counter_2.i, %loop_body.i ], [ 10, %0 ]
  %val_1.i.i = add nuw nsw i64 %loop_counter_11.i, 17050
  %4 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %val_1.i.i)
  %5 = tail call i32 @putchar(i32 10)
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 1000000
  br i1 %exitcond.not.i, label %__main.exit, label %loop_body.i

__main.exit:                                      ; preds = %loop_body.i
  ret i32 0
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.smax.i64(i64, i64) #3

attributes #0 = { nofree nounwind }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read) }
attributes #2 = { nofree norecurse nosync nounwind memory(argmem: read) }
attributes #3 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }

!0 = distinct !{!0, !1, !2}
!1 = !{!"llvm.loop.isvectorized", i32 1}
!2 = !{!"llvm.loop.unroll.runtime.disable"}
!3 = distinct !{!3, !1}
