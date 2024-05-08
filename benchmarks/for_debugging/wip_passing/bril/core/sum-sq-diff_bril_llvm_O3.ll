; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpe502VR/compile.ll'
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

; Function Attrs: nofree norecurse nosync nounwind readnone
define dso_local i64 @__sumOfSquares(i64 %n) local_unnamed_addr #2 {
pre_entry:
  %v5_0.not1 = icmp slt i64 %n, 1
  br i1 %v5_0.not1, label %for.end.1, label %for.body.1

for.body.1:                                       ; preds = %pre_entry, %for.body.1
  %res_13 = phi i64 [ %v11_0, %for.body.1 ], [ 0, %pre_entry ]
  %i_12 = phi i64 [ %v14_0, %for.body.1 ], [ 1, %pre_entry ]
  %v8_0 = mul i64 %i_12, %i_12
  %v11_0 = add i64 %res_13, %v8_0
  %v14_0 = add i64 %i_12, 1
  %v5_0.not = icmp sgt i64 %v14_0, %n
  br i1 %v5_0.not, label %for.end.1, label %for.body.1

for.end.1:                                        ; preds = %for.body.1, %pre_entry
  %res_1.lcssa = phi i64 [ 0, %pre_entry ], [ %v11_0, %for.body.1 ]
  ret i64 %res_1.lcssa
}

; Function Attrs: nofree norecurse nosync nounwind readnone
define dso_local i64 @__squareOfSum(i64 %n) local_unnamed_addr #2 {
pre_entry:
  %v5_0.not1 = icmp slt i64 %n, 1
  br i1 %v5_0.not1, label %for.end.1, label %for.body.1

for.body.1:                                       ; preds = %pre_entry, %for.body.1
  %res_13 = phi i64 [ %v8_0, %for.body.1 ], [ 0, %pre_entry ]
  %i_12 = phi i64 [ %v11_0, %for.body.1 ], [ 1, %pre_entry ]
  %v8_0 = add i64 %res_13, %i_12
  %v11_0 = add i64 %i_12, 1
  %v5_0.not = icmp sgt i64 %v11_0, %n
  br i1 %v5_0.not, label %for.end.1, label %for.body.1

for.end.1:                                        ; preds = %for.body.1, %pre_entry
  %res_1.lcssa = phi i64 [ 0, %pre_entry ], [ %v8_0, %for.body.1 ]
  %v14_0 = mul i64 %res_1.lcssa, %res_1.lcssa
  ret i64 %v14_0
}

; Function Attrs: nofree nounwind
define dso_local void @__main() local_unnamed_addr #0 {
b0:
  br label %for.body.1.i.i.preheader

for.body.1.i.i.preheader:                         ; preds = %b0, %for.body.1.i.i.preheader
  %indvars.iv12 = phi i64 [ 19, %b0 ], [ %indvars.iv.next13, %for.body.1.i.i.preheader ]
  %indvars.iv10 = phi i65 [ 72, %b0 ], [ %indvars.iv.next11, %for.body.1.i.i.preheader ]
  %indvars.iv8 = phi i65 [ 18, %b0 ], [ %indvars.iv.next9, %for.body.1.i.i.preheader ]
  %indvars.iv6 = phi i65 [ 504, %b0 ], [ %indvars.iv.next7, %for.body.1.i.i.preheader ]
  %indvars.iv4 = phi i65 [ 216, %b0 ], [ %indvars.iv.next5, %for.body.1.i.i.preheader ]
  %indvars.iv2 = phi i65 [ 54, %b0 ], [ %indvars.iv.next3, %for.body.1.i.i.preheader ]
  %indvars.iv = phi i64 [ 37, %b0 ], [ %indvars.iv.next, %for.body.1.i.i.preheader ]
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %for.body.1.i.i.preheader ]
  %0 = lshr exact i65 %indvars.iv10, 1
  %1 = trunc i65 %0 to i64
  %2 = add i64 %indvars.iv12, %1
  %3 = lshr exact i65 %indvars.iv6, 1
  %4 = trunc i65 %3 to i64
  %5 = mul i64 %4, 6148914691236517206
  %6 = mul i64 %1, 5
  %7 = add i64 %indvars.iv, %5
  %8 = add i64 %7, %6
  %v14_0.i9.i = mul i64 %2, %2
  %v4_0.i = sub i64 %v14_0.i9.i, %8
  %9 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %v4_0.i) #4
  %10 = tail call i32 @putchar(i32 10) #4
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %indvars.iv.next = add nuw nsw i64 %indvars.iv, 4
  %indvars.iv.next3 = add nuw nsw i65 %indvars.iv2, 6
  %indvars.iv.next5 = add i65 %indvars.iv4, %indvars.iv2
  %indvars.iv.next7 = add i65 %indvars.iv6, %indvars.iv4
  %indvars.iv.next9 = add nuw nsw i65 %indvars.iv8, 2
  %indvars.iv.next11 = add i65 %indvars.iv10, %indvars.iv8
  %indvars.iv.next13 = add nuw nsw i64 %indvars.iv12, 2
  %exitcond.not = icmp eq i64 %loop_counter_2, 30000
  br i1 %exitcond.not, label %loop_done, label %for.body.1.i.i.preheader

loop_done:                                        ; preds = %for.body.1.i.i.preheader
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %n) local_unnamed_addr #0 {
pre_entry:
  %v5_0.not1.i = icmp slt i64 %n, 1
  br i1 %v5_0.not1.i, label %__squareOfSum.exit, label %for.body.1.i

for.body.1.i:                                     ; preds = %pre_entry, %for.body.1.i
  %res_13.i = phi i64 [ %v11_0.i, %for.body.1.i ], [ 0, %pre_entry ]
  %i_12.i = phi i64 [ %v14_0.i, %for.body.1.i ], [ 1, %pre_entry ]
  %v8_0.i = mul i64 %i_12.i, %i_12.i
  %v11_0.i = add i64 %v8_0.i, %res_13.i
  %v14_0.i = add i64 %i_12.i, 1
  %v5_0.not.i = icmp sgt i64 %v14_0.i, %n
  br i1 %v5_0.not.i, label %for.body.1.i7, label %for.body.1.i

for.body.1.i7:                                    ; preds = %for.body.1.i, %for.body.1.i7
  %res_13.i2 = phi i64 [ %v8_0.i4, %for.body.1.i7 ], [ 0, %for.body.1.i ]
  %i_12.i3 = phi i64 [ %v11_0.i5, %for.body.1.i7 ], [ 1, %for.body.1.i ]
  %v8_0.i4 = add i64 %i_12.i3, %res_13.i2
  %v11_0.i5 = add i64 %i_12.i3, 1
  %v5_0.not.i6 = icmp sgt i64 %v11_0.i5, %n
  br i1 %v5_0.not.i6, label %__squareOfSum.exit, label %for.body.1.i7

__squareOfSum.exit:                               ; preds = %for.body.1.i7, %pre_entry
  %res_1.lcssa.i11 = phi i64 [ 0, %pre_entry ], [ %v11_0.i, %for.body.1.i7 ]
  %res_1.lcssa.i8 = phi i64 [ 0, %pre_entry ], [ %v8_0.i4, %for.body.1.i7 ]
  %v14_0.i9 = mul i64 %res_1.lcssa.i8, %res_1.lcssa.i8
  %v4_0 = sub i64 %v14_0.i9, %res_1.lcssa.i11
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %v4_0) #4
  %1 = tail call i32 @putchar(i32 10) #4
  ret void
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %for.body.1.i.i.preheader.i, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #5
  ret i32 0

for.body.1.i.i.preheader.i:                       ; preds = %0, %for.body.1.i.i.preheader.i
  %indvars.iv12.i = phi i64 [ %indvars.iv.next13.i, %for.body.1.i.i.preheader.i ], [ 19, %0 ]
  %indvars.iv10.i = phi i65 [ %indvars.iv.next11.i, %for.body.1.i.i.preheader.i ], [ 72, %0 ]
  %indvars.iv8.i = phi i65 [ %indvars.iv.next9.i, %for.body.1.i.i.preheader.i ], [ 18, %0 ]
  %indvars.iv6.i = phi i65 [ %indvars.iv.next7.i, %for.body.1.i.i.preheader.i ], [ 504, %0 ]
  %indvars.iv4.i = phi i65 [ %indvars.iv.next5.i, %for.body.1.i.i.preheader.i ], [ 216, %0 ]
  %indvars.iv2.i = phi i65 [ %indvars.iv.next3.i, %for.body.1.i.i.preheader.i ], [ 54, %0 ]
  %indvars.iv.i = phi i64 [ %indvars.iv.next.i, %for.body.1.i.i.preheader.i ], [ 37, %0 ]
  %loop_counter_11.i = phi i64 [ %loop_counter_2.i, %for.body.1.i.i.preheader.i ], [ 10, %0 ]
  %2 = lshr exact i65 %indvars.iv10.i, 1
  %3 = trunc i65 %2 to i64
  %4 = add i64 %indvars.iv12.i, %3
  %5 = lshr exact i65 %indvars.iv6.i, 1
  %6 = trunc i65 %5 to i64
  %v14_0.i9.i.i = mul i64 %4, %4
  %.neg = mul i64 %3, -5
  %.neg2 = mul i64 %6, -6148914691236517206
  %.neg3 = add i64 %v14_0.i9.i.i, %.neg
  %.neg4 = add i64 %.neg3, %.neg2
  %v4_0.i.i = sub i64 %.neg4, %indvars.iv.i
  %7 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %v4_0.i.i) #4
  %8 = tail call i32 @putchar(i32 10) #4
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %indvars.iv.next.i = add nuw nsw i64 %indvars.iv.i, 4
  %indvars.iv.next3.i = add nuw nsw i65 %indvars.iv2.i, 6
  %indvars.iv.next5.i = add i65 %indvars.iv2.i, %indvars.iv4.i
  %indvars.iv.next7.i = add i65 %indvars.iv4.i, %indvars.iv6.i
  %indvars.iv.next9.i = add nuw nsw i65 %indvars.iv8.i, 2
  %indvars.iv.next11.i = add i65 %indvars.iv8.i, %indvars.iv10.i
  %indvars.iv.next13.i = add nuw nsw i64 %indvars.iv12.i, 2
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 30000
  br i1 %exitcond.not.i, label %__main.exit, label %for.body.1.i.i.preheader.i

__main.exit:                                      ; preds = %for.body.1.i.i.preheader.i
  ret i32 0
}

; Function Attrs: cold minsize noreturn
define internal void @main.cold.1(i32 %0) #3 {
newFuncRoot:
  br label %1

1:                                                ; preds = %newFuncRoot
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([33 x i8], [33 x i8]* @.str.4, i64 0, i64 0), i32 0, i32 %0)
  tail call void @exit(i32 2)
  unreachable
}

attributes #0 = { nofree nounwind }
attributes #1 = { argmemonly mustprogress nofree norecurse nosync nounwind readonly willreturn }
attributes #2 = { nofree norecurse nosync nounwind readnone }
attributes #3 = { cold minsize noreturn }
attributes #4 = { nounwind }
attributes #5 = { noinline }
