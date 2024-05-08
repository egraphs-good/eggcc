; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmplCCIXR/compile.ll'
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
  %z1.i = tail call dereferenceable_or_null(500000) i8* @malloc(i64 500000) #4
  %ptr_0.i = bitcast i8* %z1.i to i64*
  %z1.i1 = tail call dereferenceable_or_null(500000) i8* @malloc(i64 500000) #4
  %ptr_0.i2 = bitcast i8* %z1.i1 to i64*
  %z1.i3 = tail call dereferenceable_or_null(2000) i8* @malloc(i64 2000) #4
  %ptr_0.i4 = bitcast i8* %z1.i3 to i64*
  br label %__fmod.exit.us.i

__fmod.exit.us.i:                                 ; preds = %b0, %init_j.init_j_done_crit_edge.us.i
  %i_138.us.i = phi i64 [ %i_2.us.i, %init_j.init_j_done_crit_edge.us.i ], [ 0, %b0 ]
  %val_1.us.i = udiv i64 %i_138.us.i, 250
  %ptr_0.i.us.i = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %i_138.us.i
  store i64 %val_1.us.i, i64* %ptr_0.i.us.i, align 8
  %row_offset_0.i.i.us.i = mul nuw nsw i64 %i_138.us.i, 250
  br label %init_j_body.us.i

init_j_body.us.i:                                 ; preds = %__fmod.exit28.us.i, %__fmod.exit.us.i
  %j_134.us.i = phi i64 [ 0, %__fmod.exit.us.i ], [ %j_2.us.i, %__fmod.exit28.us.i ]
  %val_3.us.i = mul nuw nsw i64 %j_134.us.i, %i_138.us.i
  %val_4.us.i = add nuw nsw i64 %val_3.us.i, 1
  %cond_0.not1.i1.us.i = icmp ult i64 %val_3.us.i, 249
  br i1 %cond_0.not1.i1.us.i, label %__fmod.exit14.us.i, label %while_inner.preheader.i3.us.i

while_inner.preheader.i3.us.i:                    ; preds = %init_j_body.us.i, %done_inner.i12.us.i
  %rem_12.i2.us.i = phi i64 [ %rem_2.i10.us.i, %done_inner.i12.us.i ], [ %val_4.us.i, %init_j_body.us.i ]
  br label %while_inner.i8.us.i

while_inner.i8.us.i:                              ; preds = %while_inner.i8.us.i, %while_inner.preheader.i3.us.i
  %decr_1.i4.us.i = phi i64 [ %decr_2.i7.us.i, %while_inner.i8.us.i ], [ 250, %while_inner.preheader.i3.us.i ]
  %diff_0.i5.us.i = sub i64 %rem_12.i2.us.i, %decr_1.i4.us.i
  %cond_2.i6.us.i = icmp sgt i64 %diff_0.i5.us.i, -1
  %decr_2.i7.us.i = shl i64 %decr_1.i4.us.i, 1
  br i1 %cond_2.i6.us.i, label %while_inner.i8.us.i, label %done_inner.i12.us.i

done_inner.i12.us.i:                              ; preds = %while_inner.i8.us.i
  %decr_3.neg.i9.us.i = sdiv i64 %decr_1.i4.us.i, -2
  %rem_2.i10.us.i = add i64 %decr_3.neg.i9.us.i, %rem_12.i2.us.i
  %cond_0.not.i11.us.i = icmp slt i64 %rem_2.i10.us.i, 250
  br i1 %cond_0.not.i11.us.i, label %__fmod.exit14.us.i, label %while_inner.preheader.i3.us.i

__fmod.exit14.us.i:                               ; preds = %done_inner.i12.us.i, %init_j_body.us.i
  %rem_1.lcssa.i13.us.i = phi i64 [ %val_4.us.i, %init_j_body.us.i ], [ %rem_2.i10.us.i, %done_inner.i12.us.i ]
  %val_6.us.i = sdiv i64 %rem_1.lcssa.i13.us.i, 250
  %offset_0.i.i.us.i = add nuw nsw i64 %j_134.us.i, %row_offset_0.i.i.us.i
  %new_ptr_0.i.i.us.i = getelementptr inbounds i64, i64* %ptr_0.i, i64 %offset_0.i.i.us.i
  store i64 %val_6.us.i, i64* %new_ptr_0.i.i.us.i, align 8
  %val_8.us.i = add nuw nsw i64 %val_3.us.i, 2
  %cond_0.not1.i15.us.i = icmp ult i64 %val_3.us.i, 248
  br i1 %cond_0.not1.i15.us.i, label %__fmod.exit28.us.i, label %while_inner.preheader.i17.us.i

while_inner.preheader.i17.us.i:                   ; preds = %__fmod.exit14.us.i, %done_inner.i26.us.i
  %rem_12.i16.us.i = phi i64 [ %rem_2.i24.us.i, %done_inner.i26.us.i ], [ %val_8.us.i, %__fmod.exit14.us.i ]
  br label %while_inner.i22.us.i

while_inner.i22.us.i:                             ; preds = %while_inner.i22.us.i, %while_inner.preheader.i17.us.i
  %decr_1.i18.us.i = phi i64 [ %decr_2.i21.us.i, %while_inner.i22.us.i ], [ 250, %while_inner.preheader.i17.us.i ]
  %diff_0.i19.us.i = sub i64 %rem_12.i16.us.i, %decr_1.i18.us.i
  %cond_2.i20.us.i = icmp sgt i64 %diff_0.i19.us.i, -1
  %decr_2.i21.us.i = shl i64 %decr_1.i18.us.i, 1
  br i1 %cond_2.i20.us.i, label %while_inner.i22.us.i, label %done_inner.i26.us.i

done_inner.i26.us.i:                              ; preds = %while_inner.i22.us.i
  %decr_3.neg.i23.us.i = sdiv i64 %decr_1.i18.us.i, -2
  %rem_2.i24.us.i = add i64 %decr_3.neg.i23.us.i, %rem_12.i16.us.i
  %cond_0.not.i25.us.i = icmp slt i64 %rem_2.i24.us.i, 250
  br i1 %cond_0.not.i25.us.i, label %__fmod.exit28.us.i, label %while_inner.preheader.i17.us.i

__fmod.exit28.us.i:                               ; preds = %done_inner.i26.us.i, %__fmod.exit14.us.i
  %rem_1.lcssa.i27.us.i = phi i64 [ %val_8.us.i, %__fmod.exit14.us.i ], [ %rem_2.i24.us.i, %done_inner.i26.us.i ]
  %val_10.us.i = sdiv i64 %rem_1.lcssa.i27.us.i, 250
  %new_ptr_0.i.i31.us.i = getelementptr inbounds i64, i64* %ptr_0.i2, i64 %offset_0.i.i.us.i
  store i64 %val_10.us.i, i64* %new_ptr_0.i.i31.us.i, align 8
  %j_2.us.i = add nuw nsw i64 %j_134.us.i, 1
  %exitcond.not.i = icmp eq i64 %j_2.us.i, 250
  br i1 %exitcond.not.i, label %init_j.init_j_done_crit_edge.us.i, label %init_j_body.us.i

init_j.init_j_done_crit_edge.us.i:                ; preds = %__fmod.exit28.us.i
  %i_2.us.i = add nuw nsw i64 %i_138.us.i, 1
  %exitcond40.not.i = icmp eq i64 %i_2.us.i, 250
  br i1 %exitcond40.not.i, label %__init.exit, label %__fmod.exit.us.i

__init.exit:                                      ; preds = %init_j.init_j_done_crit_edge.us.i
  %z1.i7 = tail call dereferenceable_or_null(2000) i8* @malloc(i64 2000) #4
  %ptr_0.i8 = bitcast i8* %z1.i7 to i64*
  br label %main_i_body

main_i_body:                                      ; preds = %__init.exit, %middle.block
  %i_136 = phi i64 [ 0, %__init.exit ], [ %i_2, %middle.block ]
  %ptr_0.i10 = getelementptr inbounds i64, i64* %ptr_0.i8, i64 %i_136
  %row_offset_0.i.i = mul nuw nsw i64 %i_136, 250
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %main_i_body
  %index = phi i64 [ 0, %main_i_body ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 0, %main_i_body ], [ %12, %vector.body ]
  %vec.phi38 = phi i64 [ 0, %main_i_body ], [ %13, %vector.body ]
  %vec.phi39 = phi i64 [ 0, %main_i_body ], [ %20, %vector.body ]
  %vec.phi40 = phi i64 [ 0, %main_i_body ], [ %21, %vector.body ]
  %induction41 = or i64 %index, 1
  %0 = add nuw nsw i64 %index, %row_offset_0.i.i
  %1 = add nuw nsw i64 %induction41, %row_offset_0.i.i
  %2 = getelementptr inbounds i64, i64* %ptr_0.i, i64 %0
  %3 = getelementptr inbounds i64, i64* %ptr_0.i, i64 %1
  %4 = load i64, i64* %2, align 8
  %5 = load i64, i64* %3, align 8
  %6 = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %index
  %7 = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %induction41
  %8 = load i64, i64* %6, align 8
  %9 = load i64, i64* %7, align 8
  %10 = mul i64 %8, %4
  %11 = mul i64 %9, %5
  %12 = add i64 %10, %vec.phi
  %13 = add i64 %11, %vec.phi38
  %14 = getelementptr inbounds i64, i64* %ptr_0.i2, i64 %0
  %15 = getelementptr inbounds i64, i64* %ptr_0.i2, i64 %1
  %16 = load i64, i64* %14, align 8
  %17 = load i64, i64* %15, align 8
  %18 = mul i64 %16, %8
  %19 = mul i64 %17, %9
  %20 = add i64 %18, %vec.phi39
  %21 = add i64 %19, %vec.phi40
  %index.next = add nuw i64 %index, 2
  %22 = icmp eq i64 %index.next, 250
  br i1 %22, label %middle.block, label %vector.body, !llvm.loop !0

middle.block:                                     ; preds = %vector.body
  %bin.rdx42 = add i64 %21, %20
  %bin.rdx = add i64 %13, %12
  %val1_0 = mul i64 %bin.rdx, 3
  %val2_0 = shl i64 %bin.rdx42, 1
  %new_yi_0 = add i64 %val2_0, %val1_0
  store i64 %new_yi_0, i64* %ptr_0.i10, align 8
  %i_2 = add nuw nsw i64 %i_136, 1
  %exitcond37.not = icmp eq i64 %i_2, 250
  br i1 %exitcond37.not, label %body.i, label %main_i_body

body.i:                                           ; preds = %middle.block, %body.i
  %i_12.i = phi i64 [ %i_2.i, %body.i ], [ 0, %middle.block ]
  %ptr_0.i.i = getelementptr inbounds i64, i64* %ptr_0.i8, i64 %i_12.i
  %val_0.i.i = load i64, i64* %ptr_0.i.i, align 8
  %23 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0.i.i) #4
  %24 = tail call i32 @putchar(i32 10) #4
  %i_2.i = add nuw nsw i64 %i_12.i, 1
  %exitcond.not.i30 = icmp eq i64 %i_2.i, 250
  br i1 %exitcond.not.i30, label %__vector_print.exit, label %body.i

__vector_print.exit:                              ; preds = %body.i
  tail call void @free(i8* %z1.i)
  tail call void @free(i8* %z1.i1)
  tail call void @free(i8* %z1.i3)
  tail call void @free(i8* nonnull %z1.i7)
  ret void
}

; Function Attrs: argmemonly nofree norecurse nosync nounwind writeonly
define dso_local void @__init(i64* nocapture writeonly %A, i64* nocapture writeonly %B, i64* nocapture writeonly %x, i64 %N, i64 %fN) local_unnamed_addr #5 {
pre_entry:
  %cond_035 = icmp sgt i64 %N, 0
  br i1 %cond_035, label %init_i_body.us, label %init_i_done

init_i_body.us:                                   ; preds = %pre_entry, %init_j.init_j_done_crit_edge.us
  %i_138.us = phi i64 [ %i_2.us, %init_j.init_j_done_crit_edge.us ], [ 0, %pre_entry ]
  %cond_0.not1.i.us = icmp slt i64 %i_138.us, %fN
  br i1 %cond_0.not1.i.us, label %__fmod.exit.us, label %while_inner.preheader.i.us

while_inner.preheader.i.us:                       ; preds = %init_i_body.us, %done_inner.i.us
  %rem_12.i.us = phi i64 [ %rem_2.i.us, %done_inner.i.us ], [ %i_138.us, %init_i_body.us ]
  br label %while_inner.i.us

while_inner.i.us:                                 ; preds = %while_inner.i.us, %while_inner.preheader.i.us
  %decr_1.i.us = phi i64 [ %decr_2.i.us, %while_inner.i.us ], [ %fN, %while_inner.preheader.i.us ]
  %diff_0.i.us = sub i64 %rem_12.i.us, %decr_1.i.us
  %cond_2.i.us = icmp sgt i64 %diff_0.i.us, -1
  %decr_2.i.us = shl i64 %decr_1.i.us, 1
  br i1 %cond_2.i.us, label %while_inner.i.us, label %done_inner.i.us

done_inner.i.us:                                  ; preds = %while_inner.i.us
  %decr_3.neg.i.us = sdiv i64 %decr_1.i.us, -2
  %rem_2.i.us = add i64 %decr_3.neg.i.us, %rem_12.i.us
  %cond_0.not.i.us = icmp slt i64 %rem_2.i.us, %fN
  br i1 %cond_0.not.i.us, label %__fmod.exit.us, label %while_inner.preheader.i.us

__fmod.exit.us:                                   ; preds = %done_inner.i.us, %init_i_body.us
  %rem_1.lcssa.i.us = phi i64 [ %i_138.us, %init_i_body.us ], [ %rem_2.i.us, %done_inner.i.us ]
  %val_1.us = sdiv i64 %rem_1.lcssa.i.us, %fN
  %ptr_0.i.us = getelementptr inbounds i64, i64* %x, i64 %i_138.us
  store i64 %val_1.us, i64* %ptr_0.i.us, align 8
  %row_offset_0.i.i.us = mul i64 %i_138.us, %N
  br label %init_j_body.us

init_j_body.us:                                   ; preds = %__fmod.exit.us, %__fmod.exit28.us
  %j_134.us = phi i64 [ 0, %__fmod.exit.us ], [ %j_2.us, %__fmod.exit28.us ]
  %val_3.us = mul i64 %j_134.us, %i_138.us
  %val_4.us = add i64 %val_3.us, 1
  %cond_0.not1.i1.us = icmp slt i64 %val_4.us, %fN
  br i1 %cond_0.not1.i1.us, label %__fmod.exit14.us, label %while_inner.preheader.i3.us

while_inner.preheader.i3.us:                      ; preds = %init_j_body.us, %done_inner.i12.us
  %rem_12.i2.us = phi i64 [ %rem_2.i10.us, %done_inner.i12.us ], [ %val_4.us, %init_j_body.us ]
  br label %while_inner.i8.us

while_inner.i8.us:                                ; preds = %while_inner.i8.us, %while_inner.preheader.i3.us
  %decr_1.i4.us = phi i64 [ %decr_2.i7.us, %while_inner.i8.us ], [ %fN, %while_inner.preheader.i3.us ]
  %diff_0.i5.us = sub i64 %rem_12.i2.us, %decr_1.i4.us
  %cond_2.i6.us = icmp sgt i64 %diff_0.i5.us, -1
  %decr_2.i7.us = shl i64 %decr_1.i4.us, 1
  br i1 %cond_2.i6.us, label %while_inner.i8.us, label %done_inner.i12.us

done_inner.i12.us:                                ; preds = %while_inner.i8.us
  %decr_3.neg.i9.us = sdiv i64 %decr_1.i4.us, -2
  %rem_2.i10.us = add i64 %decr_3.neg.i9.us, %rem_12.i2.us
  %cond_0.not.i11.us = icmp slt i64 %rem_2.i10.us, %fN
  br i1 %cond_0.not.i11.us, label %__fmod.exit14.us, label %while_inner.preheader.i3.us

__fmod.exit14.us:                                 ; preds = %done_inner.i12.us, %init_j_body.us
  %rem_1.lcssa.i13.us = phi i64 [ %val_4.us, %init_j_body.us ], [ %rem_2.i10.us, %done_inner.i12.us ]
  %val_6.us = sdiv i64 %rem_1.lcssa.i13.us, %fN
  %offset_0.i.i.us = add i64 %j_134.us, %row_offset_0.i.i.us
  %new_ptr_0.i.i.us = getelementptr inbounds i64, i64* %A, i64 %offset_0.i.i.us
  store i64 %val_6.us, i64* %new_ptr_0.i.i.us, align 8
  %val_8.us = add i64 %val_3.us, 2
  %cond_0.not1.i15.us = icmp slt i64 %val_8.us, %fN
  br i1 %cond_0.not1.i15.us, label %__fmod.exit28.us, label %while_inner.preheader.i17.us

while_inner.preheader.i17.us:                     ; preds = %__fmod.exit14.us, %done_inner.i26.us
  %rem_12.i16.us = phi i64 [ %rem_2.i24.us, %done_inner.i26.us ], [ %val_8.us, %__fmod.exit14.us ]
  br label %while_inner.i22.us

while_inner.i22.us:                               ; preds = %while_inner.i22.us, %while_inner.preheader.i17.us
  %decr_1.i18.us = phi i64 [ %decr_2.i21.us, %while_inner.i22.us ], [ %fN, %while_inner.preheader.i17.us ]
  %diff_0.i19.us = sub i64 %rem_12.i16.us, %decr_1.i18.us
  %cond_2.i20.us = icmp sgt i64 %diff_0.i19.us, -1
  %decr_2.i21.us = shl i64 %decr_1.i18.us, 1
  br i1 %cond_2.i20.us, label %while_inner.i22.us, label %done_inner.i26.us

done_inner.i26.us:                                ; preds = %while_inner.i22.us
  %decr_3.neg.i23.us = sdiv i64 %decr_1.i18.us, -2
  %rem_2.i24.us = add i64 %decr_3.neg.i23.us, %rem_12.i16.us
  %cond_0.not.i25.us = icmp slt i64 %rem_2.i24.us, %fN
  br i1 %cond_0.not.i25.us, label %__fmod.exit28.us, label %while_inner.preheader.i17.us

__fmod.exit28.us:                                 ; preds = %done_inner.i26.us, %__fmod.exit14.us
  %rem_1.lcssa.i27.us = phi i64 [ %val_8.us, %__fmod.exit14.us ], [ %rem_2.i24.us, %done_inner.i26.us ]
  %val_10.us = sdiv i64 %rem_1.lcssa.i27.us, %fN
  %new_ptr_0.i.i31.us = getelementptr inbounds i64, i64* %B, i64 %offset_0.i.i.us
  store i64 %val_10.us, i64* %new_ptr_0.i.i31.us, align 8
  %j_2.us = add nuw nsw i64 %j_134.us, 1
  %exitcond.not = icmp eq i64 %j_2.us, %N
  br i1 %exitcond.not, label %init_j.init_j_done_crit_edge.us, label %init_j_body.us

init_j.init_j_done_crit_edge.us:                  ; preds = %__fmod.exit28.us
  %i_2.us = add nuw nsw i64 %i_138.us, 1
  %exitcond40.not = icmp eq i64 %i_2.us, %N
  br i1 %exitcond40.not, label %init_i_done, label %init_i_body.us

init_i_done:                                      ; preds = %init_j.init_j_done_crit_edge.us, %pre_entry
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

; Function Attrs: mustprogress nofree nounwind willreturn
define dso_local noalias i64* @__vector_new(i64 %N) local_unnamed_addr #6 {
pre_entry:
  %z0 = shl i64 %N, 3
  %z1 = tail call i8* @malloc(i64 %z0)
  %ptr_0 = bitcast i8* %z1 to i64*
  ret i64* %ptr_0
}

; Function Attrs: argmemonly mustprogress nofree norecurse nosync nounwind readonly willreturn
define dso_local i64 @__vector_get(i64* nocapture readonly %vec, i64 %i) local_unnamed_addr #3 {
pre_entry:
  %ptr_0 = getelementptr inbounds i64, i64* %vec, i64 %i
  %val_0 = load i64, i64* %ptr_0, align 8
  ret i64 %val_0
}

; Function Attrs: argmemonly mustprogress nofree norecurse nosync nounwind willreturn writeonly
define dso_local void @__vector_set(i64* nocapture writeonly %vec, i64 %i, i64 %val) local_unnamed_addr #8 {
pre_entry:
  %ptr_0 = getelementptr inbounds i64, i64* %vec, i64 %i
  store i64 %val, i64* %ptr_0, align 8
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__vector_print(i64* nocapture readonly %vec, i64 %N) local_unnamed_addr #0 {
pre_entry:
  %cond_01 = icmp sgt i64 %N, 0
  br i1 %cond_01, label %body, label %done

body:                                             ; preds = %pre_entry, %body
  %i_12 = phi i64 [ %i_2, %body ], [ 0, %pre_entry ]
  %ptr_0.i = getelementptr inbounds i64, i64* %vec, i64 %i_12
  %val_0.i = load i64, i64* %ptr_0.i, align 8
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0.i) #4
  %1 = tail call i32 @putchar(i32 10) #4
  %i_2 = add nuw nsw i64 %i_12, 1
  %exitcond.not = icmp eq i64 %i_2, %N
  br i1 %exitcond.not, label %done, label %body

done:                                             ; preds = %body, %pre_entry
  ret void
}

; Function Attrs: nofree norecurse nosync nounwind readnone
define dso_local i64 @__fmod(i64 %n, i64 %m) local_unnamed_addr #9 {
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
  call void @main.cold.1(i32 %1) #11
  ret i32 0

2:                                                ; preds = %0
  tail call void @__main()
  ret i32 0
}

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
attributes #5 = { argmemonly nofree norecurse nosync nounwind writeonly }
attributes #6 = { mustprogress nofree nounwind willreturn }
attributes #7 = { mustprogress nofree norecurse nosync nounwind readnone willreturn }
attributes #8 = { argmemonly mustprogress nofree norecurse nosync nounwind willreturn writeonly }
attributes #9 = { nofree norecurse nosync nounwind readnone }
attributes #10 = { cold minsize noreturn }
attributes #11 = { noinline }

!0 = distinct !{!0, !1}
!1 = !{!"llvm.loop.isvectorized", i32 1}
