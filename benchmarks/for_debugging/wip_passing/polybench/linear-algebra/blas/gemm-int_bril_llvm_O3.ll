; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmphz3dCw/compile.ll'
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
  %z1.i = tail call dereferenceable_or_null(384000) i8* @malloc(i64 384000) #4
  %ptr_0.i = bitcast i8* %z1.i to i64*
  %z1.i1 = tail call dereferenceable_or_null(422400) i8* @malloc(i64 422400) #4
  %ptr_0.i2 = bitcast i8* %z1.i1 to i64*
  %z1.i3 = tail call dereferenceable_or_null(352000) i8* @malloc(i64 352000) #4
  %ptr_0.i4 = bitcast i8* %z1.i3 to i64*
  tail call void @__init(i64* %ptr_0.i, i64* %ptr_0.i2, i64* %ptr_0.i4, i64 200, i64 200, i64 220, i64 220, i64 240, i64 240)
  br label %main_j.preheader

main_j.preheader:                                 ; preds = %b0, %main_k_done
  %i_122 = phi i64 [ 0, %b0 ], [ %i_2, %main_k_done ]
  %row_offset_0.i.i = mul nuw nsw i64 %i_122, 220
  br label %vector.body32

vector.body32:                                    ; preds = %vector.body32, %main_j.preheader
  %index33 = phi i64 [ 0, %main_j.preheader ], [ %index.next38, %vector.body32 ]
  %0 = add nuw nsw i64 %index33, %row_offset_0.i.i
  %1 = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %0
  %2 = bitcast i64* %1 to <2 x i64>*
  %wide.load34 = load <2 x i64>, <2 x i64>* %2, align 8
  %3 = getelementptr inbounds i64, i64* %1, i64 2
  %4 = bitcast i64* %3 to <2 x i64>*
  %wide.load35 = load <2 x i64>, <2 x i64>* %4, align 8
  %5 = getelementptr inbounds i64, i64* %1, i64 4
  %6 = bitcast i64* %5 to <2 x i64>*
  %wide.load36 = load <2 x i64>, <2 x i64>* %6, align 8
  %7 = getelementptr inbounds i64, i64* %1, i64 6
  %8 = bitcast i64* %7 to <2 x i64>*
  %wide.load37 = load <2 x i64>, <2 x i64>* %8, align 8
  %9 = shl <2 x i64> %wide.load34, <i64 1, i64 1>
  %10 = shl <2 x i64> %wide.load35, <i64 1, i64 1>
  %11 = shl <2 x i64> %wide.load36, <i64 1, i64 1>
  %12 = shl <2 x i64> %wide.load37, <i64 1, i64 1>
  store <2 x i64> %9, <2 x i64>* %2, align 8
  store <2 x i64> %10, <2 x i64>* %4, align 8
  store <2 x i64> %11, <2 x i64>* %6, align 8
  store <2 x i64> %12, <2 x i64>* %8, align 8
  %index.next38 = add nuw i64 %index33, 8
  %13 = icmp eq i64 %index.next38, 216
  br i1 %13, label %main_j_body, label %vector.body32, !llvm.loop !0

main_j_body:                                      ; preds = %vector.body32
  %offset_0.i.i = add nuw nsw i64 %row_offset_0.i.i, 216
  %new_ptr_0.i.i = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %offset_0.i.i
  %val_0.i = load i64, i64* %new_ptr_0.i.i, align 8
  %new_val_0.i = shl i64 %val_0.i, 1
  store i64 %new_val_0.i, i64* %new_ptr_0.i.i, align 8
  %offset_0.i.i.1 = add nuw nsw i64 %row_offset_0.i.i, 217
  %new_ptr_0.i.i.1 = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %offset_0.i.i.1
  %val_0.i.1 = load i64, i64* %new_ptr_0.i.i.1, align 8
  %new_val_0.i.1 = shl i64 %val_0.i.1, 1
  store i64 %new_val_0.i.1, i64* %new_ptr_0.i.i.1, align 8
  %offset_0.i.i.2 = add nuw nsw i64 %row_offset_0.i.i, 218
  %new_ptr_0.i.i.2 = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %offset_0.i.i.2
  %val_0.i.2 = load i64, i64* %new_ptr_0.i.i.2, align 8
  %new_val_0.i.2 = shl i64 %val_0.i.2, 1
  store i64 %new_val_0.i.2, i64* %new_ptr_0.i.i.2, align 8
  %offset_0.i.i.3 = add nuw nsw i64 %row_offset_0.i.i, 219
  %new_ptr_0.i.i.3 = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %offset_0.i.i.3
  %val_0.i.3 = load i64, i64* %new_ptr_0.i.i.3, align 8
  %new_val_0.i.3 = shl i64 %val_0.i.3, 1
  store i64 %new_val_0.i.3, i64* %new_ptr_0.i.i.3, align 8
  %row_offset_0.i.i5 = mul nuw nsw i64 %i_122, 240
  br label %inner_j.preheader

inner_j.preheader:                                ; preds = %main_j_body, %inner_j_done
  %k_121 = phi i64 [ 0, %main_j_body ], [ %k_2, %inner_j_done ]
  %row_offset_0.i.i9 = mul nuw nsw i64 %k_121, 220
  %offset_0.i.i6 = add nuw nsw i64 %k_121, %row_offset_0.i.i5
  %new_ptr_0.i.i7 = getelementptr inbounds i64, i64* %ptr_0.i, i64 %offset_0.i.i6
  %val_0.i8 = load i64, i64* %new_ptr_0.i.i7, align 8
  %incr_0 = mul i64 %val_0.i8, 3
  %broadcast.splatinsert = insertelement <2 x i64> poison, i64 %incr_0, i64 0
  %broadcast.splat = shufflevector <2 x i64> %broadcast.splatinsert, <2 x i64> poison, <2 x i32> zeroinitializer
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %inner_j.preheader
  %index = phi i64 [ 0, %inner_j.preheader ], [ %index.next, %vector.body ]
  %14 = add nuw nsw i64 %index, %row_offset_0.i.i9
  %15 = getelementptr inbounds i64, i64* %ptr_0.i2, i64 %14
  %16 = bitcast i64* %15 to <2 x i64>*
  %wide.load = load <2 x i64>, <2 x i64>* %16, align 8
  %17 = mul <2 x i64> %broadcast.splat, %wide.load
  %18 = add nuw nsw i64 %index, %row_offset_0.i.i
  %19 = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %18
  %20 = bitcast i64* %19 to <2 x i64>*
  %wide.load26 = load <2 x i64>, <2 x i64>* %20, align 8
  %21 = add <2 x i64> %wide.load26, %17
  store <2 x i64> %21, <2 x i64>* %20, align 8
  %index.next = add nuw i64 %index, 2
  %22 = icmp eq i64 %index.next, 220
  br i1 %22, label %inner_j_done, label %vector.body, !llvm.loop !2

inner_j_done:                                     ; preds = %vector.body
  %k_2 = add nuw nsw i64 %k_121, 1
  %exitcond24.not = icmp eq i64 %k_2, 240
  br i1 %exitcond24.not, label %main_k_done, label %inner_j.preheader

main_k_done:                                      ; preds = %inner_j_done
  %i_2 = add nuw nsw i64 %i_122, 1
  %exitcond25.not = icmp eq i64 %i_2, 200
  br i1 %exitcond25.not, label %body.i, label %main_j.preheader

body.i:                                           ; preds = %main_k_done, %body.i
  %i_12.i = phi i64 [ %i_2.i, %body.i ], [ 0, %main_k_done ]
  %mtx_loc_0.i = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %i_12.i
  %val_0.i18 = load i64, i64* %mtx_loc_0.i, align 8
  %23 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0.i18) #4
  %24 = tail call i32 @putchar(i32 10) #4
  %i_2.i = add nuw nsw i64 %i_12.i, 1
  %exitcond.not.i = icmp eq i64 %i_2.i, 44000
  br i1 %exitcond.not.i, label %__matrix_print.exit, label %body.i

__matrix_print.exit:                              ; preds = %body.i
  tail call void @free(i8* %z1.i)
  tail call void @free(i8* %z1.i1)
  tail call void @free(i8* nonnull %z1.i3)
  ret void
}

; Function Attrs: argmemonly nofree norecurse nosync nounwind writeonly
define dso_local void @__init(i64* nocapture writeonly %A, i64* nocapture writeonly %B, i64* nocapture writeonly %C, i64 %NI, i64 %fNI, i64 %NJ, i64 %fNJ, i64 %NK, i64 %fNK) local_unnamed_addr #5 {
pre_entry:
  %cond_038 = icmp sgt i64 %NI, 0
  br i1 %cond_038, label %init_C_j.preheader.lr.ph, label %init_B_i.preheader

init_C_j.preheader.lr.ph:                         ; preds = %pre_entry
  %cond_235 = icmp sgt i64 %NJ, 0
  br i1 %cond_235, label %init_C_j.preheader.us, label %init_A_j.preheader.lr.ph

init_C_j.preheader.us:                            ; preds = %init_C_j.preheader.lr.ph, %init_C_j.init_C_j_done_crit_edge.us
  %i_140.us = phi i64 [ %i_2.us, %init_C_j.init_C_j_done_crit_edge.us ], [ 0, %init_C_j.preheader.lr.ph ]
  %row_offset_0.i.i.us = mul i64 %i_140.us, %NJ
  br label %init_C_j_body.us

init_C_j_body.us:                                 ; preds = %init_C_j.preheader.us, %__fmod.exit.us
  %j_137.us = phi i64 [ 0, %init_C_j.preheader.us ], [ %j_2.us, %__fmod.exit.us ]
  %val_0.us = mul i64 %j_137.us, %i_140.us
  %val_1.us = add i64 %val_0.us, 1
  %cond_0.not1.i.us = icmp slt i64 %val_1.us, %fNI
  br i1 %cond_0.not1.i.us, label %__fmod.exit.us, label %while_inner.preheader.i.us

while_inner.preheader.i.us:                       ; preds = %init_C_j_body.us, %done_inner.i.us
  %rem_12.i.us = phi i64 [ %rem_2.i.us, %done_inner.i.us ], [ %val_1.us, %init_C_j_body.us ]
  br label %while_inner.i.us

while_inner.i.us:                                 ; preds = %while_inner.i.us, %while_inner.preheader.i.us
  %decr_1.i.us = phi i64 [ %decr_2.i.us, %while_inner.i.us ], [ %fNI, %while_inner.preheader.i.us ]
  %diff_0.i.us = sub i64 %rem_12.i.us, %decr_1.i.us
  %cond_2.i.us = icmp sgt i64 %diff_0.i.us, -1
  %decr_2.i.us = shl i64 %decr_1.i.us, 1
  br i1 %cond_2.i.us, label %while_inner.i.us, label %done_inner.i.us

done_inner.i.us:                                  ; preds = %while_inner.i.us
  %decr_3.neg.i.us = sdiv i64 %decr_1.i.us, -2
  %rem_2.i.us = add i64 %decr_3.neg.i.us, %rem_12.i.us
  %cond_0.not.i.us = icmp slt i64 %rem_2.i.us, %fNI
  br i1 %cond_0.not.i.us, label %__fmod.exit.us, label %while_inner.preheader.i.us

__fmod.exit.us:                                   ; preds = %done_inner.i.us, %init_C_j_body.us
  %rem_1.lcssa.i.us = phi i64 [ %val_1.us, %init_C_j_body.us ], [ %rem_2.i.us, %done_inner.i.us ]
  %val_3.us = sdiv i64 %rem_1.lcssa.i.us, %fNI
  %offset_0.i.i.us = add i64 %j_137.us, %row_offset_0.i.i.us
  %new_ptr_0.i.i.us = getelementptr inbounds i64, i64* %C, i64 %offset_0.i.i.us
  store i64 %val_3.us, i64* %new_ptr_0.i.i.us, align 8
  %j_2.us = add nuw nsw i64 %j_137.us, 1
  %exitcond.not = icmp eq i64 %j_2.us, %NJ
  br i1 %exitcond.not, label %init_C_j.init_C_j_done_crit_edge.us, label %init_C_j_body.us

init_C_j.init_C_j_done_crit_edge.us:              ; preds = %__fmod.exit.us
  %i_2.us = add nuw nsw i64 %i_140.us, 1
  %exitcond59.not = icmp eq i64 %i_2.us, %NI
  br i1 %exitcond59.not, label %init_A_j.preheader.lr.ph, label %init_C_j.preheader.us

init_A_j.preheader.lr.ph:                         ; preds = %init_C_j.init_C_j_done_crit_edge.us, %init_C_j.preheader.lr.ph
  %cond_642 = icmp sgt i64 %NK, 0
  br i1 %cond_642, label %init_A_j.preheader.us, label %init_B_i_done

init_A_j.preheader.us:                            ; preds = %init_A_j.preheader.lr.ph, %init_A_j.init_A_j_done_crit_edge.us
  %i_447.us = phi i64 [ %i_5.us, %init_A_j.init_A_j_done_crit_edge.us ], [ 0, %init_A_j.preheader.lr.ph ]
  %row_offset_0.i.i15.us = mul i64 %i_447.us, %NK
  br label %init_A_j_body.us

init_A_j_body.us:                                 ; preds = %init_A_j.preheader.us, %__fmod.exit14.us
  %j_544.us = phi i64 [ 0, %init_A_j.preheader.us ], [ %j_6.us, %__fmod.exit14.us ]
  %j_6.us = add nuw nsw i64 %j_544.us, 1
  %val_5.us = mul i64 %j_6.us, %i_447.us
  %cond_0.not1.i1.us = icmp slt i64 %val_5.us, %fNK
  br i1 %cond_0.not1.i1.us, label %__fmod.exit14.us, label %while_inner.preheader.i3.us

while_inner.preheader.i3.us:                      ; preds = %init_A_j_body.us, %done_inner.i12.us
  %rem_12.i2.us = phi i64 [ %rem_2.i10.us, %done_inner.i12.us ], [ %val_5.us, %init_A_j_body.us ]
  br label %while_inner.i8.us

while_inner.i8.us:                                ; preds = %while_inner.i8.us, %while_inner.preheader.i3.us
  %decr_1.i4.us = phi i64 [ %decr_2.i7.us, %while_inner.i8.us ], [ %fNK, %while_inner.preheader.i3.us ]
  %diff_0.i5.us = sub i64 %rem_12.i2.us, %decr_1.i4.us
  %cond_2.i6.us = icmp sgt i64 %diff_0.i5.us, -1
  %decr_2.i7.us = shl i64 %decr_1.i4.us, 1
  br i1 %cond_2.i6.us, label %while_inner.i8.us, label %done_inner.i12.us

done_inner.i12.us:                                ; preds = %while_inner.i8.us
  %decr_3.neg.i9.us = sdiv i64 %decr_1.i4.us, -2
  %rem_2.i10.us = add i64 %decr_3.neg.i9.us, %rem_12.i2.us
  %cond_0.not.i11.us = icmp slt i64 %rem_2.i10.us, %fNK
  br i1 %cond_0.not.i11.us, label %__fmod.exit14.us, label %while_inner.preheader.i3.us

__fmod.exit14.us:                                 ; preds = %done_inner.i12.us, %init_A_j_body.us
  %rem_1.lcssa.i13.us = phi i64 [ %val_5.us, %init_A_j_body.us ], [ %rem_2.i10.us, %done_inner.i12.us ]
  %val_7.us = sdiv i64 %rem_1.lcssa.i13.us, %fNK
  %offset_0.i.i16.us = add i64 %j_544.us, %row_offset_0.i.i15.us
  %new_ptr_0.i.i17.us = getelementptr inbounds i64, i64* %A, i64 %offset_0.i.i16.us
  store i64 %val_7.us, i64* %new_ptr_0.i.i17.us, align 8
  %exitcond60.not = icmp eq i64 %j_6.us, %NK
  br i1 %exitcond60.not, label %init_A_j.init_A_j_done_crit_edge.us, label %init_A_j_body.us

init_A_j.init_A_j_done_crit_edge.us:              ; preds = %__fmod.exit14.us
  %i_5.us = add nuw nsw i64 %i_447.us, 1
  %exitcond61.not = icmp eq i64 %i_5.us, %NI
  br i1 %exitcond61.not, label %init_B_i.preheader, label %init_A_j.preheader.us

init_B_i.preheader:                               ; preds = %init_A_j.init_A_j_done_crit_edge.us, %pre_entry
  %cond_852 = icmp sgt i64 %NK, 0
  %cond_1049 = icmp sgt i64 %NJ, 0
  %or.cond = select i1 %cond_852, i1 %cond_1049, i1 false
  br i1 %or.cond, label %init_B_j.preheader.us, label %init_B_i_done

init_B_j.preheader.us:                            ; preds = %init_B_i.preheader, %init_B_j.init_B_j_done_crit_edge.us
  %i_754.us = phi i64 [ %i_8.us, %init_B_j.init_B_j_done_crit_edge.us ], [ 0, %init_B_i.preheader ]
  %row_offset_0.i.i32.us = mul i64 %i_754.us, %NJ
  br label %init_B_j_body.us

init_B_j_body.us:                                 ; preds = %init_B_j.preheader.us, %__fmod.exit31.us
  %j_951.us = phi i64 [ 0, %init_B_j.preheader.us ], [ %j_10.us, %__fmod.exit31.us ]
  %val_8.us = add nuw i64 %j_951.us, 2
  %val_9.us = mul i64 %val_8.us, %i_754.us
  %cond_0.not1.i18.us = icmp slt i64 %val_9.us, %fNJ
  br i1 %cond_0.not1.i18.us, label %__fmod.exit31.us, label %while_inner.preheader.i20.us

while_inner.preheader.i20.us:                     ; preds = %init_B_j_body.us, %done_inner.i29.us
  %rem_12.i19.us = phi i64 [ %rem_2.i27.us, %done_inner.i29.us ], [ %val_9.us, %init_B_j_body.us ]
  br label %while_inner.i25.us

while_inner.i25.us:                               ; preds = %while_inner.i25.us, %while_inner.preheader.i20.us
  %decr_1.i21.us = phi i64 [ %decr_2.i24.us, %while_inner.i25.us ], [ %fNJ, %while_inner.preheader.i20.us ]
  %diff_0.i22.us = sub i64 %rem_12.i19.us, %decr_1.i21.us
  %cond_2.i23.us = icmp sgt i64 %diff_0.i22.us, -1
  %decr_2.i24.us = shl i64 %decr_1.i21.us, 1
  br i1 %cond_2.i23.us, label %while_inner.i25.us, label %done_inner.i29.us

done_inner.i29.us:                                ; preds = %while_inner.i25.us
  %decr_3.neg.i26.us = sdiv i64 %decr_1.i21.us, -2
  %rem_2.i27.us = add i64 %decr_3.neg.i26.us, %rem_12.i19.us
  %cond_0.not.i28.us = icmp slt i64 %rem_2.i27.us, %fNJ
  br i1 %cond_0.not.i28.us, label %__fmod.exit31.us, label %while_inner.preheader.i20.us

__fmod.exit31.us:                                 ; preds = %done_inner.i29.us, %init_B_j_body.us
  %rem_1.lcssa.i30.us = phi i64 [ %val_9.us, %init_B_j_body.us ], [ %rem_2.i27.us, %done_inner.i29.us ]
  %val_11.us = sdiv i64 %rem_1.lcssa.i30.us, %fNJ
  %offset_0.i.i33.us = add i64 %j_951.us, %row_offset_0.i.i32.us
  %new_ptr_0.i.i34.us = getelementptr inbounds i64, i64* %B, i64 %offset_0.i.i33.us
  store i64 %val_11.us, i64* %new_ptr_0.i.i34.us, align 8
  %j_10.us = add nuw nsw i64 %j_951.us, 1
  %exitcond62.not = icmp eq i64 %j_10.us, %NJ
  br i1 %exitcond62.not, label %init_B_j.init_B_j_done_crit_edge.us, label %init_B_j_body.us

init_B_j.init_B_j_done_crit_edge.us:              ; preds = %__fmod.exit31.us
  %i_8.us = add nuw nsw i64 %i_754.us, 1
  %exitcond63.not = icmp eq i64 %i_8.us, %NK
  br i1 %exitcond63.not, label %init_B_i_done, label %init_B_j.preheader.us

init_B_i_done:                                    ; preds = %init_B_j.init_B_j_done_crit_edge.us, %init_A_j.preheader.lr.ph, %init_B_i.preheader
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

; Function Attrs: argmemonly mustprogress nofree norecurse nosync nounwind willreturn
define dso_local void @__matrix_scale(i64* nocapture %mtx, i64 %row, i64 %col, i64 %Ncol, i64 %scale) local_unnamed_addr #9 {
pre_entry:
  %row_offset_0.i = mul i64 %Ncol, %row
  %offset_0.i = add i64 %row_offset_0.i, %col
  %new_ptr_0.i = getelementptr inbounds i64, i64* %mtx, i64 %offset_0.i
  %val_0 = load i64, i64* %new_ptr_0.i, align 8
  %new_val_0 = mul i64 %val_0, %scale
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
  call void @main.cold.1(i32 %1) #12
  ret i32 0

2:                                                ; preds = %0
  tail call void @__main()
  ret i32 0
}

; Function Attrs: cold minsize noreturn
define internal void @main.cold.1(i32 %0) #11 {
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
attributes #11 = { cold minsize noreturn }
attributes #12 = { noinline }

!0 = distinct !{!0, !1}
!1 = !{!"llvm.loop.isvectorized", i32 1}
!2 = distinct !{!2, !1}
