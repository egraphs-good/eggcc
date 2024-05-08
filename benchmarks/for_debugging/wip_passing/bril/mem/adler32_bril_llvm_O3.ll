; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpy5GQT7/compile.ll'
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

; Function Attrs: inaccessiblememonly mustprogress nofree nounwind willreturn allocsize(0)
declare dso_local noalias noundef i8* @malloc(i64 noundef) local_unnamed_addr #1

; Function Attrs: inaccessiblemem_or_argmemonly mustprogress nounwind willreturn
declare dso_local void @free(i8* nocapture noundef) local_unnamed_addr #2

; Function Attrs: argmemonly mustprogress nofree norecurse nosync nounwind readonly willreturn
define dso_local i32 @btoi(i8* nocapture readonly %0) local_unnamed_addr #3 {
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

; Function Attrs: nounwind
define dso_local void @__main() local_unnamed_addr #4 {
b0:
  br label %loop_body

loop_body:                                        ; preds = %b0, %loop_body
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %loop_body ]
  tail call void @__orig_main(i64 %loop_counter_11)
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 50000
  br i1 %exitcond.not, label %loop_done, label %loop_body

loop_done:                                        ; preds = %loop_body
  ret void
}

; Function Attrs: nounwind
define dso_local void @__orig_main(i64 %size) local_unnamed_addr #4 {
pre_entry:
  %z0 = shl i64 %size, 3
  %z1 = tail call i8* @malloc(i64 %z0)
  %arr_0 = bitcast i8* %z1 to i64*
  %smax.i = tail call i64 @llvm.smax.i64(i64 %size, i64 1) #4
  %min.iters.check = icmp ult i64 %smax.i, 8
  br i1 %min.iters.check, label %loop.i.preheader, label %vector.ph

vector.ph:                                        ; preds = %pre_entry
  %n.vec = and i64 %smax.i, 9223372036854775800
  %ind.end = getelementptr i64, i64* %arr_0, i64 %n.vec
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.ind = phi <2 x i64> [ <i64 0, i64 1>, %vector.ph ], [ %vec.ind.next, %vector.body ]
  %next.gep = getelementptr i64, i64* %arr_0, i64 %index
  %step.add = add <2 x i64> %vec.ind, <i64 2, i64 2>
  %step.add12 = add <2 x i64> %vec.ind, <i64 4, i64 4>
  %step.add13 = add <2 x i64> %vec.ind, <i64 6, i64 6>
  %0 = bitcast i64* %next.gep to <2 x i64>*
  store <2 x i64> %vec.ind, <2 x i64>* %0, align 8
  %1 = getelementptr i64, i64* %next.gep, i64 2
  %2 = bitcast i64* %1 to <2 x i64>*
  store <2 x i64> %step.add, <2 x i64>* %2, align 8
  %3 = getelementptr i64, i64* %next.gep, i64 4
  %4 = bitcast i64* %3 to <2 x i64>*
  store <2 x i64> %step.add12, <2 x i64>* %4, align 8
  %5 = getelementptr i64, i64* %next.gep, i64 6
  %6 = bitcast i64* %5 to <2 x i64>*
  store <2 x i64> %step.add13, <2 x i64>* %6, align 8
  %index.next = add nuw i64 %index, 8
  %vec.ind.next = add <2 x i64> %vec.ind, <i64 8, i64 8>
  %7 = icmp eq i64 %index.next, %n.vec
  br i1 %7, label %middle.block, label %vector.body, !llvm.loop !0

middle.block:                                     ; preds = %vector.body
  %cmp.n = icmp eq i64 %smax.i, %n.vec
  br i1 %cmp.n, label %loop.i7.preheader, label %loop.i.preheader

loop.i.preheader:                                 ; preds = %pre_entry, %middle.block
  %loc_1.i.ph = phi i64* [ %arr_0, %pre_entry ], [ %ind.end, %middle.block ]
  %curr_1.i.ph = phi i64 [ 0, %pre_entry ], [ %n.vec, %middle.block ]
  br label %loop.i

loop.i:                                           ; preds = %loop.i.preheader, %loop.i
  %loc_1.i = phi i64* [ %loc_2.i, %loop.i ], [ %loc_1.i.ph, %loop.i.preheader ]
  %curr_1.i = phi i64 [ %curr_2.i, %loop.i ], [ %curr_1.i.ph, %loop.i.preheader ]
  store i64 %curr_1.i, i64* %loc_1.i, align 8
  %loc_2.i = getelementptr inbounds i64, i64* %loc_1.i, i64 1
  %curr_2.i = add nuw nsw i64 %curr_1.i, 1
  %exitcond.not.i = icmp eq i64 %curr_2.i, %smax.i
  br i1 %exitcond.not.i, label %loop.i7.preheader, label %loop.i, !llvm.loop !2

loop.i7.preheader:                                ; preds = %loop.i, %middle.block
  br label %loop.i7

loop.i7:                                          ; preds = %loop.i7.preheader, %loop.i7
  %loc_1.i2 = phi i64* [ %loc_2.i4, %loop.i7 ], [ %arr_0, %loop.i7.preheader ]
  %curr_1.i3 = phi i64 [ %curr_2.i5, %loop.i7 ], [ 0, %loop.i7.preheader ]
  %b_1.i = phi i64 [ %b_2.i, %loop.i7 ], [ 0, %loop.i7.preheader ]
  %a_1.i = phi i64 [ %a_2.i, %loop.i7 ], [ 1, %loop.i7.preheader ]
  %val_0.i = load i64, i64* %loc_1.i2, align 8
  %a_2.i = add i64 %val_0.i, %a_1.i
  %b_2.i = add i64 %a_2.i, %b_1.i
  %loc_2.i4 = getelementptr inbounds i64, i64* %loc_1.i2, i64 1
  %curr_2.i5 = add nuw nsw i64 %curr_1.i3, 1
  %exitcond.not.i6 = icmp eq i64 %curr_2.i5, %smax.i
  br i1 %exitcond.not.i6, label %exit.i, label %loop.i7

exit.i:                                           ; preds = %loop.i7
  %8 = srem i64 %a_2.i, 65521
  %9 = srem i64 %b_2.i, 65521
  %b_4.i = shl nsw i64 %9, 16
  br label %loop.i.i

loop.i.i:                                         ; preds = %loop.i.i, %exit.i
  %result_1.i.i = phi i64 [ %spec.select.i.i, %loop.i.i ], [ 0, %exit.i ]
  %val_1.i.i = phi i64 [ %val_2.i.i, %loop.i.i ], [ 1, %exit.i ]
  %y_1.i.i = phi i64 [ %y_2.i.i, %loop.i.i ], [ %8, %exit.i ]
  %x_1.i.i = phi i64 [ %x_2.i.i, %loop.i.i ], [ %b_4.i, %exit.i ]
  %10 = and i64 %x_1.i.i, -9223372036854775807
  %xodd_0.i.i = icmp eq i64 %10, 1
  %11 = and i64 %y_1.i.i, -9223372036854775807
  %yodd_0.i.i = icmp eq i64 %11, 1
  %cond_0.i.i = or i1 %yodd_0.i.i, %xodd_0.i.i
  %result_2.i.i = select i1 %cond_0.i.i, i64 %val_1.i.i, i64 0
  %spec.select.i.i = add i64 %result_2.i.i, %result_1.i.i
  %x_2.i.i = sdiv i64 %x_1.i.i, 2
  %y_2.i.i = sdiv i64 %y_1.i.i, 2
  %xpos_0.i.i = icmp sgt i64 %x_1.i.i, 1
  %ypos_0.i.i = icmp sgt i64 %y_1.i.i, 1
  %val_2.i.i = shl i64 %val_1.i.i, 1
  %continue_0.i.i = or i1 %ypos_0.i.i, %xpos_0.i.i
  br i1 %continue_0.i.i, label %loop.i.i, label %__adler32.exit

__adler32.exit:                                   ; preds = %loop.i.i
  %12 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %spec.select.i.i) #4
  %13 = tail call i32 @putchar(i32 10) #4
  tail call void @free(i8* %z1)
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind readnone willreturn
define dso_local i64 @__mod(i64 %r, i64 %s) local_unnamed_addr #5 {
pre_entry:
  %0 = srem i64 %r, %s
  ret i64 %0
}

; Function Attrs: nofree norecurse nosync nounwind writeonly
define dso_local void @__fill_array(i64* nocapture writeonly %arr, i64 %size) local_unnamed_addr #6 {
pre_entry:
  %smax = call i64 @llvm.smax.i64(i64 %size, i64 1)
  %min.iters.check = icmp ult i64 %smax, 8
  br i1 %min.iters.check, label %loop.preheader, label %vector.ph

vector.ph:                                        ; preds = %pre_entry
  %n.vec = and i64 %smax, 9223372036854775800
  %ind.end = getelementptr i64, i64* %arr, i64 %n.vec
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.ind = phi <2 x i64> [ <i64 0, i64 1>, %vector.ph ], [ %vec.ind.next, %vector.body ]
  %next.gep = getelementptr i64, i64* %arr, i64 %index
  %step.add = add <2 x i64> %vec.ind, <i64 2, i64 2>
  %step.add5 = add <2 x i64> %vec.ind, <i64 4, i64 4>
  %step.add6 = add <2 x i64> %vec.ind, <i64 6, i64 6>
  %0 = bitcast i64* %next.gep to <2 x i64>*
  store <2 x i64> %vec.ind, <2 x i64>* %0, align 8
  %1 = getelementptr i64, i64* %next.gep, i64 2
  %2 = bitcast i64* %1 to <2 x i64>*
  store <2 x i64> %step.add, <2 x i64>* %2, align 8
  %3 = getelementptr i64, i64* %next.gep, i64 4
  %4 = bitcast i64* %3 to <2 x i64>*
  store <2 x i64> %step.add5, <2 x i64>* %4, align 8
  %5 = getelementptr i64, i64* %next.gep, i64 6
  %6 = bitcast i64* %5 to <2 x i64>*
  store <2 x i64> %step.add6, <2 x i64>* %6, align 8
  %index.next = add nuw i64 %index, 8
  %vec.ind.next = add <2 x i64> %vec.ind, <i64 8, i64 8>
  %7 = icmp eq i64 %index.next, %n.vec
  br i1 %7, label %middle.block, label %vector.body, !llvm.loop !4

middle.block:                                     ; preds = %vector.body
  %cmp.n = icmp eq i64 %smax, %n.vec
  br i1 %cmp.n, label %exit, label %loop.preheader

loop.preheader:                                   ; preds = %pre_entry, %middle.block
  %loc_1.ph = phi i64* [ %arr, %pre_entry ], [ %ind.end, %middle.block ]
  %curr_1.ph = phi i64 [ 0, %pre_entry ], [ %n.vec, %middle.block ]
  br label %loop

loop:                                             ; preds = %loop.preheader, %loop
  %loc_1 = phi i64* [ %loc_2, %loop ], [ %loc_1.ph, %loop.preheader ]
  %curr_1 = phi i64 [ %curr_2, %loop ], [ %curr_1.ph, %loop.preheader ]
  store i64 %curr_1, i64* %loc_1, align 8
  %loc_2 = getelementptr inbounds i64, i64* %loc_1, i64 1
  %curr_2 = add nuw nsw i64 %curr_1, 1
  %exitcond.not = icmp eq i64 %curr_2, %smax
  br i1 %exitcond.not, label %exit, label %loop, !llvm.loop !5

exit:                                             ; preds = %loop, %middle.block
  ret void
}

; Function Attrs: nofree norecurse nosync nounwind readnone
define dso_local i64 @__bitwise_or(i64 %x, i64 %y) local_unnamed_addr #7 {
pre_entry:
  br label %loop

loop:                                             ; preds = %loop, %pre_entry
  %result_1 = phi i64 [ %spec.select, %loop ], [ 0, %pre_entry ]
  %val_1 = phi i64 [ %val_2, %loop ], [ 1, %pre_entry ]
  %y_1 = phi i64 [ %y_2, %loop ], [ %y, %pre_entry ]
  %x_1 = phi i64 [ %x_2, %loop ], [ %x, %pre_entry ]
  %0 = and i64 %x_1, -9223372036854775807
  %xodd_0 = icmp eq i64 %0, 1
  %1 = and i64 %y_1, -9223372036854775807
  %yodd_0 = icmp eq i64 %1, 1
  %cond_0 = or i1 %yodd_0, %xodd_0
  %result_2 = select i1 %cond_0, i64 %val_1, i64 0
  %spec.select = add i64 %result_2, %result_1
  %x_2 = sdiv i64 %x_1, 2
  %y_2 = sdiv i64 %y_1, 2
  %xpos_0 = icmp sgt i64 %x_1, 1
  %ypos_0 = icmp sgt i64 %y_1, 1
  %val_2 = shl i64 %val_1, 1
  %continue_0 = or i1 %ypos_0, %xpos_0
  br i1 %continue_0, label %loop, label %exit

exit:                                             ; preds = %loop
  ret i64 %spec.select
}

; Function Attrs: nofree norecurse nosync nounwind readonly
define dso_local i64 @__adler32(i64* nocapture readonly %arr, i64 %size) local_unnamed_addr #8 {
pre_entry:
  %smax = call i64 @llvm.smax.i64(i64 %size, i64 1)
  br label %loop

loop:                                             ; preds = %loop, %pre_entry
  %loc_1 = phi i64* [ %loc_2, %loop ], [ %arr, %pre_entry ]
  %curr_1 = phi i64 [ %curr_2, %loop ], [ 0, %pre_entry ]
  %b_1 = phi i64 [ %b_2, %loop ], [ 0, %pre_entry ]
  %a_1 = phi i64 [ %a_2, %loop ], [ 1, %pre_entry ]
  %val_0 = load i64, i64* %loc_1, align 8
  %a_2 = add i64 %val_0, %a_1
  %b_2 = add i64 %a_2, %b_1
  %loc_2 = getelementptr inbounds i64, i64* %loc_1, i64 1
  %curr_2 = add nuw nsw i64 %curr_1, 1
  %exitcond.not = icmp eq i64 %curr_2, %smax
  br i1 %exitcond.not, label %exit, label %loop

exit:                                             ; preds = %loop
  %0 = srem i64 %a_2, 65521
  %1 = srem i64 %b_2, 65521
  %b_4 = shl nsw i64 %1, 16
  br label %loop.i

loop.i:                                           ; preds = %loop.i, %exit
  %result_1.i = phi i64 [ %spec.select.i, %loop.i ], [ 0, %exit ]
  %val_1.i = phi i64 [ %val_2.i, %loop.i ], [ 1, %exit ]
  %y_1.i = phi i64 [ %y_2.i, %loop.i ], [ %0, %exit ]
  %x_1.i = phi i64 [ %x_2.i, %loop.i ], [ %b_4, %exit ]
  %2 = and i64 %x_1.i, -9223372036854775807
  %xodd_0.i = icmp eq i64 %2, 1
  %3 = and i64 %y_1.i, -9223372036854775807
  %yodd_0.i = icmp eq i64 %3, 1
  %cond_0.i = or i1 %yodd_0.i, %xodd_0.i
  %result_2.i = select i1 %cond_0.i, i64 %val_1.i, i64 0
  %spec.select.i = add i64 %result_2.i, %result_1.i
  %x_2.i = sdiv i64 %x_1.i, 2
  %y_2.i = sdiv i64 %y_1.i, 2
  %xpos_0.i = icmp sgt i64 %x_1.i, 1
  %ypos_0.i = icmp sgt i64 %y_1.i, 1
  %val_2.i = shl i64 %val_1.i, 1
  %continue_0.i = or i1 %ypos_0.i, %xpos_0.i
  br i1 %continue_0.i, label %loop.i, label %__bitwise_or.exit

__bitwise_or.exit:                                ; preds = %loop.i
  ret i64 %spec.select.i
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %loop_body.i, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #11
  ret i32 0

loop_body.i:                                      ; preds = %0, %loop_body.i
  %loop_counter_11.i = phi i64 [ %loop_counter_2.i, %loop_body.i ], [ 10, %0 ]
  tail call void @__orig_main(i64 %loop_counter_11.i) #4
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 50000
  br i1 %exitcond.not.i, label %__main.exit, label %loop_body.i

__main.exit:                                      ; preds = %loop_body.i
  ret i32 0
}

; Function Attrs: nocallback nofree nosync nounwind readnone speculatable willreturn
declare i64 @llvm.smax.i64(i64, i64) #9

; Function Attrs: cold minsize noreturn
define internal void @main.cold.1(i32 %0) #10 {
newFuncRoot:
  br label %1

1:                                                ; preds = %newFuncRoot
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([33 x i8], [33 x i8]* @.str.4, i64 0, i64 0), i32 0, i32 %0)
  tail call void @exit(i32 2)
  unreachable
}

attributes #0 = { nofree nounwind }
attributes #1 = { inaccessiblememonly mustprogress nofree nounwind willreturn allocsize(0) }
attributes #2 = { inaccessiblemem_or_argmemonly mustprogress nounwind willreturn }
attributes #3 = { argmemonly mustprogress nofree norecurse nosync nounwind readonly willreturn }
attributes #4 = { nounwind }
attributes #5 = { mustprogress nofree norecurse nosync nounwind readnone willreturn }
attributes #6 = { nofree norecurse nosync nounwind writeonly }
attributes #7 = { nofree norecurse nosync nounwind readnone }
attributes #8 = { nofree norecurse nosync nounwind readonly }
attributes #9 = { nocallback nofree nosync nounwind readnone speculatable willreturn }
attributes #10 = { cold minsize noreturn }
attributes #11 = { noinline }

!0 = distinct !{!0, !1}
!1 = !{!"llvm.loop.isvectorized", i32 1}
!2 = distinct !{!2, !3, !1}
!3 = !{!"llvm.loop.unroll.runtime.disable"}
!4 = distinct !{!4, !1}
!5 = distinct !{!5, !3, !1}
