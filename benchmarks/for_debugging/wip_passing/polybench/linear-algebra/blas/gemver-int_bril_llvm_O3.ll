; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpt5ai2W/compile.ll'
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
  %z1.i = tail call dereferenceable_or_null(1280000) i8* @malloc(i64 1280000) #4
  %ptr_0.i = bitcast i8* %z1.i to i64*
  %z1.i1 = tail call dereferenceable_or_null(3200) i8* @malloc(i64 3200) #4
  %ptr_0.i2 = bitcast i8* %z1.i1 to i64*
  %z1.i3 = tail call dereferenceable_or_null(3200) i8* @malloc(i64 3200) #4
  %ptr_0.i4 = bitcast i8* %z1.i3 to i64*
  %z1.i5 = tail call dereferenceable_or_null(3200) i8* @malloc(i64 3200) #4
  %ptr_0.i6 = bitcast i8* %z1.i5 to i64*
  %z1.i7 = tail call dereferenceable_or_null(3200) i8* @malloc(i64 3200) #4
  %ptr_0.i8 = bitcast i8* %z1.i7 to i64*
  %calloc71 = call dereferenceable_or_null(3200) i8* @calloc(i64 1, i64 3200)
  %calloc = call dereferenceable_or_null(3200) i8* @calloc(i64 1, i64 3200)
  %z1.i13 = tail call dereferenceable_or_null(3200) i8* @malloc(i64 3200) #4
  %ptr_0.i14 = bitcast i8* %z1.i13 to i64*
  %z1.i15 = tail call dereferenceable_or_null(3200) i8* @malloc(i64 3200) #4
  %ptr_0.i16 = bitcast i8* %z1.i15 to i64*
  br label %init_i_body.us.i

init_i_body.us.i:                                 ; preds = %init_j.init_j_done_crit_edge.us.i, %b0
  %i_113.us.i = phi i64 [ %i_2.us.i, %init_j.init_j_done_crit_edge.us.i ], [ 0, %b0 ]
  %ptr_0.i.us.i = getelementptr inbounds i64, i64* %ptr_0.i2, i64 %i_113.us.i
  store i64 %i_113.us.i, i64* %ptr_0.i.us.i, align 8
  %i_2.us.i = add nuw nsw i64 %i_113.us.i, 1
  %val_2.us.i = udiv i64 %i_2.us.i, 800
  %ptr_0.i1.us.i = getelementptr inbounds i64, i64* %ptr_0.i6, i64 %i_113.us.i
  store i64 %val_2.us.i, i64* %ptr_0.i1.us.i, align 8
  %val_5.us.i = udiv i64 %i_2.us.i, 1600
  %ptr_0.i2.us.i = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %i_113.us.i
  store i64 %val_5.us.i, i64* %ptr_0.i2.us.i, align 8
  %val_8.us.i = udiv i64 %i_2.us.i, 2400
  %ptr_0.i3.us.i = getelementptr inbounds i64, i64* %ptr_0.i8, i64 %i_113.us.i
  store i64 %val_8.us.i, i64* %ptr_0.i3.us.i, align 8
  %val_11.us.i = udiv i64 %i_2.us.i, 3200
  %ptr_0.i4.us.i = getelementptr inbounds i64, i64* %ptr_0.i14, i64 %i_113.us.i
  store i64 %val_11.us.i, i64* %ptr_0.i4.us.i, align 8
  %val_14.us.i = udiv i64 %i_2.us.i, 3600
  %ptr_0.i5.us.i = getelementptr inbounds i64, i64* %ptr_0.i16, i64 %i_113.us.i
  store i64 %val_14.us.i, i64* %ptr_0.i5.us.i, align 8
  %row_offset_0.i.i.us.i = mul nuw nsw i64 %i_113.us.i, 400
  br label %init_j_body.us.i

init_j_body.us.i:                                 ; preds = %__fmod.exit.us.i, %init_i_body.us.i
  %j_110.us.i = phi i64 [ 0, %init_i_body.us.i ], [ %j_2.us.i, %__fmod.exit.us.i ]
  %val_16.us.i = mul nuw nsw i64 %j_110.us.i, %i_113.us.i
  %cond_0.not1.i.us.i = icmp ult i64 %val_16.us.i, 400
  br i1 %cond_0.not1.i.us.i, label %__fmod.exit.us.i, label %while_inner.preheader.i.us.i

while_inner.preheader.i.us.i:                     ; preds = %init_j_body.us.i, %done_inner.i.us.i
  %rem_12.i.us.i = phi i64 [ %rem_2.i.us.i, %done_inner.i.us.i ], [ %val_16.us.i, %init_j_body.us.i ]
  br label %while_inner.i.us.i

while_inner.i.us.i:                               ; preds = %while_inner.i.us.i, %while_inner.preheader.i.us.i
  %decr_1.i.us.i = phi i64 [ %decr_2.i.us.i, %while_inner.i.us.i ], [ 400, %while_inner.preheader.i.us.i ]
  %diff_0.i.us.i = sub i64 %rem_12.i.us.i, %decr_1.i.us.i
  %cond_2.i.us.i = icmp sgt i64 %diff_0.i.us.i, -1
  %decr_2.i.us.i = shl i64 %decr_1.i.us.i, 1
  br i1 %cond_2.i.us.i, label %while_inner.i.us.i, label %done_inner.i.us.i

done_inner.i.us.i:                                ; preds = %while_inner.i.us.i
  %decr_3.neg.i.us.i = sdiv i64 %decr_1.i.us.i, -2
  %rem_2.i.us.i = add i64 %decr_3.neg.i.us.i, %rem_12.i.us.i
  %cond_0.not.i.us.i = icmp slt i64 %rem_2.i.us.i, 400
  br i1 %cond_0.not.i.us.i, label %__fmod.exit.us.i, label %while_inner.preheader.i.us.i

__fmod.exit.us.i:                                 ; preds = %done_inner.i.us.i, %init_j_body.us.i
  %rem_1.lcssa.i.us.i = phi i64 [ %val_16.us.i, %init_j_body.us.i ], [ %rem_2.i.us.i, %done_inner.i.us.i ]
  %val_18.us.i = sdiv i64 %rem_1.lcssa.i.us.i, 400
  %offset_0.i.i.us.i = add nuw nsw i64 %j_110.us.i, %row_offset_0.i.i.us.i
  %new_ptr_0.i.i.us.i = getelementptr inbounds i64, i64* %ptr_0.i, i64 %offset_0.i.i.us.i
  store i64 %val_18.us.i, i64* %new_ptr_0.i.i.us.i, align 8
  %j_2.us.i = add nuw nsw i64 %j_110.us.i, 1
  %exitcond.not.i = icmp eq i64 %j_2.us.i, 400
  br i1 %exitcond.not.i, label %init_j.init_j_done_crit_edge.us.i, label %init_j_body.us.i

init_j.init_j_done_crit_edge.us.i:                ; preds = %__fmod.exit.us.i
  %exitcond17.not.i = icmp eq i64 %i_2.us.i, 400
  br i1 %exitcond17.not.i, label %part1_j.preheader.preheader, label %init_i_body.us.i

part1_j.preheader.preheader:                      ; preds = %init_j.init_j_done_crit_edge.us.i
  %ptr_0.i10 = bitcast i8* %calloc71 to i64*
  br label %part1_j.preheader

part1_j.preheader:                                ; preds = %part1_j.preheader.preheader, %part1_j_done
  %i_153 = phi i64 [ %i_2, %part1_j_done ], [ 0, %part1_j.preheader.preheader ]
  %ptr_0.i17 = getelementptr inbounds i64, i64* %ptr_0.i2, i64 %i_153
  %val_0.i = load i64, i64* %ptr_0.i17, align 8
  %ptr_0.i20 = getelementptr inbounds i64, i64* %ptr_0.i6, i64 %i_153
  %val_0.i21 = load i64, i64* %ptr_0.i20, align 8
  %row_offset_0.i.i = mul nuw nsw i64 %i_153, 400
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %part1_j.preheader
  %index = phi i64 [ 0, %part1_j.preheader ], [ %index.next, %vector.body ]
  %induction72 = or i64 %index, 1
  %induction73 = or i64 %index, 2
  %induction74 = or i64 %index, 3
  %0 = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %index
  %1 = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %induction72
  %2 = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %induction73
  %3 = getelementptr inbounds i64, i64* %ptr_0.i4, i64 %induction74
  %4 = load i64, i64* %0, align 8
  %5 = load i64, i64* %1, align 8
  %6 = load i64, i64* %2, align 8
  %7 = load i64, i64* %3, align 8
  %8 = getelementptr inbounds i64, i64* %ptr_0.i8, i64 %index
  %9 = getelementptr inbounds i64, i64* %ptr_0.i8, i64 %induction72
  %10 = getelementptr inbounds i64, i64* %ptr_0.i8, i64 %induction73
  %11 = getelementptr inbounds i64, i64* %ptr_0.i8, i64 %induction74
  %12 = load i64, i64* %8, align 8
  %13 = load i64, i64* %9, align 8
  %14 = load i64, i64* %10, align 8
  %15 = load i64, i64* %11, align 8
  %16 = add nuw nsw i64 %index, %row_offset_0.i.i
  %17 = add nuw nsw i64 %induction72, %row_offset_0.i.i
  %18 = add nuw nsw i64 %induction73, %row_offset_0.i.i
  %19 = add nuw nsw i64 %induction74, %row_offset_0.i.i
  %20 = getelementptr inbounds i64, i64* %ptr_0.i, i64 %16
  %21 = getelementptr inbounds i64, i64* %ptr_0.i, i64 %17
  %22 = getelementptr inbounds i64, i64* %ptr_0.i, i64 %18
  %23 = getelementptr inbounds i64, i64* %ptr_0.i, i64 %19
  %24 = load i64, i64* %20, align 8
  %25 = load i64, i64* %21, align 8
  %26 = load i64, i64* %22, align 8
  %27 = load i64, i64* %23, align 8
  %28 = mul i64 %12, %val_0.i21
  %29 = mul i64 %13, %val_0.i21
  %30 = mul i64 %14, %val_0.i21
  %31 = mul i64 %15, %val_0.i21
  %32 = mul i64 %4, %val_0.i
  %33 = mul i64 %5, %val_0.i
  %34 = mul i64 %6, %val_0.i
  %35 = mul i64 %7, %val_0.i
  %36 = add i64 %28, %32
  %37 = add i64 %29, %33
  %38 = add i64 %30, %34
  %39 = add i64 %31, %35
  %40 = add i64 %36, %24
  %41 = add i64 %37, %25
  %42 = add i64 %38, %26
  %43 = add i64 %39, %27
  store i64 %40, i64* %20, align 8
  store i64 %41, i64* %21, align 8
  store i64 %42, i64* %22, align 8
  store i64 %43, i64* %23, align 8
  %index.next = add nuw i64 %index, 4
  %44 = icmp eq i64 %index.next, 400
  br i1 %44, label %part1_j_done, label %vector.body, !llvm.loop !0

part1_j_done:                                     ; preds = %vector.body
  %i_2 = add nuw nsw i64 %i_153, 1
  %exitcond63.not = icmp eq i64 %i_2, 400
  br i1 %exitcond63.not, label %part2_j.preheader.preheader, label %part1_j.preheader

part2_j.preheader.preheader:                      ; preds = %part1_j_done
  %ptr_0.i12 = bitcast i8* %calloc to i64*
  br label %part2_j.preheader

part2_j.preheader:                                ; preds = %part2_j_done.part2_j.preheader_crit_edge, %part2_j.preheader.preheader
  %ptr_0.i34.promoted = phi i64 [ %ptr_0.i34.promoted.pre, %part2_j_done.part2_j.preheader_crit_edge ], [ 0, %part2_j.preheader.preheader ]
  %i_457 = phi i64 [ %i_5, %part2_j_done.part2_j.preheader_crit_edge ], [ 0, %part2_j.preheader.preheader ]
  %ptr_0.i34 = getelementptr inbounds i64, i64* %ptr_0.i12, i64 %i_457
  br label %vector.body80

vector.body80:                                    ; preds = %vector.body80, %part2_j.preheader
  %index81 = phi i64 [ 0, %part2_j.preheader ], [ %index.next85, %vector.body80 ]
  %vec.phi = phi i64 [ %ptr_0.i34.promoted, %part2_j.preheader ], [ %61, %vector.body80 ]
  %vec.phi82 = phi i64 [ 0, %part2_j.preheader ], [ %62, %vector.body80 ]
  %induction84 = or i64 %index81, 1
  %45 = mul nuw nsw i64 %index81, 400
  %46 = mul nuw nsw i64 %induction84, 400
  %47 = add nuw nsw i64 %45, %i_457
  %48 = add nuw nsw i64 %46, %i_457
  %49 = getelementptr inbounds i64, i64* %ptr_0.i, i64 %47
  %50 = getelementptr inbounds i64, i64* %ptr_0.i, i64 %48
  %51 = load i64, i64* %49, align 8
  %52 = load i64, i64* %50, align 8
  %53 = getelementptr inbounds i64, i64* %ptr_0.i14, i64 %index81
  %54 = getelementptr inbounds i64, i64* %ptr_0.i14, i64 %induction84
  %55 = load i64, i64* %53, align 8
  %56 = load i64, i64* %54, align 8
  %57 = shl i64 %51, 1
  %58 = shl i64 %52, 1
  %59 = mul i64 %57, %55
  %60 = mul i64 %58, %56
  %61 = add i64 %vec.phi, %59
  %62 = add i64 %vec.phi82, %60
  %index.next85 = add nuw i64 %index81, 2
  %63 = icmp eq i64 %index.next85, 400
  br i1 %63, label %middle.block75, label %vector.body80, !llvm.loop !2

middle.block75:                                   ; preds = %vector.body80
  %bin.rdx = add i64 %62, %61
  store i64 %bin.rdx, i64* %ptr_0.i34, align 8
  %i_5 = add nuw nsw i64 %i_457, 1
  %exitcond65.not = icmp eq i64 %i_5, 400
  br i1 %exitcond65.not, label %vector.body91, label %part2_j_done.part2_j.preheader_crit_edge

vector.body91:                                    ; preds = %middle.block75, %vector.body91
  %index92 = phi i64 [ %index.next100, %vector.body91 ], [ 0, %middle.block75 ]
  %64 = getelementptr inbounds i64, i64* %ptr_0.i12, i64 %index92
  %65 = bitcast i64* %64 to <2 x i64>*
  %wide.load = load <2 x i64>, <2 x i64>* %65, align 8
  %66 = getelementptr inbounds i64, i64* %64, i64 2
  %67 = bitcast i64* %66 to <2 x i64>*
  %wide.load93 = load <2 x i64>, <2 x i64>* %67, align 8
  %68 = getelementptr inbounds i64, i64* %64, i64 4
  %69 = bitcast i64* %68 to <2 x i64>*
  %wide.load94 = load <2 x i64>, <2 x i64>* %69, align 8
  %70 = getelementptr inbounds i64, i64* %64, i64 6
  %71 = bitcast i64* %70 to <2 x i64>*
  %wide.load95 = load <2 x i64>, <2 x i64>* %71, align 8
  %72 = getelementptr inbounds i64, i64* %ptr_0.i16, i64 %index92
  %73 = bitcast i64* %72 to <2 x i64>*
  %wide.load96 = load <2 x i64>, <2 x i64>* %73, align 8
  %74 = getelementptr inbounds i64, i64* %72, i64 2
  %75 = bitcast i64* %74 to <2 x i64>*
  %wide.load97 = load <2 x i64>, <2 x i64>* %75, align 8
  %76 = getelementptr inbounds i64, i64* %72, i64 4
  %77 = bitcast i64* %76 to <2 x i64>*
  %wide.load98 = load <2 x i64>, <2 x i64>* %77, align 8
  %78 = getelementptr inbounds i64, i64* %72, i64 6
  %79 = bitcast i64* %78 to <2 x i64>*
  %wide.load99 = load <2 x i64>, <2 x i64>* %79, align 8
  %80 = add <2 x i64> %wide.load96, %wide.load
  %81 = add <2 x i64> %wide.load97, %wide.load93
  %82 = add <2 x i64> %wide.load98, %wide.load94
  %83 = add <2 x i64> %wide.load99, %wide.load95
  store <2 x i64> %80, <2 x i64>* %65, align 8
  store <2 x i64> %81, <2 x i64>* %67, align 8
  store <2 x i64> %82, <2 x i64>* %69, align 8
  store <2 x i64> %83, <2 x i64>* %71, align 8
  %index.next100 = add nuw i64 %index92, 8
  %84 = icmp eq i64 %index.next100, 400
  br i1 %84, label %part4_j.preheader, label %vector.body91, !llvm.loop !3

part2_j_done.part2_j.preheader_crit_edge:         ; preds = %middle.block75
  %ptr_0.i34.phi.trans.insert = getelementptr inbounds i64, i64* %ptr_0.i12, i64 %i_5
  %ptr_0.i34.promoted.pre = load i64, i64* %ptr_0.i34.phi.trans.insert, align 8
  br label %part2_j.preheader

part4_j.preheader:                                ; preds = %vector.body91, %part4_j_done.part4_j.preheader_crit_edge
  %ptr_0.i48.promoted = phi i64 [ %ptr_0.i48.promoted.pre, %part4_j_done.part4_j.preheader_crit_edge ], [ 0, %vector.body91 ]
  %i_1062 = phi i64 [ %i_11, %part4_j_done.part4_j.preheader_crit_edge ], [ 0, %vector.body91 ]
  %row_offset_0.i.i42 = mul nuw nsw i64 %i_1062, 400
  %ptr_0.i48 = getelementptr inbounds i64, i64* %ptr_0.i10, i64 %i_1062
  br label %vector.body106

vector.body106:                                   ; preds = %vector.body106, %part4_j.preheader
  %index107 = phi i64 [ 0, %part4_j.preheader ], [ %index.next112, %vector.body106 ]
  %vec.phi108 = phi i64 [ %ptr_0.i48.promoted, %part4_j.preheader ], [ %99, %vector.body106 ]
  %vec.phi109 = phi i64 [ 0, %part4_j.preheader ], [ %100, %vector.body106 ]
  %induction111 = or i64 %index107, 1
  %85 = add nuw nsw i64 %index107, %row_offset_0.i.i42
  %86 = add nuw nsw i64 %induction111, %row_offset_0.i.i42
  %87 = getelementptr inbounds i64, i64* %ptr_0.i, i64 %85
  %88 = getelementptr inbounds i64, i64* %ptr_0.i, i64 %86
  %89 = load i64, i64* %87, align 8
  %90 = load i64, i64* %88, align 8
  %91 = getelementptr inbounds i64, i64* %ptr_0.i12, i64 %index107
  %92 = getelementptr inbounds i64, i64* %ptr_0.i12, i64 %induction111
  %93 = load i64, i64* %91, align 8
  %94 = load i64, i64* %92, align 8
  %95 = mul i64 %89, 3
  %96 = mul i64 %90, 3
  %97 = mul i64 %95, %93
  %98 = mul i64 %96, %94
  %99 = add i64 %vec.phi108, %97
  %100 = add i64 %vec.phi109, %98
  %index.next112 = add nuw i64 %index107, 2
  %101 = icmp eq i64 %index.next112, 400
  br i1 %101, label %middle.block101, label %vector.body106, !llvm.loop !4

middle.block101:                                  ; preds = %vector.body106
  %bin.rdx113 = add i64 %100, %99
  store i64 %bin.rdx113, i64* %ptr_0.i48, align 8
  %i_11 = add nuw nsw i64 %i_1062, 1
  %exitcond68.not = icmp eq i64 %i_11, 400
  br i1 %exitcond68.not, label %body.i, label %part4_j_done.part4_j.preheader_crit_edge

part4_j_done.part4_j.preheader_crit_edge:         ; preds = %middle.block101
  %ptr_0.i48.phi.trans.insert = getelementptr inbounds i64, i64* %ptr_0.i10, i64 %i_11
  %ptr_0.i48.promoted.pre = load i64, i64* %ptr_0.i48.phi.trans.insert, align 8
  br label %part4_j.preheader

body.i:                                           ; preds = %middle.block101, %body.i
  %i_12.i = phi i64 [ %i_2.i, %body.i ], [ 0, %middle.block101 ]
  %ptr_0.i.i = getelementptr inbounds i64, i64* %ptr_0.i10, i64 %i_12.i
  %val_0.i.i = load i64, i64* %ptr_0.i.i, align 8
  %102 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0.i.i) #4
  %103 = tail call i32 @putchar(i32 10) #4
  %i_2.i = add nuw nsw i64 %i_12.i, 1
  %exitcond.not.i51 = icmp eq i64 %i_2.i, 400
  br i1 %exitcond.not.i51, label %__vector_print.exit, label %body.i

__vector_print.exit:                              ; preds = %body.i
  tail call void @free(i8* %z1.i)
  tail call void @free(i8* %z1.i1)
  tail call void @free(i8* %z1.i3)
  tail call void @free(i8* %z1.i5)
  tail call void @free(i8* %z1.i7)
  tail call void @free(i8* nonnull %calloc71)
  tail call void @free(i8* %calloc)
  tail call void @free(i8* %z1.i13)
  tail call void @free(i8* %z1.i15)
  ret void
}

; Function Attrs: argmemonly nofree norecurse nosync nounwind writeonly
define dso_local void @__init(i64* nocapture writeonly %A, i64* nocapture writeonly %u1, i64* nocapture writeonly %v1, i64* nocapture writeonly %u2, i64* nocapture writeonly %v2, i64* nocapture writeonly %w, i64* nocapture writeonly %x, i64* nocapture writeonly %y, i64* nocapture writeonly %z, i64 %N, i64 %fN) local_unnamed_addr #5 {
pre_entry:
  %cond_011 = icmp sgt i64 %N, 0
  br i1 %cond_011, label %init_i_body.us, label %init_i_done

init_i_body.us:                                   ; preds = %pre_entry, %init_j.init_j_done_crit_edge.us
  %i_113.us = phi i64 [ %i_2.us, %init_j.init_j_done_crit_edge.us ], [ 0, %pre_entry ]
  %ptr_0.i.us = getelementptr inbounds i64, i64* %u1, i64 %i_113.us
  store i64 %i_113.us, i64* %ptr_0.i.us, align 8
  %i_2.us = add nuw nsw i64 %i_113.us, 1
  %val_1.us = sdiv i64 %i_2.us, %fN
  %val_2.us = sdiv i64 %val_1.us, 2
  %ptr_0.i1.us = getelementptr inbounds i64, i64* %u2, i64 %i_113.us
  store i64 %val_2.us, i64* %ptr_0.i1.us, align 8
  %val_5.us = sdiv i64 %val_1.us, 4
  %ptr_0.i2.us = getelementptr inbounds i64, i64* %v1, i64 %i_113.us
  store i64 %val_5.us, i64* %ptr_0.i2.us, align 8
  %val_8.us = sdiv i64 %val_1.us, 6
  %ptr_0.i3.us = getelementptr inbounds i64, i64* %v2, i64 %i_113.us
  store i64 %val_8.us, i64* %ptr_0.i3.us, align 8
  %val_11.us = sdiv i64 %val_1.us, 8
  %ptr_0.i4.us = getelementptr inbounds i64, i64* %y, i64 %i_113.us
  store i64 %val_11.us, i64* %ptr_0.i4.us, align 8
  %val_14.us = sdiv i64 %val_1.us, 9
  %ptr_0.i5.us = getelementptr inbounds i64, i64* %z, i64 %i_113.us
  store i64 %val_14.us, i64* %ptr_0.i5.us, align 8
  %ptr_0.i6.us = getelementptr inbounds i64, i64* %x, i64 %i_113.us
  store i64 0, i64* %ptr_0.i6.us, align 8
  %ptr_0.i7.us = getelementptr inbounds i64, i64* %w, i64 %i_113.us
  store i64 0, i64* %ptr_0.i7.us, align 8
  %row_offset_0.i.i.us = mul i64 %i_113.us, %N
  br label %init_j_body.us

init_j_body.us:                                   ; preds = %init_i_body.us, %__fmod.exit.us
  %j_110.us = phi i64 [ 0, %init_i_body.us ], [ %j_2.us, %__fmod.exit.us ]
  %val_16.us = mul i64 %j_110.us, %i_113.us
  %cond_0.not1.i.us = icmp slt i64 %val_16.us, %fN
  br i1 %cond_0.not1.i.us, label %__fmod.exit.us, label %while_inner.preheader.i.us

while_inner.preheader.i.us:                       ; preds = %init_j_body.us, %done_inner.i.us
  %rem_12.i.us = phi i64 [ %rem_2.i.us, %done_inner.i.us ], [ %val_16.us, %init_j_body.us ]
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

__fmod.exit.us:                                   ; preds = %done_inner.i.us, %init_j_body.us
  %rem_1.lcssa.i.us = phi i64 [ %val_16.us, %init_j_body.us ], [ %rem_2.i.us, %done_inner.i.us ]
  %val_18.us = sdiv i64 %rem_1.lcssa.i.us, %fN
  %offset_0.i.i.us = add i64 %j_110.us, %row_offset_0.i.i.us
  %new_ptr_0.i.i.us = getelementptr inbounds i64, i64* %A, i64 %offset_0.i.i.us
  store i64 %val_18.us, i64* %new_ptr_0.i.i.us, align 8
  %j_2.us = add nuw nsw i64 %j_110.us, 1
  %exitcond.not = icmp eq i64 %j_2.us, %N
  br i1 %exitcond.not, label %init_j.init_j_done_crit_edge.us, label %init_j_body.us

init_j.init_j_done_crit_edge.us:                  ; preds = %__fmod.exit.us
  %exitcond17.not = icmp eq i64 %i_2.us, %N
  br i1 %exitcond17.not, label %init_i_done, label %init_i_body.us

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
  call void @main.cold.1(i32 %1) #12
  ret i32 0

2:                                                ; preds = %0
  tail call void @__main()
  ret i32 0
}

; Function Attrs: inaccessiblememonly nofree nounwind willreturn allocsize(0,1)
declare noalias noundef i8* @calloc(i64 noundef, i64 noundef) local_unnamed_addr #10

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
attributes #9 = { nofree norecurse nosync nounwind readnone }
attributes #10 = { inaccessiblememonly nofree nounwind willreturn allocsize(0,1) }
attributes #11 = { cold minsize noreturn }
attributes #12 = { noinline }

!0 = distinct !{!0, !1}
!1 = !{!"llvm.loop.isvectorized", i32 1}
!2 = distinct !{!2, !1}
!3 = distinct !{!3, !1}
!4 = distinct !{!4, !1}
