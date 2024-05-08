; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpmPsNmU/compile.ll'
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

; Function Attrs: argmemonly mustprogress nofree norecurse nosync nounwind willreturn
define dso_local i64 @__rand(i64* nocapture %seq, i64 %max) local_unnamed_addr #4 {
pre_entry:
  %x_0 = load i64, i64* %seq, align 8
  %ax_0 = mul i64 %x_0, 25214903917
  %axpc_0 = add i64 %ax_0, 11
  %next_2 = srem i64 %axpc_0, 281474976710656
  store i64 %next_2, i64* %seq, align 8
  %0 = srem i64 %next_2, %max
  ret i64 %0
}

; Function Attrs: nofree nounwind
define dso_local noalias i64* @__randarray(i64 %size, i64* nocapture %rng) local_unnamed_addr #0 {
pre_entry:
  %z0 = shl i64 %size, 3
  %z1 = tail call i8* @malloc(i64 %z0)
  %arr_0 = bitcast i8* %z1 to i64*
  %cond_01 = icmp sgt i64 %size, 0
  br i1 %cond_01, label %body.lr.ph, label %done

body.lr.ph:                                       ; preds = %pre_entry
  %rng.promoted = load i64, i64* %rng, align 8
  br label %body

body:                                             ; preds = %body.lr.ph, %body
  %next_2.i3 = phi i64 [ %rng.promoted, %body.lr.ph ], [ %next_2.i, %body ]
  %i_12 = phi i64 [ 0, %body.lr.ph ], [ %i_2, %body ]
  %ax_0.i = mul i64 %next_2.i3, 25214903917
  %axpc_0.i = add i64 %ax_0.i, 11
  %next_2.i = srem i64 %axpc_0.i, 281474976710656
  %0 = srem i64 %next_2.i, 1000
  %loc_0 = getelementptr inbounds i64, i64* %arr_0, i64 %i_12
  store i64 %0, i64* %loc_0, align 8
  %i_2 = add nuw nsw i64 %i_12, 1
  %exitcond.not = icmp eq i64 %i_2, %size
  br i1 %exitcond.not, label %loop.done_crit_edge, label %body

loop.done_crit_edge:                              ; preds = %body
  store i64 %next_2.i, i64* %rng, align 8
  br label %done

done:                                             ; preds = %loop.done_crit_edge, %pre_entry
  ret i64* %arr_0
}

; Function Attrs: nofree nounwind
define dso_local void @__printarray(i64 %size, i64* nocapture readonly %arr) local_unnamed_addr #0 {
pre_entry:
  %cond_01 = icmp sgt i64 %size, 0
  br i1 %cond_01, label %body, label %done

body:                                             ; preds = %pre_entry, %body
  %i_12 = phi i64 [ %i_2, %body ], [ 0, %pre_entry ]
  %loc_0 = getelementptr inbounds i64, i64* %arr, i64 %i_12
  %val_0 = load i64, i64* %loc_0, align 8
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0) #6
  %1 = tail call i32 @putchar(i32 10) #6
  %i_2 = add nuw nsw i64 %i_12, 1
  %exitcond.not = icmp eq i64 %i_2, %size
  br i1 %exitcond.not, label %done, label %body

done:                                             ; preds = %body, %pre_entry
  ret void
}

; Function Attrs: argmemonly nofree norecurse nosync nounwind
define dso_local void @__matmul(i64 %size, i64* nocapture readonly %arr1, i64* nocapture readonly %arr2, i64* nocapture writeonly %dest) local_unnamed_addr #5 {
pre_entry:
  %cond_06 = icmp sgt i64 %size, 0
  br i1 %cond_06, label %col.loop.preheader.us.us.preheader, label %row.done

col.loop.preheader.us.us.preheader:               ; preds = %pre_entry
  %min.iters.check = icmp eq i64 %size, 1
  %n.vec = and i64 %size, -2
  %cmp.n = icmp eq i64 %n.vec, %size
  br i1 %min.iters.check, label %sum.body.us.us.us.us.us, label %col.loop.preheader.us.us

sum.body.us.us.us.us.us:                          ; preds = %col.loop.preheader.us.us.preheader
  %lval_0.us.us.us.us.us = load i64, i64* %arr1, align 8
  %rval_0.us.us.us.us.us = load i64, i64* %arr2, align 8
  %prod_0.us.us.us.us.us = mul i64 %rval_0.us.us.us.us.us, %lval_0.us.us.us.us.us
  store i64 %prod_0.us.us.us.us.us, i64* %dest, align 8
  br label %row.done

col.loop.preheader.us.us:                         ; preds = %col.loop.preheader.us.us.preheader, %col.loop.col.done_crit_edge.split.us.us.us
  %row_17.us.us = phi i64 [ %row_2.us.us, %col.loop.col.done_crit_edge.split.us.us.us ], [ 0, %col.loop.preheader.us.us.preheader ]
  %lidx_0.us.us = mul i64 %row_17.us.us, %size
  br label %sum.loop.preheader.us.us.us

sum.loop.preheader.us.us.us:                      ; preds = %sum.loop.sum.done_crit_edge.us.us.us, %col.loop.preheader.us.us
  %col_15.us.us.us = phi i64 [ 0, %col.loop.preheader.us.us ], [ %col_2.us.us.us, %sum.loop.sum.done_crit_edge.us.us.us ]
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %sum.loop.preheader.us.us.us
  %index = phi i64 [ 0, %sum.loop.preheader.us.us.us ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 0, %sum.loop.preheader.us.us.us ], [ %16, %vector.body ]
  %vec.phi18 = phi i64 [ 0, %sum.loop.preheader.us.us.us ], [ %17, %vector.body ]
  %induction19 = or i64 %index, 1
  %0 = add i64 %index, %lidx_0.us.us
  %1 = add i64 %induction19, %lidx_0.us.us
  %2 = mul i64 %index, %size
  %3 = mul i64 %induction19, %size
  %4 = add i64 %2, %col_15.us.us.us
  %5 = add i64 %3, %col_15.us.us.us
  %6 = getelementptr inbounds i64, i64* %arr1, i64 %0
  %7 = getelementptr inbounds i64, i64* %arr1, i64 %1
  %8 = load i64, i64* %6, align 8
  %9 = load i64, i64* %7, align 8
  %10 = getelementptr inbounds i64, i64* %arr2, i64 %4
  %11 = getelementptr inbounds i64, i64* %arr2, i64 %5
  %12 = load i64, i64* %10, align 8
  %13 = load i64, i64* %11, align 8
  %14 = mul i64 %12, %8
  %15 = mul i64 %13, %9
  %16 = add i64 %14, %vec.phi
  %17 = add i64 %15, %vec.phi18
  %index.next = add nuw i64 %index, 2
  %18 = icmp eq i64 %index.next, %n.vec
  br i1 %18, label %middle.block, label %vector.body, !llvm.loop !0

middle.block:                                     ; preds = %vector.body
  %bin.rdx = add i64 %17, %16
  br i1 %cmp.n, label %sum.loop.sum.done_crit_edge.us.us.us, label %sum.body.us.us.us

sum.body.us.us.us:                                ; preds = %middle.block, %sum.body.us.us.us
  %sum_13.us.us.us = phi i64 [ %sum_2.us.us.us, %sum.body.us.us.us ], [ %bin.rdx, %middle.block ]
  %i_12.us.us.us = phi i64 [ %i_2.us.us.us, %sum.body.us.us.us ], [ %n.vec, %middle.block ]
  %lidx_1.us.us.us = add i64 %i_12.us.us.us, %lidx_0.us.us
  %ridx_0.us.us.us = mul i64 %i_12.us.us.us, %size
  %ridx_1.us.us.us = add i64 %ridx_0.us.us.us, %col_15.us.us.us
  %lvalloc_0.us.us.us = getelementptr inbounds i64, i64* %arr1, i64 %lidx_1.us.us.us
  %lval_0.us.us.us = load i64, i64* %lvalloc_0.us.us.us, align 8
  %rvalloc_0.us.us.us = getelementptr inbounds i64, i64* %arr2, i64 %ridx_1.us.us.us
  %rval_0.us.us.us = load i64, i64* %rvalloc_0.us.us.us, align 8
  %prod_0.us.us.us = mul i64 %rval_0.us.us.us, %lval_0.us.us.us
  %sum_2.us.us.us = add i64 %prod_0.us.us.us, %sum_13.us.us.us
  %i_2.us.us.us = add nuw nsw i64 %i_12.us.us.us, 1
  %exitcond.not = icmp eq i64 %i_2.us.us.us, %size
  br i1 %exitcond.not, label %sum.loop.sum.done_crit_edge.us.us.us, label %sum.body.us.us.us, !llvm.loop !2

sum.loop.sum.done_crit_edge.us.us.us:             ; preds = %sum.body.us.us.us, %middle.block
  %sum_2.us.us.us.lcssa = phi i64 [ %bin.rdx, %middle.block ], [ %sum_2.us.us.us, %sum.body.us.us.us ]
  %idx_1.us.us.us = add i64 %col_15.us.us.us, %lidx_0.us.us
  %loc_0.us.us.us = getelementptr inbounds i64, i64* %dest, i64 %idx_1.us.us.us
  store i64 %sum_2.us.us.us.lcssa, i64* %loc_0.us.us.us, align 8
  %col_2.us.us.us = add nuw nsw i64 %col_15.us.us.us, 1
  %exitcond16.not = icmp eq i64 %col_2.us.us.us, %size
  br i1 %exitcond16.not, label %col.loop.col.done_crit_edge.split.us.us.us, label %sum.loop.preheader.us.us.us

col.loop.col.done_crit_edge.split.us.us.us:       ; preds = %sum.loop.sum.done_crit_edge.us.us.us
  %row_2.us.us = add nuw nsw i64 %row_17.us.us, 1
  %exitcond17.not = icmp eq i64 %row_2.us.us, %size
  br i1 %exitcond17.not, label %row.done, label %col.loop.preheader.us.us

row.done:                                         ; preds = %col.loop.col.done_crit_edge.split.us.us.us, %sum.body.us.us.us.us.us, %pre_entry
  ret void
}

; Function Attrs: nounwind
define dso_local void @__main() local_unnamed_addr #6 {
b0:
  br label %loop_body

loop_body:                                        ; preds = %b0, %loop_body
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %loop_body ]
  tail call void @__orig_main(i64 %loop_counter_11)
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 130
  br i1 %exitcond.not, label %loop_done, label %loop_body

loop_done:                                        ; preds = %loop_body
  ret void
}

; Function Attrs: nounwind
define dso_local void @__orig_main(i64 %size) local_unnamed_addr #6 {
pre_entry:
  %sqsize_0 = mul i64 %size, %size
  %z0.i = shl i64 %sqsize_0, 3
  %z1.i = tail call i8* @malloc(i64 %z0.i) #6
  %arr_0.i = bitcast i8* %z1.i to i64*
  %cond_01.i = icmp sgt i64 %sqsize_0, 0
  br i1 %cond_01.i, label %body.i, label %__randarray.exit17.thread

body.i:                                           ; preds = %pre_entry, %body.i
  %next_2.i3.i = phi i64 [ %next_2.i.i, %body.i ], [ 109658, %pre_entry ]
  %i_12.i = phi i64 [ %i_2.i, %body.i ], [ 0, %pre_entry ]
  %ax_0.i.i = mul i64 %next_2.i3.i, 25214903917
  %axpc_0.i.i = add i64 %ax_0.i.i, 11
  %next_2.i.i = srem i64 %axpc_0.i.i, 281474976710656
  %0 = srem i64 %next_2.i.i, 1000
  %loc_0.i = getelementptr inbounds i64, i64* %arr_0.i, i64 %i_12.i
  store i64 %0, i64* %loc_0.i, align 8
  %i_2.i = add nuw nsw i64 %i_12.i, 1
  %exitcond.not.i = icmp eq i64 %i_2.i, %sqsize_0
  br i1 %exitcond.not.i, label %body.lr.ph.i6, label %body.i

body.lr.ph.i6:                                    ; preds = %body.i
  %z1.i2 = tail call i8* @malloc(i64 %z0.i) #6
  %arr_0.i3 = bitcast i8* %z1.i2 to i64*
  br label %body.i15

body.i15:                                         ; preds = %body.i15, %body.lr.ph.i6
  %next_2.i3.i7 = phi i64 [ %next_2.i.i, %body.lr.ph.i6 ], [ %next_2.i.i11, %body.i15 ]
  %i_12.i8 = phi i64 [ 0, %body.lr.ph.i6 ], [ %i_2.i13, %body.i15 ]
  %ax_0.i.i9 = mul i64 %next_2.i3.i7, 25214903917
  %axpc_0.i.i10 = add i64 %ax_0.i.i9, 11
  %next_2.i.i11 = srem i64 %axpc_0.i.i10, 281474976710656
  %1 = srem i64 %next_2.i.i11, 1000
  %loc_0.i12 = getelementptr inbounds i64, i64* %arr_0.i3, i64 %i_12.i8
  store i64 %1, i64* %loc_0.i12, align 8
  %i_2.i13 = add nuw nsw i64 %i_12.i8, 1
  %exitcond.not.i14 = icmp eq i64 %i_2.i13, %sqsize_0
  br i1 %exitcond.not.i14, label %body.lr.ph.i23, label %body.i15

__randarray.exit17.thread:                        ; preds = %pre_entry
  %z1.i258 = tail call i8* @malloc(i64 %z0.i) #6
  %arr_0.i359 = bitcast i8* %z1.i258 to i64*
  %z1.i1965 = tail call i8* @malloc(i64 %z0.i) #6
  %arr_0.i2066 = bitcast i8* %z1.i1965 to i64*
  br label %__randarray.exit34

body.lr.ph.i23:                                   ; preds = %body.i15
  %z1.i19 = tail call i8* @malloc(i64 %z0.i) #6
  %arr_0.i20 = bitcast i8* %z1.i19 to i64*
  br label %body.i32

body.i32:                                         ; preds = %body.i32, %body.lr.ph.i23
  %next_2.i3.i24 = phi i64 [ %next_2.i.i11, %body.lr.ph.i23 ], [ %next_2.i.i28, %body.i32 ]
  %i_12.i25 = phi i64 [ 0, %body.lr.ph.i23 ], [ %i_2.i30, %body.i32 ]
  %ax_0.i.i26 = mul i64 %next_2.i3.i24, 25214903917
  %axpc_0.i.i27 = add i64 %ax_0.i.i26, 11
  %next_2.i.i28 = srem i64 %axpc_0.i.i27, 281474976710656
  %2 = srem i64 %next_2.i.i28, 1000
  %loc_0.i29 = getelementptr inbounds i64, i64* %arr_0.i20, i64 %i_12.i25
  store i64 %2, i64* %loc_0.i29, align 8
  %i_2.i30 = add nuw nsw i64 %i_12.i25, 1
  %exitcond.not.i31 = icmp eq i64 %i_2.i30, %sqsize_0
  br i1 %exitcond.not.i31, label %__randarray.exit34, label %body.i32

__randarray.exit34:                               ; preds = %body.i32, %__randarray.exit17.thread
  %arr_0.i2070 = phi i64* [ %arr_0.i2066, %__randarray.exit17.thread ], [ %arr_0.i20, %body.i32 ]
  %z1.i1969 = phi i8* [ %z1.i1965, %__randarray.exit17.thread ], [ %z1.i19, %body.i32 ]
  %z1.i26068 = phi i8* [ %z1.i258, %__randarray.exit17.thread ], [ %z1.i2, %body.i32 ]
  %arr_0.i36167 = phi i64* [ %arr_0.i359, %__randarray.exit17.thread ], [ %arr_0.i3, %body.i32 ]
  %cond_06.i = icmp sgt i64 %size, 0
  br i1 %cond_06.i, label %col.loop.preheader.us.us.i.preheader, label %__matmul.exit

col.loop.preheader.us.us.i.preheader:             ; preds = %__randarray.exit34
  %min.iters.check = icmp eq i64 %size, 1
  %n.vec = and i64 %size, -2
  %cmp.n = icmp eq i64 %n.vec, %size
  br i1 %min.iters.check, label %sum.body.us.us.us.i.us.us, label %col.loop.preheader.us.us.i

sum.body.us.us.us.i.us.us:                        ; preds = %col.loop.preheader.us.us.i.preheader
  %lval_0.us.us.us.i.us.us = load i64, i64* %arr_0.i, align 8
  %rval_0.us.us.us.i.us.us = load i64, i64* %arr_0.i36167, align 8
  %prod_0.us.us.us.i.us.us = mul i64 %rval_0.us.us.us.i.us.us, %lval_0.us.us.us.i.us.us
  store i64 %prod_0.us.us.us.i.us.us, i64* %arr_0.i2070, align 8
  br label %__matmul.exit

col.loop.preheader.us.us.i:                       ; preds = %col.loop.preheader.us.us.i.preheader, %col.loop.col.done_crit_edge.split.us.us.us.i
  %row_17.us.us.i = phi i64 [ %row_2.us.us.i, %col.loop.col.done_crit_edge.split.us.us.us.i ], [ 0, %col.loop.preheader.us.us.i.preheader ]
  %lidx_0.us.us.i = mul i64 %row_17.us.us.i, %size
  br label %sum.loop.preheader.us.us.us.i

sum.loop.preheader.us.us.us.i:                    ; preds = %sum.loop.sum.done_crit_edge.us.us.us.i, %col.loop.preheader.us.us.i
  %col_15.us.us.us.i = phi i64 [ 0, %col.loop.preheader.us.us.i ], [ %col_2.us.us.us.i, %sum.loop.sum.done_crit_edge.us.us.us.i ]
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %sum.loop.preheader.us.us.us.i
  %index = phi i64 [ 0, %sum.loop.preheader.us.us.us.i ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 0, %sum.loop.preheader.us.us.us.i ], [ %19, %vector.body ]
  %vec.phi72 = phi i64 [ 0, %sum.loop.preheader.us.us.us.i ], [ %20, %vector.body ]
  %induction73 = or i64 %index, 1
  %3 = add i64 %index, %lidx_0.us.us.i
  %4 = add i64 %induction73, %lidx_0.us.us.i
  %5 = mul i64 %index, %size
  %6 = mul i64 %induction73, %size
  %7 = add i64 %5, %col_15.us.us.us.i
  %8 = add i64 %6, %col_15.us.us.us.i
  %9 = getelementptr inbounds i64, i64* %arr_0.i, i64 %3
  %10 = getelementptr inbounds i64, i64* %arr_0.i, i64 %4
  %11 = load i64, i64* %9, align 8
  %12 = load i64, i64* %10, align 8
  %13 = getelementptr inbounds i64, i64* %arr_0.i36167, i64 %7
  %14 = getelementptr inbounds i64, i64* %arr_0.i36167, i64 %8
  %15 = load i64, i64* %13, align 8
  %16 = load i64, i64* %14, align 8
  %17 = mul i64 %15, %11
  %18 = mul i64 %16, %12
  %19 = add i64 %17, %vec.phi
  %20 = add i64 %18, %vec.phi72
  %index.next = add nuw i64 %index, 2
  %21 = icmp eq i64 %index.next, %n.vec
  br i1 %21, label %middle.block, label %vector.body, !llvm.loop !3

middle.block:                                     ; preds = %vector.body
  %bin.rdx = add i64 %20, %19
  br i1 %cmp.n, label %sum.loop.sum.done_crit_edge.us.us.us.i, label %sum.body.us.us.us.i

sum.body.us.us.us.i:                              ; preds = %middle.block, %sum.body.us.us.us.i
  %sum_13.us.us.us.i = phi i64 [ %sum_2.us.us.us.i, %sum.body.us.us.us.i ], [ %bin.rdx, %middle.block ]
  %i_12.us.us.us.i = phi i64 [ %i_2.us.us.us.i, %sum.body.us.us.us.i ], [ %n.vec, %middle.block ]
  %lidx_1.us.us.us.i = add i64 %i_12.us.us.us.i, %lidx_0.us.us.i
  %ridx_0.us.us.us.i = mul i64 %i_12.us.us.us.i, %size
  %ridx_1.us.us.us.i = add i64 %ridx_0.us.us.us.i, %col_15.us.us.us.i
  %lvalloc_0.us.us.us.i = getelementptr inbounds i64, i64* %arr_0.i, i64 %lidx_1.us.us.us.i
  %lval_0.us.us.us.i = load i64, i64* %lvalloc_0.us.us.us.i, align 8
  %rvalloc_0.us.us.us.i = getelementptr inbounds i64, i64* %arr_0.i36167, i64 %ridx_1.us.us.us.i
  %rval_0.us.us.us.i = load i64, i64* %rvalloc_0.us.us.us.i, align 8
  %prod_0.us.us.us.i = mul i64 %rval_0.us.us.us.i, %lval_0.us.us.us.i
  %sum_2.us.us.us.i = add i64 %prod_0.us.us.us.i, %sum_13.us.us.us.i
  %i_2.us.us.us.i = add nuw nsw i64 %i_12.us.us.us.i, 1
  %exitcond.not.i35 = icmp eq i64 %i_2.us.us.us.i, %size
  br i1 %exitcond.not.i35, label %sum.loop.sum.done_crit_edge.us.us.us.i, label %sum.body.us.us.us.i, !llvm.loop !4

sum.loop.sum.done_crit_edge.us.us.us.i:           ; preds = %sum.body.us.us.us.i, %middle.block
  %sum_2.us.us.us.i.lcssa = phi i64 [ %bin.rdx, %middle.block ], [ %sum_2.us.us.us.i, %sum.body.us.us.us.i ]
  %idx_1.us.us.us.i = add i64 %col_15.us.us.us.i, %lidx_0.us.us.i
  %loc_0.us.us.us.i = getelementptr inbounds i64, i64* %arr_0.i2070, i64 %idx_1.us.us.us.i
  store i64 %sum_2.us.us.us.i.lcssa, i64* %loc_0.us.us.us.i, align 8
  %col_2.us.us.us.i = add nuw nsw i64 %col_15.us.us.us.i, 1
  %exitcond16.not.i = icmp eq i64 %col_2.us.us.us.i, %size
  br i1 %exitcond16.not.i, label %col.loop.col.done_crit_edge.split.us.us.us.i, label %sum.loop.preheader.us.us.us.i

col.loop.col.done_crit_edge.split.us.us.us.i:     ; preds = %sum.loop.sum.done_crit_edge.us.us.us.i
  %row_2.us.us.i = add nuw nsw i64 %row_17.us.us.i, 1
  %exitcond17.not.i = icmp eq i64 %row_2.us.us.i, %size
  br i1 %exitcond17.not.i, label %__matmul.exit, label %col.loop.preheader.us.us.i

__matmul.exit:                                    ; preds = %col.loop.col.done_crit_edge.split.us.us.us.i, %sum.body.us.us.us.i.us.us, %__randarray.exit34
  br i1 %cond_01.i, label %body.i41, label %__printarray.exit57

body.i41:                                         ; preds = %__matmul.exit, %body.i41
  %i_12.i37 = phi i64 [ %i_2.i39, %body.i41 ], [ 0, %__matmul.exit ]
  %loc_0.i38 = getelementptr inbounds i64, i64* %arr_0.i, i64 %i_12.i37
  %val_0.i = load i64, i64* %loc_0.i38, align 8
  %22 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0.i) #6
  %23 = tail call i32 @putchar(i32 10) #6
  %i_2.i39 = add nuw nsw i64 %i_12.i37, 1
  %exitcond.not.i40 = icmp eq i64 %i_2.i39, %sqsize_0
  br i1 %exitcond.not.i40, label %body.i48, label %body.i41

body.i48:                                         ; preds = %body.i41, %body.i48
  %i_12.i43 = phi i64 [ %i_2.i46, %body.i48 ], [ 0, %body.i41 ]
  %loc_0.i44 = getelementptr inbounds i64, i64* %arr_0.i36167, i64 %i_12.i43
  %val_0.i45 = load i64, i64* %loc_0.i44, align 8
  %24 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0.i45) #6
  %25 = tail call i32 @putchar(i32 10) #6
  %i_2.i46 = add nuw nsw i64 %i_12.i43, 1
  %exitcond.not.i47 = icmp eq i64 %i_2.i46, %sqsize_0
  br i1 %exitcond.not.i47, label %body.i56, label %body.i48

body.i56:                                         ; preds = %body.i48, %body.i56
  %i_12.i51 = phi i64 [ %i_2.i54, %body.i56 ], [ 0, %body.i48 ]
  %loc_0.i52 = getelementptr inbounds i64, i64* %arr_0.i2070, i64 %i_12.i51
  %val_0.i53 = load i64, i64* %loc_0.i52, align 8
  %26 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0.i53) #6
  %27 = tail call i32 @putchar(i32 10) #6
  %i_2.i54 = add nuw nsw i64 %i_12.i51, 1
  %exitcond.not.i55 = icmp eq i64 %i_2.i54, %sqsize_0
  br i1 %exitcond.not.i55, label %__printarray.exit57, label %body.i56

__printarray.exit57:                              ; preds = %body.i56, %__matmul.exit
  tail call void @free(i8* %z1.i)
  tail call void @free(i8* %z1.i26068)
  tail call void @free(i8* %z1.i1969)
  ret void
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %loop_body.i, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #8
  ret i32 0

loop_body.i:                                      ; preds = %0, %loop_body.i
  %loop_counter_11.i = phi i64 [ %loop_counter_2.i, %loop_body.i ], [ 10, %0 ]
  tail call void @__orig_main(i64 %loop_counter_11.i) #6
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 130
  br i1 %exitcond.not.i, label %__main.exit, label %loop_body.i

__main.exit:                                      ; preds = %loop_body.i
  ret i32 0
}

; Function Attrs: cold minsize noreturn
define internal void @main.cold.1(i32 %0) #7 {
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
attributes #4 = { argmemonly mustprogress nofree norecurse nosync nounwind willreturn }
attributes #5 = { argmemonly nofree norecurse nosync nounwind }
attributes #6 = { nounwind }
attributes #7 = { cold minsize noreturn }
attributes #8 = { noinline }

!0 = distinct !{!0, !1}
!1 = !{!"llvm.loop.isvectorized", i32 1}
!2 = distinct !{!2, !1}
!3 = distinct !{!3, !1}
!4 = distinct !{!4, !1}
