; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpTHxLbK/compile.ll'
source_filename = "stdin"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"
target triple = "arm64-apple-macosx13.0.0"

@.str = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.str.1 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@.str.2 = private unnamed_addr constant [4 x i8] c"%ld\00", align 1
@.str.3 = private unnamed_addr constant [9 x i8] c"[object]\00", align 1
@.str.4 = private unnamed_addr constant [33 x i8] c"error: expected %d args, got %d\0A\00", align 1
@.memset_pattern = private unnamed_addr constant [2 x i64] [i64 -999, i64 -999], align 16

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
  %z1.i = tail call dereferenceable_or_null(384000) i8* @malloc(i64 384000) #4
  %ptr_0.i = bitcast i8* %z1.i to i64*
  %z1.i1 = tail call dereferenceable_or_null(320000) i8* @malloc(i64 320000) #4
  %ptr_0.i2 = bitcast i8* %z1.i1 to i64*
  %z1.i3 = tail call dereferenceable_or_null(384000) i8* @malloc(i64 384000) #4
  %ptr_0.i4 = bitcast i8* %z1.i3 to i64*
  tail call void @__init(i64* %ptr_0.i, i64* %ptr_0.i2, i64* %ptr_0.i4, i64 200, i64 200, i64 240, i64 240)
  br label %main_j.preheader

main_j.preheader:                                 ; preds = %b0, %main_j_done
  %i_141 = phi i64 [ 0, %b0 ], [ %i_2, %main_j_done ]
  %cond_437.not = icmp eq i64 %i_141, 0
  %row_offset_0.i.i21 = mul nuw nsw i64 %i_141, 240
  %offset_0.i.i30 = mul nuw nsw i64 %i_141, 201
  %new_ptr_0.i.i31 = getelementptr inbounds i64, i64* %ptr_0.i2, i64 %offset_0.i.i30
  %val_0.i32 = load i64, i64* %new_ptr_0.i.i31, align 8
  %row_offset_0.i.i5 = mul nuw nsw i64 %i_141, 200
  br i1 %cond_437.not, label %vector.body, label %main_k.preheader.us.preheader

main_k.preheader.us.preheader:                    ; preds = %main_j.preheader
  %min.iters.check = icmp eq i64 %i_141, 1
  %n.vec = and i64 %i_141, 9223372036854775806
  %cmp.n56 = icmp eq i64 %i_141, %n.vec
  br label %main_k.preheader.us

vector.body:                                      ; preds = %main_j.preheader, %vector.body
  %index = phi i64 [ %index.next, %vector.body ], [ 0, %main_j.preheader ]
  %induction49 = or i64 %index, 1
  %induction50 = or i64 %index, 2
  %induction51 = or i64 %index, 3
  %0 = add nuw nsw i64 %index, %row_offset_0.i.i21
  %1 = add nuw nsw i64 %induction49, %row_offset_0.i.i21
  %2 = add nuw nsw i64 %induction50, %row_offset_0.i.i21
  %3 = add nuw nsw i64 %induction51, %row_offset_0.i.i21
  %4 = getelementptr inbounds i64, i64* %ptr_0.i, i64 %0
  %5 = getelementptr inbounds i64, i64* %ptr_0.i, i64 %1
  %6 = getelementptr inbounds i64, i64* %ptr_0.i, i64 %2
  %7 = getelementptr inbounds i64, i64* %ptr_0.i, i64 %3
  %8 = load i64, i64* %4, align 8
  %9 = load i64, i64* %5, align 8
  %10 = load i64, i64* %6, align 8
  %11 = load i64, i64* %7, align 8
  %12 = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %0
  %13 = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %1
  %14 = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %2
  %15 = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %3
  %16 = load i64, i64* %12, align 8
  %17 = load i64, i64* %13, align 8
  %18 = load i64, i64* %14, align 8
  %19 = load i64, i64* %15, align 8
  %20 = shl i64 %8, 1
  %21 = shl i64 %9, 1
  %22 = shl i64 %10, 1
  %23 = shl i64 %11, 1
  %24 = mul i64 %val_0.i32, %16
  %25 = mul i64 %val_0.i32, %17
  %26 = mul i64 %val_0.i32, %18
  %27 = mul i64 %val_0.i32, %19
  %28 = mul i64 %24, 3
  %29 = mul i64 %25, 3
  %30 = mul i64 %26, 3
  %31 = mul i64 %27, 3
  %32 = add i64 %28, %20
  %33 = add i64 %29, %21
  %34 = add i64 %30, %22
  %35 = add i64 %31, %23
  store i64 %32, i64* %4, align 8
  store i64 %33, i64* %5, align 8
  store i64 %34, i64* %6, align 8
  store i64 %35, i64* %7, align 8
  %index.next = add nuw i64 %index, 4
  %36 = icmp eq i64 %index.next, 240
  br i1 %36, label %main_j_done, label %vector.body, !llvm.loop !0

main_k.preheader.us:                              ; preds = %main_k.preheader.us.preheader, %main_k.main_k_done_crit_edge.us
  %j_140.us = phi i64 [ %j_2.us, %main_k.main_k_done_crit_edge.us ], [ 0, %main_k.preheader.us.preheader ]
  %offset_0.i.i.us = add nuw nsw i64 %j_140.us, %row_offset_0.i.i21
  %new_ptr_0.i.i.us = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %offset_0.i.i.us
  %val_0.i.us = load i64, i64* %new_ptr_0.i.i.us, align 8
  %incr_0.us = mul i64 %val_0.i.us, 3
  br i1 %min.iters.check, label %main_k_body.us.preheader, label %vector.body57

vector.body57:                                    ; preds = %main_k.preheader.us, %vector.body57
  %index58 = phi i64 [ %index.next62, %vector.body57 ], [ 0, %main_k.preheader.us ]
  %vec.phi = phi i64 [ %61, %vector.body57 ], [ 0, %main_k.preheader.us ]
  %vec.phi59 = phi i64 [ %62, %vector.body57 ], [ 0, %main_k.preheader.us ]
  %induction61 = or i64 %index58, 1
  %37 = add nuw nsw i64 %index58, %row_offset_0.i.i5
  %38 = add nuw nsw i64 %induction61, %row_offset_0.i.i5
  %39 = getelementptr inbounds i64, i64* %ptr_0.i2, i64 %37
  %40 = getelementptr inbounds i64, i64* %ptr_0.i2, i64 %38
  %41 = load i64, i64* %39, align 8
  %42 = load i64, i64* %40, align 8
  %43 = mul i64 %incr_0.us, %41
  %44 = mul i64 %incr_0.us, %42
  %45 = mul nuw nsw i64 %index58, 240
  %46 = mul nuw nsw i64 %induction61, 240
  %47 = add nuw nsw i64 %45, %j_140.us
  %48 = add nuw nsw i64 %46, %j_140.us
  %49 = getelementptr inbounds i64, i64* %ptr_0.i, i64 %47
  %50 = getelementptr inbounds i64, i64* %ptr_0.i, i64 %48
  %51 = load i64, i64* %49, align 8
  %52 = load i64, i64* %50, align 8
  %53 = add i64 %51, %43
  %54 = add i64 %52, %44
  store i64 %53, i64* %49, align 8
  store i64 %54, i64* %50, align 8
  %55 = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %47
  %56 = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %48
  %57 = load i64, i64* %55, align 8
  %58 = load i64, i64* %56, align 8
  %59 = mul i64 %57, %41
  %60 = mul i64 %58, %42
  %61 = add i64 %59, %vec.phi
  %62 = add i64 %60, %vec.phi59
  %index.next62 = add nuw i64 %index58, 2
  %63 = icmp eq i64 %index.next62, %n.vec
  br i1 %63, label %middle.block52, label %vector.body57, !llvm.loop !2

middle.block52:                                   ; preds = %vector.body57
  %bin.rdx = add i64 %62, %61
  br i1 %cmp.n56, label %main_k.main_k_done_crit_edge.us, label %main_k_body.us.preheader

main_k_body.us.preheader:                         ; preds = %main_k.preheader.us, %middle.block52
  %temp2_139.us.ph = phi i64 [ 0, %main_k.preheader.us ], [ %bin.rdx, %middle.block52 ]
  %k_138.us.ph = phi i64 [ 0, %main_k.preheader.us ], [ %n.vec, %middle.block52 ]
  br label %main_k_body.us

main_k_body.us:                                   ; preds = %main_k_body.us.preheader, %main_k_body.us
  %temp2_139.us = phi i64 [ %temp2_2.us, %main_k_body.us ], [ %temp2_139.us.ph, %main_k_body.us.preheader ]
  %k_138.us = phi i64 [ %k_2.us, %main_k_body.us ], [ %k_138.us.ph, %main_k_body.us.preheader ]
  %offset_0.i.i6.us = add nuw nsw i64 %k_138.us, %row_offset_0.i.i5
  %new_ptr_0.i.i7.us = getelementptr inbounds i64, i64* %ptr_0.i2, i64 %offset_0.i.i6.us
  %val_0.i8.us = load i64, i64* %new_ptr_0.i.i7.us, align 8
  %incr_1.us = mul i64 %incr_0.us, %val_0.i8.us
  %row_offset_0.i.i9.us = mul nuw nsw i64 %k_138.us, 240
  %offset_0.i.i10.us = add nuw nsw i64 %row_offset_0.i.i9.us, %j_140.us
  %new_ptr_0.i.i11.us = getelementptr inbounds i64, i64* %ptr_0.i, i64 %offset_0.i.i10.us
  %val_0.i12.us = load i64, i64* %new_ptr_0.i.i11.us, align 8
  %new_val_0.i.us = add i64 %val_0.i12.us, %incr_1.us
  store i64 %new_val_0.i.us, i64* %new_ptr_0.i.i11.us, align 8
  %new_ptr_0.i.i15.us = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %offset_0.i.i10.us
  %val_0.i16.us = load i64, i64* %new_ptr_0.i.i15.us, align 8
  %incr_2.us = mul i64 %val_0.i16.us, %val_0.i8.us
  %temp2_2.us = add i64 %incr_2.us, %temp2_139.us
  %k_2.us = add nuw nsw i64 %k_138.us, 1
  %exitcond44.not = icmp eq i64 %k_2.us, %i_141
  br i1 %exitcond44.not, label %main_k.main_k_done_crit_edge.us, label %main_k_body.us, !llvm.loop !3

main_k.main_k_done_crit_edge.us:                  ; preds = %main_k_body.us, %middle.block52
  %temp2_2.us.lcssa = phi i64 [ %bin.rdx, %middle.block52 ], [ %temp2_2.us, %main_k_body.us ]
  %new_ptr_0.i.i23.us = getelementptr inbounds i64, i64* %ptr_0.i, i64 %offset_0.i.i.us
  %val_0.i24.us = load i64, i64* %new_ptr_0.i.i23.us, align 8
  %val1_0.us = shl i64 %val_0.i24.us, 1
  %val2_1.us = mul i64 %val_0.i32, %val_0.i.us
  %reass.add.us = add i64 %val2_1.us, %temp2_2.us.lcssa
  %reass.mul.us = mul i64 %reass.add.us, 3
  %val_1.us = add i64 %reass.mul.us, %val1_0.us
  store i64 %val_1.us, i64* %new_ptr_0.i.i23.us, align 8
  %j_2.us = add nuw nsw i64 %j_140.us, 1
  %exitcond45.not = icmp eq i64 %j_2.us, 240
  br i1 %exitcond45.not, label %main_j_done, label %main_k.preheader.us

main_j_done:                                      ; preds = %main_k.main_k_done_crit_edge.us, %vector.body
  %i_2 = add nuw nsw i64 %i_141, 1
  %exitcond46.not = icmp eq i64 %i_2, 200
  br i1 %exitcond46.not, label %body.i, label %main_j.preheader

body.i:                                           ; preds = %main_j_done, %body.i
  %i_12.i = phi i64 [ %i_2.i, %body.i ], [ 0, %main_j_done ]
  %mtx_loc_0.i = getelementptr inbounds i64, i64* %ptr_0.i, i64 %i_12.i
  %val_0.i36 = load i64, i64* %mtx_loc_0.i, align 8
  %64 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0.i36) #4
  %65 = tail call i32 @putchar(i32 10) #4
  %i_2.i = add nuw nsw i64 %i_12.i, 1
  %exitcond.not.i = icmp eq i64 %i_2.i, 48000
  br i1 %exitcond.not.i, label %__matrix_print.exit, label %body.i

__matrix_print.exit:                              ; preds = %body.i
  tail call void @free(i8* nonnull %z1.i)
  tail call void @free(i8* %z1.i1)
  tail call void @free(i8* %z1.i3)
  ret void
}

; Function Attrs: argmemonly nofree norecurse nosync nounwind writeonly
define dso_local void @__init(i64* nocapture writeonly %C, i64* nocapture writeonly %A, i64* nocapture writeonly %B, i64 %M, i64 %fM, i64 %N, i64 %fN) local_unnamed_addr #5 {
pre_entry:
  %cond_041 = icmp sgt i64 %M, 0
  br i1 %cond_041, label %init_CB_j.preheader.lr.ph, label %init_A_i_done

init_CB_j.preheader.lr.ph:                        ; preds = %pre_entry
  %cond_238 = icmp sgt i64 %N, 0
  br i1 %cond_238, label %init_CB_j.preheader.us, label %init_A_j1.preheader.preheader

init_CB_j.preheader.us:                           ; preds = %init_CB_j.preheader.lr.ph, %init_CB_j.init_CB_j_done_crit_edge.us
  %i_143.us = phi i64 [ %i_2.us, %init_CB_j.init_CB_j_done_crit_edge.us ], [ 0, %init_CB_j.preheader.lr.ph ]
  %row_offset_0.i.i.us = mul i64 %i_143.us, %N
  %val_3.us = add i64 %i_143.us, %fN
  br label %init_CB_j_body.us

init_CB_j_body.us:                                ; preds = %init_CB_j.preheader.us, %__fmod.exit14.us
  %j_140.us = phi i64 [ 0, %init_CB_j.preheader.us ], [ %j_2.us, %__fmod.exit14.us ]
  %val_0.us = add nuw i64 %j_140.us, %i_143.us
  %cond_0.not1.i.us = icmp slt i64 %val_0.us, 100
  br i1 %cond_0.not1.i.us, label %__fmod.exit.us, label %while_inner.preheader.i.us

while_inner.preheader.i.us:                       ; preds = %init_CB_j_body.us, %done_inner.i.us
  %rem_12.i.us = phi i64 [ %rem_2.i.us, %done_inner.i.us ], [ %val_0.us, %init_CB_j_body.us ]
  br label %while_inner.i.us

while_inner.i.us:                                 ; preds = %while_inner.i.us, %while_inner.preheader.i.us
  %decr_1.i.us = phi i64 [ %decr_2.i.us, %while_inner.i.us ], [ 100, %while_inner.preheader.i.us ]
  %diff_0.i.us = sub i64 %rem_12.i.us, %decr_1.i.us
  %cond_2.i.us = icmp sgt i64 %diff_0.i.us, -1
  %decr_2.i.us = shl i64 %decr_1.i.us, 1
  br i1 %cond_2.i.us, label %while_inner.i.us, label %done_inner.i.us

done_inner.i.us:                                  ; preds = %while_inner.i.us
  %decr_3.neg.i.us = sdiv i64 %decr_1.i.us, -2
  %rem_2.i.us = add i64 %decr_3.neg.i.us, %rem_12.i.us
  %cond_0.not.i.us = icmp slt i64 %rem_2.i.us, 100
  br i1 %cond_0.not.i.us, label %__fmod.exit.us, label %while_inner.preheader.i.us

__fmod.exit.us:                                   ; preds = %done_inner.i.us, %init_CB_j_body.us
  %rem_1.lcssa.i.us = phi i64 [ %val_0.us, %init_CB_j_body.us ], [ %rem_2.i.us, %done_inner.i.us ]
  %val_2.us = sdiv i64 %rem_1.lcssa.i.us, %fM
  %offset_0.i.i.us = add i64 %j_140.us, %row_offset_0.i.i.us
  %new_ptr_0.i.i.us = getelementptr inbounds i64, i64* %C, i64 %offset_0.i.i.us
  store i64 %val_2.us, i64* %new_ptr_0.i.i.us, align 8
  %val_4.us = sub i64 %val_3.us, %j_140.us
  %cond_0.not1.i1.us = icmp slt i64 %val_4.us, 100
  br i1 %cond_0.not1.i1.us, label %__fmod.exit14.us, label %while_inner.preheader.i3.us

while_inner.preheader.i3.us:                      ; preds = %__fmod.exit.us, %done_inner.i12.us
  %rem_12.i2.us = phi i64 [ %rem_2.i10.us, %done_inner.i12.us ], [ %val_4.us, %__fmod.exit.us ]
  br label %while_inner.i8.us

while_inner.i8.us:                                ; preds = %while_inner.i8.us, %while_inner.preheader.i3.us
  %decr_1.i4.us = phi i64 [ %decr_2.i7.us, %while_inner.i8.us ], [ 100, %while_inner.preheader.i3.us ]
  %diff_0.i5.us = sub i64 %rem_12.i2.us, %decr_1.i4.us
  %cond_2.i6.us = icmp sgt i64 %diff_0.i5.us, -1
  %decr_2.i7.us = shl i64 %decr_1.i4.us, 1
  br i1 %cond_2.i6.us, label %while_inner.i8.us, label %done_inner.i12.us

done_inner.i12.us:                                ; preds = %while_inner.i8.us
  %decr_3.neg.i9.us = sdiv i64 %decr_1.i4.us, -2
  %rem_2.i10.us = add i64 %decr_3.neg.i9.us, %rem_12.i2.us
  %cond_0.not.i11.us = icmp slt i64 %rem_2.i10.us, 100
  br i1 %cond_0.not.i11.us, label %__fmod.exit14.us, label %while_inner.preheader.i3.us

__fmod.exit14.us:                                 ; preds = %done_inner.i12.us, %__fmod.exit.us
  %rem_1.lcssa.i13.us = phi i64 [ %val_4.us, %__fmod.exit.us ], [ %rem_2.i10.us, %done_inner.i12.us ]
  %val_6.us = sdiv i64 %rem_1.lcssa.i13.us, %fM
  %new_ptr_0.i.i17.us = getelementptr inbounds i64, i64* %B, i64 %offset_0.i.i.us
  store i64 %val_6.us, i64* %new_ptr_0.i.i17.us, align 8
  %j_2.us = add nuw nsw i64 %j_140.us, 1
  %exitcond.not = icmp eq i64 %j_2.us, %N
  br i1 %exitcond.not, label %init_CB_j.init_CB_j_done_crit_edge.us, label %init_CB_j_body.us

init_CB_j.init_CB_j_done_crit_edge.us:            ; preds = %__fmod.exit14.us
  %i_2.us = add nuw nsw i64 %i_143.us, 1
  %exitcond53.not = icmp eq i64 %i_2.us, %M
  br i1 %exitcond53.not, label %init_A_j1.preheader.preheader, label %init_CB_j.preheader.us

init_A_j1.preheader.preheader:                    ; preds = %init_CB_j.init_CB_j_done_crit_edge.us, %init_CB_j.preheader.lr.ph
  %scevgep = getelementptr i64, i64* %A, i64 1
  %scevgep55 = bitcast i64* %scevgep to i8*
  %0 = shl i64 %M, 3
  %1 = add i64 %0, 8
  %2 = add i64 %0, -8
  br label %init_A_j1.preheader

init_A_j1.preheader:                              ; preds = %init_A_j1.preheader.preheader, %init_A_j2_done
  %indvars.iv = phi i64 [ 1, %init_A_j1.preheader.preheader ], [ %indvars.iv.next, %init_A_j2_done ]
  %i_451 = phi i64 [ 0, %init_A_j1.preheader.preheader ], [ %j_7, %init_A_j2_done ]
  %3 = mul i64 %1, %i_451
  %uglygep = getelementptr i8, i8* %scevgep55, i64 %3
  %4 = mul i64 %i_451, -8
  %5 = add i64 %2, %4
  %row_offset_0.i.i32 = mul i64 %i_451, %M
  br label %init_A_j1_body

init_A_j1_body:                                   ; preds = %init_A_j1.preheader, %__fmod.exit31
  %j_546 = phi i64 [ 0, %init_A_j1.preheader ], [ %j_6, %__fmod.exit31 ]
  %val_7 = add nuw i64 %j_546, %i_451
  %cond_0.not1.i18 = icmp slt i64 %val_7, 100
  br i1 %cond_0.not1.i18, label %__fmod.exit31, label %while_inner.preheader.i20

while_inner.preheader.i20:                        ; preds = %init_A_j1_body, %done_inner.i29
  %rem_12.i19 = phi i64 [ %rem_2.i27, %done_inner.i29 ], [ %val_7, %init_A_j1_body ]
  br label %while_inner.i25

while_inner.i25:                                  ; preds = %while_inner.i25, %while_inner.preheader.i20
  %decr_1.i21 = phi i64 [ %decr_2.i24, %while_inner.i25 ], [ 100, %while_inner.preheader.i20 ]
  %diff_0.i22 = sub i64 %rem_12.i19, %decr_1.i21
  %cond_2.i23 = icmp sgt i64 %diff_0.i22, -1
  %decr_2.i24 = shl i64 %decr_1.i21, 1
  br i1 %cond_2.i23, label %while_inner.i25, label %done_inner.i29

done_inner.i29:                                   ; preds = %while_inner.i25
  %decr_3.neg.i26 = sdiv i64 %decr_1.i21, -2
  %rem_2.i27 = add i64 %decr_3.neg.i26, %rem_12.i19
  %cond_0.not.i28 = icmp slt i64 %rem_2.i27, 100
  br i1 %cond_0.not.i28, label %__fmod.exit31, label %while_inner.preheader.i20

__fmod.exit31:                                    ; preds = %done_inner.i29, %init_A_j1_body
  %rem_1.lcssa.i30 = phi i64 [ %val_7, %init_A_j1_body ], [ %rem_2.i27, %done_inner.i29 ]
  %val_9 = sdiv i64 %rem_1.lcssa.i30, %fM
  %offset_0.i.i33 = add i64 %j_546, %row_offset_0.i.i32
  %new_ptr_0.i.i34 = getelementptr inbounds i64, i64* %A, i64 %offset_0.i.i33
  store i64 %val_9, i64* %new_ptr_0.i.i34, align 8
  %j_6 = add nuw nsw i64 %j_546, 1
  %exitcond54 = icmp eq i64 %j_6, %indvars.iv
  br i1 %exitcond54, label %init_A_j1_done, label %init_A_j1_body

init_A_j1_done:                                   ; preds = %__fmod.exit31
  %j_7 = add nuw nsw i64 %i_451, 1
  %cond_847 = icmp slt i64 %j_7, %M
  br i1 %cond_847, label %init_A_j2_body.lr.ph, label %init_A_j2_done

init_A_j2_body.lr.ph:                             ; preds = %init_A_j1_done
  call void @memset_pattern16(i8* %uglygep, i8* bitcast ([2 x i64]* @.memset_pattern to i8*), i64 %5)
  br label %init_A_j2_done

init_A_j2_done:                                   ; preds = %init_A_j2_body.lr.ph, %init_A_j1_done
  %indvars.iv.next = add nuw i64 %indvars.iv, 1
  %exitcond56.not = icmp eq i64 %j_7, %M
  br i1 %exitcond56.not, label %init_A_i_done, label %init_A_j1.preheader

init_A_i_done:                                    ; preds = %init_A_j2_done, %pre_entry
  ret void
}

; Function Attrs: mustprogress nofree nounwind willreturn
define dso_local noalias i64* @__matrix_new(i64 %Nrow, i64 %Ncol) local_unnamed_addr #6 {
pre_entry:
  %total_0 = shl i64 %Nrow, 3
  %z0 = mul i64 %total_0, %Ncol
  %z1 = tail call i8* @malloc(i64 %z0)
  %ptr_0 = bitcast i8* %z1 to i64*
  ret i64* %ptr_0
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind readnone willreturn
define dso_local i64* @__matrix_loc(i64* readnone %mtx, i64 %row, i64 %col, i64 %Ncol) local_unnamed_addr #7 {
pre_entry:
  %row_offset_0 = mul i64 %Ncol, %row
  %offset_0 = add i64 %row_offset_0, %col
  %new_ptr_0 = getelementptr inbounds i64, i64* %mtx, i64 %offset_0
  ret i64* %new_ptr_0
}

; Function Attrs: argmemonly mustprogress nofree norecurse nosync nounwind readonly willreturn
define dso_local i64 @__matrix_get(i64* nocapture readonly %mtx, i64 %row, i64 %col, i64 %Ncol) local_unnamed_addr #3 {
pre_entry:
  %row_offset_0.i = mul i64 %Ncol, %row
  %offset_0.i = add i64 %row_offset_0.i, %col
  %new_ptr_0.i = getelementptr inbounds i64, i64* %mtx, i64 %offset_0.i
  %val_0 = load i64, i64* %new_ptr_0.i, align 8
  ret i64 %val_0
}

; Function Attrs: argmemonly mustprogress nofree norecurse nosync nounwind willreturn writeonly
define dso_local void @__matrix_set(i64* nocapture writeonly %mtx, i64 %row, i64 %col, i64 %Ncol, i64 %val) local_unnamed_addr #8 {
pre_entry:
  %row_offset_0.i = mul i64 %Ncol, %row
  %offset_0.i = add i64 %row_offset_0.i, %col
  %new_ptr_0.i = getelementptr inbounds i64, i64* %mtx, i64 %offset_0.i
  store i64 %val, i64* %new_ptr_0.i, align 8
  ret void
}

; Function Attrs: argmemonly mustprogress nofree norecurse nosync nounwind willreturn
define dso_local void @__matrix_incr(i64* nocapture %mtx, i64 %row, i64 %col, i64 %Ncol, i64 %incr) local_unnamed_addr #9 {
pre_entry:
  %row_offset_0.i = mul i64 %Ncol, %row
  %offset_0.i = add i64 %row_offset_0.i, %col
  %new_ptr_0.i = getelementptr inbounds i64, i64* %mtx, i64 %offset_0.i
  %val_0 = load i64, i64* %new_ptr_0.i, align 8
  %new_val_0 = add i64 %val_0, %incr
  store i64 %new_val_0, i64* %new_ptr_0.i, align 8
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__matrix_print(i64* nocapture readonly %mtx, i64 %Nrow, i64 %Ncol) local_unnamed_addr #0 {
pre_entry:
  %total_0 = mul i64 %Ncol, %Nrow
  %cond_01 = icmp sgt i64 %total_0, 0
  br i1 %cond_01, label %body, label %done

body:                                             ; preds = %pre_entry, %body
  %i_12 = phi i64 [ %i_2, %body ], [ 0, %pre_entry ]
  %mtx_loc_0 = getelementptr inbounds i64, i64* %mtx, i64 %i_12
  %val_0 = load i64, i64* %mtx_loc_0, align 8
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0) #4
  %1 = tail call i32 @putchar(i32 10) #4
  %i_2 = add nuw nsw i64 %i_12, 1
  %exitcond.not = icmp eq i64 %i_2, %total_0
  br i1 %exitcond.not, label %done, label %body

done:                                             ; preds = %body, %pre_entry
  ret void
}

; Function Attrs: nofree norecurse nosync nounwind readnone
define dso_local i64 @__fmod(i64 %n, i64 %m) local_unnamed_addr #10 {
pre_entry:
  %cond_0.not1 = icmp slt i64 %n, %m
  br i1 %cond_0.not1, label %done, label %while_inner.preheader

while_inner.preheader:                            ; preds = %pre_entry, %done_inner
  %rem_12 = phi i64 [ %rem_2, %done_inner ], [ %n, %pre_entry ]
  br label %while_inner

while_inner:                                      ; preds = %while_inner, %while_inner.preheader
  %decr_1 = phi i64 [ %decr_2, %while_inner ], [ %m, %while_inner.preheader ]
  %diff_0 = sub i64 %rem_12, %decr_1
  %cond_2 = icmp sgt i64 %diff_0, -1
  %decr_2 = shl i64 %decr_1, 1
  br i1 %cond_2, label %while_inner, label %done_inner

done_inner:                                       ; preds = %while_inner
  %decr_3.neg = sdiv i64 %decr_1, -2
  %rem_2 = add i64 %decr_3.neg, %rem_12
  %cond_0.not = icmp slt i64 %rem_2, %m
  br i1 %cond_0.not, label %done, label %while_inner.preheader

done:                                             ; preds = %done_inner, %pre_entry
  %rem_1.lcssa = phi i64 [ %n, %pre_entry ], [ %rem_2, %done_inner ]
  ret i64 %rem_1.lcssa
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %2, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #13
  ret i32 0

2:                                                ; preds = %0
  tail call void @__main()
  ret i32 0
}

; Function Attrs: argmemonly nofree nounwind willreturn
declare void @memset_pattern16(i8* nocapture writeonly, i8* nocapture readonly, i64) local_unnamed_addr #11

; Function Attrs: cold minsize noreturn
define internal void @main.cold.1(i32 %0) #12 {
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
attributes #5 = { argmemonly nofree norecurse nosync nounwind writeonly }
attributes #6 = { mustprogress nofree nounwind willreturn }
attributes #7 = { mustprogress nofree norecurse nosync nounwind readnone willreturn }
attributes #8 = { argmemonly mustprogress nofree norecurse nosync nounwind willreturn writeonly }
attributes #9 = { argmemonly mustprogress nofree norecurse nosync nounwind willreturn }
attributes #10 = { nofree norecurse nosync nounwind readnone }
attributes #11 = { argmemonly nofree nounwind willreturn }
attributes #12 = { cold minsize noreturn }
attributes #13 = { noinline }

!0 = distinct !{!0, !1}
!1 = !{!"llvm.loop.isvectorized", i32 1}
!2 = distinct !{!2, !1}
!3 = distinct !{!3, !1}
