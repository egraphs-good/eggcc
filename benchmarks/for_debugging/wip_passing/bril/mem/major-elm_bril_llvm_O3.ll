; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpLzWSv3/compile.ll'
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
  br label %loop2_cond.preheader

loop2_cond.preheader:                             ; preds = %b0, %loop2_done
  %loop_counter_16 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %loop2_done ]
  br label %loop2_body

loop2_body:                                       ; preds = %loop2_cond.preheader, %__orig_main.exit
  %loop2_counter_15 = phi i64 [ 10, %loop2_cond.preheader ], [ %loop2_counter_2, %__orig_main.exit ]
  %z1.i.i = tail call dereferenceable_or_null(24) i8* @malloc(i64 24) #4
  %array_0.i.i = bitcast i8* %z1.i.i to i64*
  store i64 %loop_counter_16, i64* %array_0.i.i, align 8
  %loc_1.i.i = getelementptr inbounds i64, i64* %array_0.i.i, i64 1
  store i64 %loop2_counter_15, i64* %loc_1.i.i, align 8
  %loc_2.i.i = getelementptr inbounds i64, i64* %array_0.i.i, i64 2
  store i64 %loop_counter_16, i64* %loc_2.i.i, align 8
  br label %body.lr.ph.i

body.lr.ph.i:                                     ; preds = %eq_zero_if.i, %loop2_body
  %major_elm_1.ph7.i = phi i64 [ %loop_counter_16, %loop2_body ], [ %cur_val_0.i, %eq_zero_if.i ]
  %i_1.ph6.i = phi i64 [ 1, %loop2_body ], [ %i_3.i, %eq_zero_if.i ]
  %smax.i = tail call i64 @llvm.smax.i64(i64 %i_1.ph6.i, i64 2) #4
  br label %body.i

body.i:                                           ; preds = %check_bound.backedge.i, %body.lr.ph.i
  %count_14.i = phi i64 [ 1, %body.lr.ph.i ], [ %count_2.i, %check_bound.backedge.i ]
  %i_13.i = phi i64 [ %i_1.ph6.i, %body.lr.ph.i ], [ %i_1.be.i, %check_bound.backedge.i ]
  %cur_ptr_0.i = getelementptr inbounds i64, i64* %array_0.i.i, i64 %i_13.i
  %cur_val_0.i = load i64, i64* %cur_ptr_0.i, align 8
  %cur_major_cond_0.i = icmp eq i64 %cur_val_0.i, %major_elm_1.ph7.i
  br i1 %cur_major_cond_0.i, label %check_bound.backedge.i, label %body.else.i

check_bound.backedge.i:                           ; preds = %body.else.i, %body.i
  %.sink.i = phi i64 [ -1, %body.else.i ], [ 1, %body.i ]
  %count_2.i = add i64 %.sink.i, %count_14.i
  %i_1.be.i = add i64 %i_13.i, 1
  %exitcond.i = icmp eq i64 %i_13.i, %smax.i
  br i1 %exitcond.i, label %__orig_main.exit, label %body.i

body.else.i:                                      ; preds = %body.i
  %cnt_eq_0_0.i = icmp eq i64 %count_14.i, 0
  br i1 %cnt_eq_0_0.i, label %eq_zero_if.i, label %check_bound.backedge.i

eq_zero_if.i:                                     ; preds = %body.else.i
  %i_3.i = add nsw i64 %i_13.i, 1
  %end_cond_02.i = icmp sgt i64 %i_13.i, 1
  br i1 %end_cond_02.i, label %__orig_main.exit, label %body.lr.ph.i

__orig_main.exit:                                 ; preds = %eq_zero_if.i, %check_bound.backedge.i
  %major_elm_1.ph.lcssa.i = phi i64 [ %major_elm_1.ph7.i, %check_bound.backedge.i ], [ %cur_val_0.i, %eq_zero_if.i ]
  tail call void @free(i8* nonnull %z1.i.i) #4
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %major_elm_1.ph.lcssa.i) #4
  %1 = tail call i32 @putchar(i32 10) #4
  %loop2_counter_2 = add nuw nsw i64 %loop2_counter_15, 1
  %exitcond.not = icmp eq i64 %loop2_counter_2, 1000
  br i1 %exitcond.not, label %loop2_done, label %loop2_body

loop2_done:                                       ; preds = %__orig_main.exit
  %loop_counter_2 = add nuw nsw i64 %loop_counter_16, 1
  %exitcond11.not = icmp eq i64 %loop_counter_2, 1000
  br i1 %exitcond11.not, label %loop_done, label %loop2_cond.preheader

loop_done:                                        ; preds = %loop2_done
  ret void
}

; Function Attrs: nounwind
define dso_local void @__orig_main(i64 %e1, i64 %e2, i64 %e3) local_unnamed_addr #4 {
pre_entry:
  %z1.i = tail call dereferenceable_or_null(24) i8* @malloc(i64 24) #4
  %array_0.i = bitcast i8* %z1.i to i64*
  store i64 %e1, i64* %array_0.i, align 8
  %loc_1.i = getelementptr inbounds i64, i64* %array_0.i, i64 1
  store i64 %e2, i64* %loc_1.i, align 8
  %loc_2.i = getelementptr inbounds i64, i64* %array_0.i, i64 2
  store i64 %e3, i64* %loc_2.i, align 8
  br label %body.lr.ph

body.lr.ph:                                       ; preds = %pre_entry, %eq_zero_if
  %major_elm_1.ph7 = phi i64 [ %e1, %pre_entry ], [ %cur_val_0, %eq_zero_if ]
  %i_1.ph6 = phi i64 [ 1, %pre_entry ], [ %i_3, %eq_zero_if ]
  %smax = call i64 @llvm.smax.i64(i64 %i_1.ph6, i64 2)
  br label %body

body:                                             ; preds = %body.lr.ph, %check_bound.backedge
  %count_14 = phi i64 [ 1, %body.lr.ph ], [ %count_2, %check_bound.backedge ]
  %i_13 = phi i64 [ %i_1.ph6, %body.lr.ph ], [ %i_1.be, %check_bound.backedge ]
  %cur_ptr_0 = getelementptr inbounds i64, i64* %array_0.i, i64 %i_13
  %cur_val_0 = load i64, i64* %cur_ptr_0, align 8
  %cur_major_cond_0 = icmp eq i64 %cur_val_0, %major_elm_1.ph7
  br i1 %cur_major_cond_0, label %check_bound.backedge, label %body.else

check_bound.backedge:                             ; preds = %body, %body.else
  %.sink = phi i64 [ -1, %body.else ], [ 1, %body ]
  %count_2 = add i64 %count_14, %.sink
  %i_1.be = add i64 %i_13, 1
  %exitcond = icmp eq i64 %i_13, %smax
  br i1 %exitcond, label %end, label %body

body.else:                                        ; preds = %body
  %cnt_eq_0_0 = icmp eq i64 %count_14, 0
  br i1 %cnt_eq_0_0, label %eq_zero_if, label %check_bound.backedge

eq_zero_if:                                       ; preds = %body.else
  %i_3 = add nsw i64 %i_13, 1
  %end_cond_02 = icmp sgt i64 %i_13, 1
  br i1 %end_cond_02, label %end, label %body.lr.ph

end:                                              ; preds = %eq_zero_if, %check_bound.backedge
  %major_elm_1.ph.lcssa = phi i64 [ %major_elm_1.ph7, %check_bound.backedge ], [ %cur_val_0, %eq_zero_if ]
  tail call void @free(i8* nonnull %z1.i)
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %major_elm_1.ph.lcssa) #4
  %1 = tail call i32 @putchar(i32 10) #4
  ret void
}

; Function Attrs: mustprogress nofree nounwind willreturn
define dso_local noalias i64* @__create_arr(i64 %size, i64 %e1, i64 %e2, i64 %e3) local_unnamed_addr #5 {
pre_entry:
  %z0 = shl i64 %size, 3
  %z1 = tail call i8* @malloc(i64 %z0)
  %array_0 = bitcast i8* %z1 to i64*
  store i64 %e1, i64* %array_0, align 8
  %loc_1 = getelementptr inbounds i64, i64* %array_0, i64 1
  store i64 %e2, i64* %loc_1, align 8
  %loc_2 = getelementptr inbounds i64, i64* %array_0, i64 2
  store i64 %e3, i64* %loc_2, align 8
  ret i64* %array_0
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %loop2_cond.preheader.i, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #8
  ret i32 0

loop2_cond.preheader.i:                           ; preds = %0, %loop2_done.i
  %loop_counter_16.i = phi i64 [ %loop_counter_2.i, %loop2_done.i ], [ 10, %0 ]
  br label %loop2_body.i

loop2_body.i:                                     ; preds = %__orig_main.exit.i, %loop2_cond.preheader.i
  %loop2_counter_15.i = phi i64 [ 10, %loop2_cond.preheader.i ], [ %loop2_counter_2.i, %__orig_main.exit.i ]
  %z1.i.i.i = tail call dereferenceable_or_null(24) i8* @malloc(i64 24) #4
  %array_0.i.i.i = bitcast i8* %z1.i.i.i to i64*
  store i64 %loop_counter_16.i, i64* %array_0.i.i.i, align 8
  %loc_1.i.i.i = getelementptr inbounds i64, i64* %array_0.i.i.i, i64 1
  store i64 %loop2_counter_15.i, i64* %loc_1.i.i.i, align 8
  %loc_2.i.i.i = getelementptr inbounds i64, i64* %array_0.i.i.i, i64 2
  store i64 %loop_counter_16.i, i64* %loc_2.i.i.i, align 8
  br label %body.lr.ph.i.i

body.lr.ph.i.i:                                   ; preds = %eq_zero_if.i.i, %loop2_body.i
  %major_elm_1.ph7.i.i = phi i64 [ %loop_counter_16.i, %loop2_body.i ], [ %cur_val_0.i.i, %eq_zero_if.i.i ]
  %i_1.ph6.i.i = phi i64 [ 1, %loop2_body.i ], [ %i_3.i.i, %eq_zero_if.i.i ]
  %smax.i.i = tail call i64 @llvm.smax.i64(i64 %i_1.ph6.i.i, i64 2) #4
  br label %body.i.i

body.i.i:                                         ; preds = %check_bound.backedge.i.i, %body.lr.ph.i.i
  %count_14.i.i = phi i64 [ 1, %body.lr.ph.i.i ], [ %count_2.i.i, %check_bound.backedge.i.i ]
  %i_13.i.i = phi i64 [ %i_1.ph6.i.i, %body.lr.ph.i.i ], [ %i_1.be.i.i, %check_bound.backedge.i.i ]
  %cur_ptr_0.i.i = getelementptr inbounds i64, i64* %array_0.i.i.i, i64 %i_13.i.i
  %cur_val_0.i.i = load i64, i64* %cur_ptr_0.i.i, align 8
  %cur_major_cond_0.i.i = icmp eq i64 %cur_val_0.i.i, %major_elm_1.ph7.i.i
  br i1 %cur_major_cond_0.i.i, label %check_bound.backedge.i.i, label %body.else.i.i

check_bound.backedge.i.i:                         ; preds = %body.else.i.i, %body.i.i
  %.sink.i.i = phi i64 [ -1, %body.else.i.i ], [ 1, %body.i.i ]
  %count_2.i.i = add i64 %.sink.i.i, %count_14.i.i
  %i_1.be.i.i = add i64 %i_13.i.i, 1
  %exitcond.i.i = icmp eq i64 %i_13.i.i, %smax.i.i
  br i1 %exitcond.i.i, label %__orig_main.exit.i, label %body.i.i

body.else.i.i:                                    ; preds = %body.i.i
  %cnt_eq_0_0.i.i = icmp eq i64 %count_14.i.i, 0
  br i1 %cnt_eq_0_0.i.i, label %eq_zero_if.i.i, label %check_bound.backedge.i.i

eq_zero_if.i.i:                                   ; preds = %body.else.i.i
  %i_3.i.i = add nsw i64 %i_13.i.i, 1
  %end_cond_02.i.i = icmp sgt i64 %i_13.i.i, 1
  br i1 %end_cond_02.i.i, label %__orig_main.exit.i, label %body.lr.ph.i.i

__orig_main.exit.i:                               ; preds = %eq_zero_if.i.i, %check_bound.backedge.i.i
  %major_elm_1.ph.lcssa.i.i = phi i64 [ %major_elm_1.ph7.i.i, %check_bound.backedge.i.i ], [ %cur_val_0.i.i, %eq_zero_if.i.i ]
  tail call void @free(i8* nonnull %z1.i.i.i) #4
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %major_elm_1.ph.lcssa.i.i) #4
  %3 = tail call i32 @putchar(i32 10) #4
  %loop2_counter_2.i = add nuw nsw i64 %loop2_counter_15.i, 1
  %exitcond.not.i = icmp eq i64 %loop2_counter_2.i, 1000
  br i1 %exitcond.not.i, label %loop2_done.i, label %loop2_body.i

loop2_done.i:                                     ; preds = %__orig_main.exit.i
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_16.i, 1
  %exitcond11.not.i = icmp eq i64 %loop_counter_2.i, 1000
  br i1 %exitcond11.not.i, label %__main.exit, label %loop2_cond.preheader.i

__main.exit:                                      ; preds = %loop2_done.i
  ret i32 0
}

; Function Attrs: nocallback nofree nosync nounwind readnone speculatable willreturn
declare i64 @llvm.smax.i64(i64, i64) #6

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
attributes #4 = { nounwind }
attributes #5 = { mustprogress nofree nounwind willreturn }
attributes #6 = { nocallback nofree nosync nounwind readnone speculatable willreturn }
attributes #7 = { cold minsize noreturn }
attributes #8 = { noinline }
