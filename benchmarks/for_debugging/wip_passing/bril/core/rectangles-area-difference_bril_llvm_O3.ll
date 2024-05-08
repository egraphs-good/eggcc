; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpxgDFMj/compile.ll'
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
  %area_0.i.i = mul nuw nsw i64 %loop2_counter_13, %loop_counter_14
  br label %loop4_cond.preheader

loop4_cond.preheader:                             ; preds = %loop3_cond.preheader, %loop4_done
  %loop3_counter_12 = phi i64 [ 10, %loop3_cond.preheader ], [ %loop3_counter_2, %loop4_done ]
  br label %loop4_body

loop4_body:                                       ; preds = %loop4_cond.preheader, %loop4_body
  %loop4_counter_11 = phi i64 [ 10, %loop4_cond.preheader ], [ %loop4_counter_2, %loop4_body ]
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %area_0.i.i) #3
  %1 = tail call i32 @putchar(i32 10) #3
  %area_0.i1.i = mul nuw nsw i64 %loop4_counter_11, %loop3_counter_12
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %area_0.i1.i) #3
  %3 = tail call i32 @putchar(i32 10) #3
  %res_0.i = sub nsw i64 %area_0.i.i, %area_0.i1.i
  %a1_bigger_0.i = icmp ugt i64 %area_0.i.i, %area_0.i1.i
  %res_1.i = sub nsw i64 0, %res_0.i
  %spec.select.i = select i1 %a1_bigger_0.i, i64 %res_0.i, i64 %res_1.i
  %4 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %spec.select.i) #3
  %5 = tail call i32 @putchar(i32 10) #3
  %loop4_counter_2 = add nuw nsw i64 %loop4_counter_11, 1
  %exitcond.not = icmp eq i64 %loop4_counter_2, 30
  br i1 %exitcond.not, label %loop4_done, label %loop4_body

loop4_done:                                       ; preds = %loop4_body
  %loop3_counter_2 = add nuw nsw i64 %loop3_counter_12, 1
  %exitcond5.not = icmp eq i64 %loop3_counter_2, 30
  br i1 %exitcond5.not, label %loop3_done, label %loop4_cond.preheader

loop3_done:                                       ; preds = %loop4_done
  %loop2_counter_2 = add nuw nsw i64 %loop2_counter_13, 1
  %exitcond6.not = icmp eq i64 %loop2_counter_2, 30
  br i1 %exitcond6.not, label %loop2_done, label %loop3_cond.preheader

loop2_done:                                       ; preds = %loop3_done
  %loop_counter_2 = add nuw nsw i64 %loop_counter_14, 1
  %exitcond7.not = icmp eq i64 %loop_counter_2, 30
  br i1 %exitcond7.not, label %loop_done, label %loop2_cond.preheader

loop_done:                                        ; preds = %loop2_done
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %x1, i64 %y1, i64 %x2, i64 %y2) local_unnamed_addr #0 {
pre_entry:
  %area_0.i = mul i64 %y1, %x1
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %area_0.i) #3
  %1 = tail call i32 @putchar(i32 10) #3
  %area_0.i1 = mul i64 %y2, %x2
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %area_0.i1) #3
  %3 = tail call i32 @putchar(i32 10) #3
  %res_0 = sub i64 %area_0.i, %area_0.i1
  %a1_bigger_0 = icmp sgt i64 %area_0.i, %area_0.i1
  %res_1 = sub i64 0, %res_0
  %spec.select = select i1 %a1_bigger_0, i64 %res_0, i64 %res_1
  %4 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %spec.select) #3
  %5 = tail call i32 @putchar(i32 10) #3
  ret void
}

; Function Attrs: nofree nounwind
define dso_local i64 @__area(i64 %x, i64 %y) local_unnamed_addr #0 {
pre_entry:
  %area_0 = mul i64 %y, %x
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %area_0) #3
  %1 = tail call i32 @putchar(i32 10) #3
  ret i64 %area_0
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %2, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #4
  ret i32 0

2:                                                ; preds = %0
  tail call void @__main()
  ret i32 0
}

; Function Attrs: cold minsize noreturn
define internal void @main.cold.1(i32 %0) #2 {
newFuncRoot:
  br label %1

1:                                                ; preds = %newFuncRoot
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([33 x i8], [33 x i8]* @.str.4, i64 0, i64 0), i32 0, i32 %0)
  tail call void @exit(i32 2)
  unreachable
}

attributes #0 = { nofree nounwind }
attributes #1 = { argmemonly mustprogress nofree norecurse nosync nounwind readonly willreturn }
attributes #2 = { cold minsize noreturn }
attributes #3 = { nounwind }
attributes #4 = { noinline }
