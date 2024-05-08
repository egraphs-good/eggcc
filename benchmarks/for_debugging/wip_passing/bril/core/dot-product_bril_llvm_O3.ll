; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpJJGhLK/compile.ll'
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
declare dso_local noundef i32 @printf(i8* nocapture noundef readonly, ...) local_unnamed_addr #0

declare dso_local void @exit(i32) local_unnamed_addr

; Function Attrs: argmemonly mustprogress nofree norecurse nosync nounwind readonly willreturn
define dso_local i32 @btoi(i8* nocapture readonly %0) local_unnamed_addr #1 {
  %2 = load i8, i8* %0, align 1
  %3 = icmp eq i8 %2, 116
  %4 = zext i1 %3 to i32
  ret i32 %4
}

; Function Attrs: nofree nounwind
define dso_local void @print_bool(i1 %0) local_unnamed_addr #0 {
  %. = select i1 %0, i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str, i64 0, i64 0), i8* getelementptr inbounds ([6 x i8], [6 x i8]* @.str.1, i64 0, i64 0)
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) %.)
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
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %0)
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @print_ptr(i8* nocapture readnone %0) local_unnamed_addr #0 {
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([9 x i8], [9 x i8]* @.str.3, i64 0, i64 0))
  ret void
}

; Function Attrs: argmemonly nofree norecurse nosync nounwind readonly
define dso_local i64 @__dot_product(i64* nocapture readonly %vectorA, i64* nocapture readonly %vectorB, i64 %size) local_unnamed_addr #2 {
pre_entry:
  %smax = call i64 @llvm.smax.i64(i64 %size, i64 1)
  %min.iters.check = icmp ult i64 %smax, 4
  br i1 %min.iters.check, label %loop.preheader, label %vector.ph

vector.ph:                                        ; preds = %pre_entry
  %n.vec = and i64 %smax, 9223372036854775804
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 0, %vector.ph ], [ %20, %vector.body ]
  %vec.phi1 = phi i64 [ 0, %vector.ph ], [ %21, %vector.body ]
  %vec.phi2 = phi i64 [ 0, %vector.ph ], [ %22, %vector.body ]
  %vec.phi3 = phi i64 [ 0, %vector.ph ], [ %23, %vector.body ]
  %induction4 = or i64 %index, 1
  %induction5 = or i64 %index, 2
  %induction6 = or i64 %index, 3
  %0 = getelementptr inbounds i64, i64* %vectorA, i64 %index
  %1 = getelementptr inbounds i64, i64* %vectorA, i64 %induction4
  %2 = getelementptr inbounds i64, i64* %vectorA, i64 %induction5
  %3 = getelementptr inbounds i64, i64* %vectorA, i64 %induction6
  %4 = getelementptr inbounds i64, i64* %vectorB, i64 %index
  %5 = getelementptr inbounds i64, i64* %vectorB, i64 %induction4
  %6 = getelementptr inbounds i64, i64* %vectorB, i64 %induction5
  %7 = getelementptr inbounds i64, i64* %vectorB, i64 %induction6
  %8 = load i64, i64* %0, align 8
  %9 = load i64, i64* %1, align 8
  %10 = load i64, i64* %2, align 8
  %11 = load i64, i64* %3, align 8
  %12 = load i64, i64* %4, align 8
  %13 = load i64, i64* %5, align 8
  %14 = load i64, i64* %6, align 8
  %15 = load i64, i64* %7, align 8
  %16 = mul i64 %12, %8
  %17 = mul i64 %13, %9
  %18 = mul i64 %14, %10
  %19 = mul i64 %15, %11
  %20 = add i64 %16, %vec.phi
  %21 = add i64 %17, %vec.phi1
  %22 = add i64 %18, %vec.phi2
  %23 = add i64 %19, %vec.phi3
  %index.next = add nuw i64 %index, 4
  %24 = icmp eq i64 %index.next, %n.vec
  br i1 %24, label %middle.block, label %vector.body, !llvm.loop !0

middle.block:                                     ; preds = %vector.body
  %bin.rdx = add i64 %21, %20
  %bin.rdx7 = add i64 %22, %bin.rdx
  %bin.rdx8 = add i64 %23, %bin.rdx7
  %cmp.n = icmp eq i64 %smax, %n.vec
  br i1 %cmp.n, label %done, label %loop.preheader

loop.preheader:                                   ; preds = %pre_entry, %middle.block
  %answer_1.ph = phi i64 [ 0, %pre_entry ], [ %bin.rdx8, %middle.block ]
  %index_1.ph = phi i64 [ 0, %pre_entry ], [ %n.vec, %middle.block ]
  br label %loop

loop:                                             ; preds = %loop.preheader, %loop
  %answer_1 = phi i64 [ %answer_2, %loop ], [ %answer_1.ph, %loop.preheader ]
  %index_1 = phi i64 [ %index_2, %loop ], [ %index_1.ph, %loop.preheader ]
  %ptrA_0 = getelementptr inbounds i64, i64* %vectorA, i64 %index_1
  %ptrB_0 = getelementptr inbounds i64, i64* %vectorB, i64 %index_1
  %valA_0 = load i64, i64* %ptrA_0, align 8
  %valB_0 = load i64, i64* %ptrB_0, align 8
  %tmp_0 = mul i64 %valB_0, %valA_0
  %answer_2 = add i64 %tmp_0, %answer_1
  %index_2 = add nuw nsw i64 %index_1, 1
  %exitcond.not = icmp eq i64 %index_2, %smax
  br i1 %exitcond.not, label %done, label %loop, !llvm.loop !2

done:                                             ; preds = %loop, %middle.block
  %answer_2.lcssa = phi i64 [ %bin.rdx8, %middle.block ], [ %answer_2, %loop ]
  ret i64 %answer_2.lcssa
}

; Function Attrs: nofree nounwind
define dso_local void @__main() local_unnamed_addr #0 {
b0:
  br label %loop_body

loop_body:                                        ; preds = %b0, %loop_body
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %loop_body ]
  %val_1.i = add nuw nsw i64 %loop_counter_11, 17050
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_1.i) #3
  %1 = tail call i32 @putchar(i32 10) #3
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 1000000
  br i1 %exitcond.not, label %loop_done, label %loop_body

loop_done:                                        ; preds = %loop_body
  ret void
}

; Function Attrs: nounwind
define dso_local void @__orig_main(i64 %x) local_unnamed_addr #3 {
pre_entry:
  %val_1 = add i64 %x, 17050
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_1) #3
  %1 = tail call i32 @putchar(i32 10) #3
  ret void
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %loop_body.i, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #6
  ret i32 0

loop_body.i:                                      ; preds = %0, %loop_body.i
  %loop_counter_11.i = phi i64 [ %loop_counter_2.i, %loop_body.i ], [ 10, %0 ]
  %val_1.i.i = add nuw nsw i64 %loop_counter_11.i, 17050
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_1.i.i) #3
  %3 = tail call i32 @putchar(i32 10) #3
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 1000000
  br i1 %exitcond.not.i, label %__main.exit, label %loop_body.i

__main.exit:                                      ; preds = %loop_body.i
  ret i32 0
}

; Function Attrs: nocallback nofree nosync nounwind readnone speculatable willreturn
declare i64 @llvm.smax.i64(i64, i64) #4

; Function Attrs: cold minsize noreturn
define internal void @main.cold.1(i32 %0) #5 {
newFuncRoot:
  br label %1

1:                                                ; preds = %newFuncRoot
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([33 x i8], [33 x i8]* @.str.4, i64 0, i64 0), i32 0, i32 %0)
  tail call void @exit(i32 2)
  unreachable
}

attributes #0 = { nofree nounwind }
attributes #1 = { argmemonly mustprogress nofree norecurse nosync nounwind readonly willreturn }
attributes #2 = { argmemonly nofree norecurse nosync nounwind readonly }
attributes #3 = { nounwind }
attributes #4 = { nocallback nofree nosync nounwind readnone speculatable willreturn }
attributes #5 = { cold minsize noreturn }
attributes #6 = { noinline }

!0 = distinct !{!0, !1}
!1 = !{!"llvm.loop.isvectorized", i32 1}
!2 = distinct !{!2, !1}
