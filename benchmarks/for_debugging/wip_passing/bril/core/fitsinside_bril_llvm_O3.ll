; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmplyMUzm/compile.ll'
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

; Function Attrs: nofree nounwind
define dso_local void @__main() local_unnamed_addr #0 {
b0:
  br label %loop2_cond.preheader

loop2_cond.preheader:                             ; preds = %b0, %loop2_done
  %loop_counter_14 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %loop2_done ]
  br label %loop3_cond.preheader

loop3_cond.preheader:                             ; preds = %loop2_cond.preheader, %loop3_done
  %loop2_counter_13 = phi i64 [ 10, %loop2_cond.preheader ], [ %loop2_counter_2, %loop3_done ]
  br label %loop4_cond.preheader

loop4_cond.preheader:                             ; preds = %loop3_cond.preheader, %loop4_done
  %loop3_counter_12 = phi i64 [ 10, %loop3_cond.preheader ], [ %loop3_counter_2, %loop4_done ]
  %width_check_0.i.i = icmp ule i64 %loop_counter_14, %loop3_counter_12
  %heightwidth_check_0.i.i = icmp ule i64 %loop2_counter_13, %loop3_counter_12
  br label %loop4_body

loop4_body:                                       ; preds = %loop4_cond.preheader, %loop4_body
  %loop4_counter_11 = phi i64 [ 10, %loop4_cond.preheader ], [ %loop4_counter_2, %loop4_body ]
  %height_check_0.i.i = icmp ule i64 %loop2_counter_13, %loop4_counter_11
  %first_check_0.i.i = and i1 %width_check_0.i.i, %height_check_0.i.i
  %widthheight_check_0.i.i = icmp ule i64 %loop_counter_14, %loop4_counter_11
  %second_check_0.i.i = and i1 %heightwidth_check_0.i.i, %widthheight_check_0.i.i
  %ret_val_0.i.i = or i1 %first_check_0.i.i, %second_check_0.i.i
  %..i.i = select i1 %ret_val_0.i.i, i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str, i64 0, i64 0), i8* getelementptr inbounds ([6 x i8], [6 x i8]* @.str.1, i64 0, i64 0)
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) %..i.i) #4
  %1 = tail call i32 @putchar(i32 10) #4
  %loop4_counter_2 = add nuw nsw i64 %loop4_counter_11, 1
  %exitcond.not = icmp eq i64 %loop4_counter_2, 40
  br i1 %exitcond.not, label %loop4_done, label %loop4_body

loop4_done:                                       ; preds = %loop4_body
  %loop3_counter_2 = add nuw nsw i64 %loop3_counter_12, 1
  %exitcond5.not = icmp eq i64 %loop3_counter_2, 40
  br i1 %exitcond5.not, label %loop3_done, label %loop4_cond.preheader

loop3_done:                                       ; preds = %loop4_done
  %loop2_counter_2 = add nuw nsw i64 %loop2_counter_13, 1
  %exitcond6.not = icmp eq i64 %loop2_counter_2, 40
  br i1 %exitcond6.not, label %loop2_done, label %loop3_cond.preheader

loop2_done:                                       ; preds = %loop3_done
  %loop_counter_2 = add nuw nsw i64 %loop_counter_14, 1
  %exitcond7.not = icmp eq i64 %loop_counter_2, 40
  br i1 %exitcond7.not, label %loop_done, label %loop2_cond.preheader

loop_done:                                        ; preds = %loop2_done
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %width1, i64 %height1, i64 %width2, i64 %height2) local_unnamed_addr #0 {
pre_entry:
  %width_check_0.i = icmp sle i64 %width1, %width2
  %height_check_0.i = icmp sle i64 %height1, %height2
  %first_check_0.i = and i1 %width_check_0.i, %height_check_0.i
  %widthheight_check_0.i = icmp sle i64 %width1, %height2
  %heightwidth_check_0.i = icmp sle i64 %height1, %width2
  %second_check_0.i = and i1 %heightwidth_check_0.i, %widthheight_check_0.i
  %ret_val_0.i = or i1 %first_check_0.i, %second_check_0.i
  %..i = select i1 %ret_val_0.i, i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str, i64 0, i64 0), i8* getelementptr inbounds ([6 x i8], [6 x i8]* @.str.1, i64 0, i64 0)
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) %..i) #4
  %1 = tail call i32 @putchar(i32 10) #4
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind readnone willreturn
define dso_local i1 @__fitsInside(i64 %w1, i64 %h1, i64 %w2, i64 %h2) local_unnamed_addr #2 {
pre_entry:
  %width_check_0 = icmp sle i64 %w1, %w2
  %height_check_0 = icmp sle i64 %h1, %h2
  %first_check_0 = and i1 %width_check_0, %height_check_0
  %widthheight_check_0 = icmp sle i64 %w1, %h2
  %heightwidth_check_0 = icmp sle i64 %h1, %w2
  %second_check_0 = and i1 %heightwidth_check_0, %widthheight_check_0
  %ret_val_0 = or i1 %first_check_0, %second_check_0
  ret i1 %ret_val_0
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %loop2_cond.preheader.i, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #5
  ret i32 0

loop2_cond.preheader.i:                           ; preds = %0, %loop2_done.i
  %loop_counter_14.i = phi i64 [ %loop_counter_2.i, %loop2_done.i ], [ 10, %0 ]
  br label %loop3_cond.preheader.i

loop3_cond.preheader.i:                           ; preds = %loop3_done.i, %loop2_cond.preheader.i
  %loop2_counter_13.i = phi i64 [ 10, %loop2_cond.preheader.i ], [ %loop2_counter_2.i, %loop3_done.i ]
  br label %loop4_cond.preheader.i

loop4_cond.preheader.i:                           ; preds = %loop4_done.i, %loop3_cond.preheader.i
  %loop3_counter_12.i = phi i64 [ 10, %loop3_cond.preheader.i ], [ %loop3_counter_2.i, %loop4_done.i ]
  %width_check_0.i.i.i = icmp ule i64 %loop_counter_14.i, %loop3_counter_12.i
  %heightwidth_check_0.i.i.i = icmp ule i64 %loop2_counter_13.i, %loop3_counter_12.i
  br label %loop4_body.i

loop4_body.i:                                     ; preds = %loop4_body.i, %loop4_cond.preheader.i
  %loop4_counter_11.i = phi i64 [ 10, %loop4_cond.preheader.i ], [ %loop4_counter_2.i, %loop4_body.i ]
  %height_check_0.i.i.i = icmp ule i64 %loop2_counter_13.i, %loop4_counter_11.i
  %first_check_0.i.i.i = and i1 %width_check_0.i.i.i, %height_check_0.i.i.i
  %widthheight_check_0.i.i.i = icmp ule i64 %loop_counter_14.i, %loop4_counter_11.i
  %second_check_0.i.i.i = and i1 %heightwidth_check_0.i.i.i, %widthheight_check_0.i.i.i
  %ret_val_0.i.i.i = or i1 %first_check_0.i.i.i, %second_check_0.i.i.i
  %..i.i.i = select i1 %ret_val_0.i.i.i, i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str, i64 0, i64 0), i8* getelementptr inbounds ([6 x i8], [6 x i8]* @.str.1, i64 0, i64 0)
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) %..i.i.i) #4
  %3 = tail call i32 @putchar(i32 10) #4
  %loop4_counter_2.i = add nuw nsw i64 %loop4_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop4_counter_2.i, 40
  br i1 %exitcond.not.i, label %loop4_done.i, label %loop4_body.i

loop4_done.i:                                     ; preds = %loop4_body.i
  %loop3_counter_2.i = add nuw nsw i64 %loop3_counter_12.i, 1
  %exitcond5.not.i = icmp eq i64 %loop3_counter_2.i, 40
  br i1 %exitcond5.not.i, label %loop3_done.i, label %loop4_cond.preheader.i

loop3_done.i:                                     ; preds = %loop4_done.i
  %loop2_counter_2.i = add nuw nsw i64 %loop2_counter_13.i, 1
  %exitcond6.not.i = icmp eq i64 %loop2_counter_2.i, 40
  br i1 %exitcond6.not.i, label %loop2_done.i, label %loop3_cond.preheader.i

loop2_done.i:                                     ; preds = %loop3_done.i
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_14.i, 1
  %exitcond7.not.i = icmp eq i64 %loop_counter_2.i, 40
  br i1 %exitcond7.not.i, label %__main.exit, label %loop2_cond.preheader.i

__main.exit:                                      ; preds = %loop2_done.i
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
attributes #2 = { mustprogress nofree norecurse nosync nounwind readnone willreturn }
attributes #3 = { cold minsize noreturn }
attributes #4 = { nounwind }
attributes #5 = { noinline }
