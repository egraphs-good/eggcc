; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpfDHo9l/compile.ll'
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

; Function Attrs: mustprogress nofree nounwind willreturn
define dso_local noalias i64* @__pack(i64 %size, i64 %n1, i64 %n2, i64 %n3, i64 %n4, i64 %n5) local_unnamed_addr #4 {
pre_entry:
  %z0 = shl i64 %size, 3
  %z1 = tail call i8* @malloc(i64 %z0)
  %array_0 = bitcast i8* %z1 to i64*
  store i64 %n1, i64* %array_0, align 8
  %loc_1 = getelementptr inbounds i64, i64* %array_0, i64 1
  store i64 %n2, i64* %loc_1, align 8
  %loc_2 = getelementptr inbounds i64, i64* %array_0, i64 2
  store i64 %n3, i64* %loc_2, align 8
  %loc_3 = getelementptr inbounds i64, i64* %array_0, i64 3
  store i64 %n4, i64* %loc_3, align 8
  %loc_4 = getelementptr inbounds i64, i64* %array_0, i64 4
  store i64 %n5, i64* %loc_4, align 8
  ret i64* %array_0
}

; Function Attrs: nofree nounwind
define dso_local void @__print_array(i64* nocapture readonly %array, i64 %size) local_unnamed_addr #0 {
pre_entry:
  %cond_01 = icmp sgt i64 %size, 0
  br i1 %cond_01, label %body, label %done

body:                                             ; preds = %pre_entry, %body
  %i_12 = phi i64 [ %i_2, %body ], [ 0, %pre_entry ]
  %loc_0 = getelementptr inbounds i64, i64* %array, i64 %i_12
  %val_0 = load i64, i64* %loc_0, align 8
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0) #6
  %1 = tail call i32 @putchar(i32 10) #6
  %i_2 = add nuw nsw i64 %i_12, 1
  %exitcond.not = icmp eq i64 %i_2, %size
  br i1 %exitcond.not, label %done, label %body

done:                                             ; preds = %body, %pre_entry
  ret void
}

; Function Attrs: argmemonly nofree nosync nounwind readonly
define dso_local i64 @__binary_search(i64* readonly %array, i64 %target, i64 %left, i64 %right) local_unnamed_addr #5 {
pre_entry:
  %end_cond_026 = icmp sgt i64 %left, %right
  br i1 %end_cond_026, label %common.ret, label %body.lr.ph

body.lr.ph:                                       ; preds = %pre_entry, %call_gt
  %right.tr.ph8 = phi i64 [ %right.tr3, %call_gt ], [ %right, %pre_entry ]
  %left.tr.ph7 = phi i64 [ %newleft_0, %call_gt ], [ %left, %pre_entry ]
  br label %body

body:                                             ; preds = %body.lr.ph, %call_lt
  %right.tr3 = phi i64 [ %right.tr.ph8, %body.lr.ph ], [ %newright_0, %call_lt ]
  %mid_0 = add i64 %right.tr3, %left.tr.ph7
  %mid_1 = sdiv i64 %mid_0, 2
  %midloc_0 = getelementptr inbounds i64, i64* %array, i64 %mid_1
  %midval_0 = load i64, i64* %midloc_0, align 8
  %equal_cond_0 = icmp eq i64 %midval_0, %target
  br i1 %equal_cond_0, label %common.ret, label %check_gt

common.ret:                                       ; preds = %call_gt, %body, %call_lt, %pre_entry
  %common.ret.op = phi i64 [ -1, %pre_entry ], [ -1, %call_lt ], [ %mid_1, %body ], [ -1, %call_gt ]
  ret i64 %common.ret.op

check_gt:                                         ; preds = %body
  %gt_cond_0 = icmp slt i64 %midval_0, %target
  br i1 %gt_cond_0, label %call_gt, label %call_lt

call_gt:                                          ; preds = %check_gt
  %newleft_0 = add i64 %left.tr.ph7, 1
  %end_cond_02 = icmp sgt i64 %newleft_0, %right.tr3
  br i1 %end_cond_02, label %common.ret, label %body.lr.ph

call_lt:                                          ; preds = %check_gt
  %newright_0 = add i64 %right.tr3, -1
  %end_cond_0 = icmp sgt i64 %left.tr.ph7, %newright_0
  br i1 %end_cond_0, label %common.ret, label %body
}

; Function Attrs: nounwind
define dso_local void @__orig_main(i64 %e1, i64 %e2, i64 %e3, i64 %e4, i64 %e5) local_unnamed_addr #6 {
pre_entry:
  %z1.i = tail call dereferenceable_or_null(40) i8* @malloc(i64 40) #6
  %array_0.i = bitcast i8* %z1.i to i64*
  store i64 %e1, i64* %array_0.i, align 8
  %loc_1.i = getelementptr inbounds i64, i64* %array_0.i, i64 1
  store i64 %e2, i64* %loc_1.i, align 8
  %loc_2.i = getelementptr inbounds i64, i64* %array_0.i, i64 2
  store i64 %e3, i64* %loc_2.i, align 8
  %loc_3.i = getelementptr inbounds i64, i64* %array_0.i, i64 3
  store i64 %e4, i64* %loc_3.i, align 8
  %loc_4.i = getelementptr inbounds i64, i64* %array_0.i, i64 4
  store i64 %e5, i64* %loc_4.i, align 8
  br label %body.lr.ph.i

body.lr.ph.i:                                     ; preds = %call_gt.i, %pre_entry
  %right.tr.ph8.i = phi i64 [ %right.tr3.i, %call_gt.i ], [ 4, %pre_entry ]
  %left.tr.ph7.i = phi i64 [ %newleft_0.i, %call_gt.i ], [ 0, %pre_entry ]
  br label %body.i

body.i:                                           ; preds = %call_lt.i, %body.lr.ph.i
  %right.tr3.i = phi i64 [ %right.tr.ph8.i, %body.lr.ph.i ], [ %newright_0.i, %call_lt.i ]
  %mid_0.i = add i64 %right.tr3.i, %left.tr.ph7.i
  %mid_1.i = sdiv i64 %mid_0.i, 2
  %midloc_0.i = getelementptr inbounds i64, i64* %array_0.i, i64 %mid_1.i
  %midval_0.i = load i64, i64* %midloc_0.i, align 8
  %equal_cond_0.i = icmp eq i64 %midval_0.i, 7
  br i1 %equal_cond_0.i, label %__binary_search.exit, label %check_gt.i

check_gt.i:                                       ; preds = %body.i
  %gt_cond_0.i = icmp slt i64 %midval_0.i, 7
  br i1 %gt_cond_0.i, label %call_gt.i, label %call_lt.i

call_gt.i:                                        ; preds = %check_gt.i
  %newleft_0.i = add i64 %left.tr.ph7.i, 1
  %end_cond_02.i = icmp sgt i64 %newleft_0.i, %right.tr3.i
  br i1 %end_cond_02.i, label %__binary_search.exit, label %body.lr.ph.i

call_lt.i:                                        ; preds = %check_gt.i
  %newright_0.i = add i64 %right.tr3.i, -1
  %end_cond_0.i = icmp sgt i64 %left.tr.ph7.i, %newright_0.i
  br i1 %end_cond_0.i, label %__binary_search.exit, label %body.i

__binary_search.exit:                             ; preds = %call_gt.i, %body.i, %call_lt.i
  %common.ret.op.i = phi i64 [ %mid_1.i, %body.i ], [ -1, %call_lt.i ], [ -1, %call_gt.i ]
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %common.ret.op.i) #6
  %1 = tail call i32 @putchar(i32 10) #6
  tail call void @free(i8* %z1.i)
  ret void
}

; Function Attrs: nounwind
define dso_local void @__main() local_unnamed_addr #6 {
b0:
  br label %loop2_cond.preheader

loop2_cond.preheader:                             ; preds = %b0, %loop2_done
  %loop_counter_17 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %loop2_done ]
  br label %loop3_cond.preheader

loop3_cond.preheader:                             ; preds = %loop2_cond.preheader, %loop3_done
  %loop2_counter_16 = phi i64 [ 10, %loop2_cond.preheader ], [ %loop2_counter_2, %loop3_done ]
  br label %loop4_cond.preheader

loop4_cond.preheader:                             ; preds = %loop3_cond.preheader, %loop4_done
  %loop3_counter_15 = phi i64 [ 10, %loop3_cond.preheader ], [ %loop3_counter_2, %loop4_done ]
  br label %loop5_cond.preheader

loop5_cond.preheader:                             ; preds = %loop4_cond.preheader, %loop5_done
  %loop4_counter_14 = phi i64 [ 10, %loop4_cond.preheader ], [ %loop4_counter_2, %loop5_done ]
  br label %loop5_body

loop5_body:                                       ; preds = %loop5_cond.preheader, %__orig_main.exit
  %loop5_counter_13 = phi i64 [ 10, %loop5_cond.preheader ], [ %loop5_counter_2, %__orig_main.exit ]
  %z1.i.i = tail call dereferenceable_or_null(40) i8* @malloc(i64 40) #6
  %array_0.i.i = bitcast i8* %z1.i.i to i64*
  store i64 %loop_counter_17, i64* %array_0.i.i, align 8
  %loc_1.i.i = getelementptr inbounds i64, i64* %array_0.i.i, i64 1
  store i64 %loop2_counter_16, i64* %loc_1.i.i, align 8
  %loc_2.i.i = getelementptr inbounds i64, i64* %array_0.i.i, i64 2
  store i64 %loop3_counter_15, i64* %loc_2.i.i, align 8
  %loc_3.i.i = getelementptr inbounds i64, i64* %array_0.i.i, i64 3
  store i64 %loop4_counter_14, i64* %loc_3.i.i, align 8
  %loc_4.i.i = getelementptr inbounds i64, i64* %array_0.i.i, i64 4
  store i64 %loop5_counter_13, i64* %loc_4.i.i, align 8
  br label %body.lr.ph.i.i

body.lr.ph.i.i:                                   ; preds = %call_gt.i.i, %loop5_body
  %right.tr.ph8.i.i = phi i64 [ %right.tr3.i.i, %call_gt.i.i ], [ 4, %loop5_body ]
  %left.tr.ph7.i.i = phi i64 [ %newleft_0.i.i, %call_gt.i.i ], [ 0, %loop5_body ]
  br label %body.i.i

body.i.i:                                         ; preds = %call_lt.i.i, %body.lr.ph.i.i
  %right.tr3.i.i = phi i64 [ %right.tr.ph8.i.i, %body.lr.ph.i.i ], [ %newright_0.i.i, %call_lt.i.i ]
  %mid_0.i.i = add i64 %right.tr3.i.i, %left.tr.ph7.i.i
  %mid_1.i.i = sdiv i64 %mid_0.i.i, 2
  %midloc_0.i.i = getelementptr inbounds i64, i64* %array_0.i.i, i64 %mid_1.i.i
  %midval_0.i.i = load i64, i64* %midloc_0.i.i, align 8
  %equal_cond_0.i.i = icmp eq i64 %midval_0.i.i, 7
  br i1 %equal_cond_0.i.i, label %__orig_main.exit, label %check_gt.i.i

check_gt.i.i:                                     ; preds = %body.i.i
  %gt_cond_0.i.i = icmp slt i64 %midval_0.i.i, 7
  br i1 %gt_cond_0.i.i, label %call_gt.i.i, label %call_lt.i.i

call_gt.i.i:                                      ; preds = %check_gt.i.i
  %newleft_0.i.i = add i64 %left.tr.ph7.i.i, 1
  %end_cond_02.i.i = icmp sgt i64 %newleft_0.i.i, %right.tr3.i.i
  br i1 %end_cond_02.i.i, label %__orig_main.exit, label %body.lr.ph.i.i

call_lt.i.i:                                      ; preds = %check_gt.i.i
  %newright_0.i.i = add i64 %right.tr3.i.i, -1
  %end_cond_0.i.i = icmp sgt i64 %left.tr.ph7.i.i, %newright_0.i.i
  br i1 %end_cond_0.i.i, label %__orig_main.exit, label %body.i.i

__orig_main.exit:                                 ; preds = %call_gt.i.i, %body.i.i, %call_lt.i.i
  %common.ret.op.i.i = phi i64 [ -1, %call_lt.i.i ], [ %mid_1.i.i, %body.i.i ], [ -1, %call_gt.i.i ]
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %common.ret.op.i.i) #6
  %1 = tail call i32 @putchar(i32 10) #6
  tail call void @free(i8* %z1.i.i) #6
  %loop5_counter_2 = add nuw nsw i64 %loop5_counter_13, 1
  %exitcond.not = icmp eq i64 %loop5_counter_2, 25
  br i1 %exitcond.not, label %loop5_done, label %loop5_body

loop5_done:                                       ; preds = %__orig_main.exit
  %loop4_counter_2 = add nuw nsw i64 %loop4_counter_14, 1
  %exitcond10.not = icmp eq i64 %loop4_counter_2, 25
  br i1 %exitcond10.not, label %loop4_done, label %loop5_cond.preheader

loop4_done:                                       ; preds = %loop5_done
  %loop3_counter_2 = add nuw nsw i64 %loop3_counter_15, 1
  %exitcond11.not = icmp eq i64 %loop3_counter_2, 25
  br i1 %exitcond11.not, label %loop3_done, label %loop4_cond.preheader

loop3_done:                                       ; preds = %loop4_done
  %loop2_counter_2 = add nuw nsw i64 %loop2_counter_16, 1
  %exitcond12.not = icmp eq i64 %loop2_counter_2, 25
  br i1 %exitcond12.not, label %loop2_done, label %loop3_cond.preheader

loop2_done:                                       ; preds = %loop3_done
  %loop_counter_2 = add nuw nsw i64 %loop_counter_17, 1
  %exitcond13.not = icmp eq i64 %loop_counter_2, 25
  br i1 %exitcond13.not, label %loop_done, label %loop2_cond.preheader

loop_done:                                        ; preds = %loop2_done
  ret void
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %2, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #8
  ret i32 0

2:                                                ; preds = %0
  tail call void @__main()
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
attributes #4 = { mustprogress nofree nounwind willreturn }
attributes #5 = { argmemonly nofree nosync nounwind readonly }
attributes #6 = { nounwind }
attributes #7 = { cold minsize noreturn }
attributes #8 = { noinline }
