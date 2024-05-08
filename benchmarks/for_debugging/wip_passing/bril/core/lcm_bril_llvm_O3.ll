; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmp4dqnRU/compile.ll'
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
  br label %inner_cond.preheader

inner_cond.preheader:                             ; preds = %b0, %inner_done
  %loop_counter_12 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %inner_done ]
  br label %inner_body

inner_body:                                       ; preds = %inner_cond.preheader, %__orig_main.exit
  %inner_counter_11 = phi i64 [ 10, %inner_cond.preheader ], [ %inner_counter_2, %__orig_main.exit ]
  %0 = tail call i64 @llvm.smax.i64(i64 %loop_counter_12, i64 %inner_counter_11) #5
  br label %else.1.i

else.1.i:                                         ; preds = %else.1.i, %inner_body
  %greater_2.i = phi i64 [ %greater_3.i, %else.1.i ], [ %0, %inner_body ]
  %1 = srem i64 %greater_2.i, %loop_counter_12
  %2 = srem i64 %greater_2.i, %inner_counter_11
  %3 = or i64 %2, %1
  %4 = icmp eq i64 %3, 0
  %greater_3.i = add i64 %greater_2.i, 1
  br i1 %4, label %__orig_main.exit, label %else.1.i

__orig_main.exit:                                 ; preds = %else.1.i
  %5 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %greater_2.i) #5
  %6 = tail call i32 @putchar(i32 10) #5
  %inner_counter_2 = add nuw nsw i64 %inner_counter_11, 1
  %exitcond.not = icmp eq i64 %inner_counter_2, 250
  br i1 %exitcond.not, label %inner_done, label %inner_body

inner_done:                                       ; preds = %__orig_main.exit
  %loop_counter_2 = add nuw nsw i64 %loop_counter_12, 1
  %exitcond3.not = icmp eq i64 %loop_counter_2, 250
  br i1 %exitcond3.not, label %loop_done, label %inner_cond.preheader

loop_done:                                        ; preds = %inner_done
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %x, i64 %y) local_unnamed_addr #0 {
pre_entry:
  %0 = tail call i64 @llvm.smax.i64(i64 %x, i64 %y)
  br label %else.1

else.1:                                           ; preds = %else.1, %pre_entry
  %greater_2 = phi i64 [ %greater_3, %else.1 ], [ %0, %pre_entry ]
  %1 = srem i64 %greater_2, %x
  %2 = srem i64 %greater_2, %y
  %3 = or i64 %2, %1
  %4 = icmp eq i64 %3, 0
  %greater_3 = add i64 %greater_2, 1
  br i1 %4, label %then.2, label %else.1

then.2:                                           ; preds = %else.1
  %5 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %greater_2) #5
  %6 = tail call i32 @putchar(i32 10) #5
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind readnone willreturn
define dso_local i64 @__getMod(i64 %val, i64 %mod) local_unnamed_addr #2 {
pre_entry:
  %0 = srem i64 %val, %mod
  ret i64 %0
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %inner_cond.preheader.i, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #6
  ret i32 0

inner_cond.preheader.i:                           ; preds = %0, %inner_done.i
  %loop_counter_12.i = phi i64 [ %loop_counter_2.i, %inner_done.i ], [ 10, %0 ]
  br label %inner_body.i

inner_body.i:                                     ; preds = %__orig_main.exit.i, %inner_cond.preheader.i
  %inner_counter_11.i = phi i64 [ 10, %inner_cond.preheader.i ], [ %inner_counter_2.i, %__orig_main.exit.i ]
  %2 = tail call i64 @llvm.smax.i64(i64 %loop_counter_12.i, i64 %inner_counter_11.i) #5
  br label %else.1.i.i

else.1.i.i:                                       ; preds = %else.1.i.i, %inner_body.i
  %greater_2.i.i = phi i64 [ %greater_3.i.i, %else.1.i.i ], [ %2, %inner_body.i ]
  %3 = srem i64 %greater_2.i.i, %loop_counter_12.i
  %4 = srem i64 %greater_2.i.i, %inner_counter_11.i
  %5 = or i64 %4, %3
  %6 = icmp eq i64 %5, 0
  %greater_3.i.i = add i64 %greater_2.i.i, 1
  br i1 %6, label %__orig_main.exit.i, label %else.1.i.i

__orig_main.exit.i:                               ; preds = %else.1.i.i
  %7 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %greater_2.i.i) #5
  %8 = tail call i32 @putchar(i32 10) #5
  %inner_counter_2.i = add nuw nsw i64 %inner_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %inner_counter_2.i, 250
  br i1 %exitcond.not.i, label %inner_done.i, label %inner_body.i

inner_done.i:                                     ; preds = %__orig_main.exit.i
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_12.i, 1
  %exitcond3.not.i = icmp eq i64 %loop_counter_2.i, 250
  br i1 %exitcond3.not.i, label %__main.exit, label %inner_cond.preheader.i

__main.exit:                                      ; preds = %inner_done.i
  ret i32 0
}

; Function Attrs: nocallback nofree nosync nounwind readnone speculatable willreturn
declare i64 @llvm.smax.i64(i64, i64) #3

; Function Attrs: cold minsize noreturn
define internal void @main.cold.1(i32 %0) #4 {
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
attributes #3 = { nocallback nofree nosync nounwind readnone speculatable willreturn }
attributes #4 = { cold minsize noreturn }
attributes #5 = { nounwind }
attributes #6 = { noinline }
