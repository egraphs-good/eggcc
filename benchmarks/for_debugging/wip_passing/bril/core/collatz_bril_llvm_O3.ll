; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmplg4h9N/compile.ll'
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
  br label %loop.i.preheader

loop.i.preheader:                                 ; preds = %__orig_main.exit, %b0
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %__orig_main.exit ]
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %loop_counter_11) #3
  %1 = tail call i32 @putchar(i32 10) #3
  br label %loop.i

loop.i:                                           ; preds = %loop.i.preheader, %loop.i
  %x_12.i = phi i64 [ %x_1.be.i, %loop.i ], [ %loop_counter_11, %loop.i.preheader ]
  %half_0.i = sdiv i64 %x_12.i, 2
  %doublehalf_0.i = shl nsw i64 %half_0.i, 1
  %even_0.i = icmp eq i64 %x_12.i, %doublehalf_0.i
  %x_3.i = mul i64 %x_12.i, 3
  %x_4.i = add i64 %x_3.i, 1
  %x_1.be.i = select i1 %even_0.i, i64 %half_0.i, i64 %x_4.i
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %x_1.be.i) #3
  %3 = tail call i32 @putchar(i32 10) #3
  %eq_one_0.i = icmp eq i64 %x_1.be.i, 1
  br i1 %eq_one_0.i, label %__orig_main.exit, label %loop.i

__orig_main.exit:                                 ; preds = %loop.i
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 10000
  br i1 %exitcond.not, label %loop_done, label %loop.i.preheader

loop_done:                                        ; preds = %__orig_main.exit
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %x) local_unnamed_addr #0 {
pre_entry:
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %x) #3
  %1 = tail call i32 @putchar(i32 10) #3
  %eq_one_01 = icmp eq i64 %x, 1
  br i1 %eq_one_01, label %end, label %loop

loop:                                             ; preds = %pre_entry, %loop
  %x_12 = phi i64 [ %x_1.be, %loop ], [ %x, %pre_entry ]
  %half_0 = sdiv i64 %x_12, 2
  %doublehalf_0 = shl nsw i64 %half_0, 1
  %even_0 = icmp eq i64 %x_12, %doublehalf_0
  %x_3 = mul i64 %x_12, 3
  %x_4 = add i64 %x_3, 1
  %x_1.be = select i1 %even_0, i64 %half_0, i64 %x_4
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %x_1.be) #3
  %3 = tail call i32 @putchar(i32 10) #3
  %eq_one_0 = icmp eq i64 %x_1.be, 1
  br i1 %eq_one_0, label %end, label %loop

end:                                              ; preds = %loop, %pre_entry
  ret void
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %loop.i.preheader.i, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #4
  ret i32 0

loop.i.preheader.i:                               ; preds = %0, %__orig_main.exit.i
  %loop_counter_11.i = phi i64 [ %loop_counter_2.i, %__orig_main.exit.i ], [ 10, %0 ]
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %loop_counter_11.i) #3
  %3 = tail call i32 @putchar(i32 10) #3
  br label %loop.i.i

loop.i.i:                                         ; preds = %loop.i.i, %loop.i.preheader.i
  %x_12.i.i = phi i64 [ %x_1.be.i.i, %loop.i.i ], [ %loop_counter_11.i, %loop.i.preheader.i ]
  %half_0.i.i = sdiv i64 %x_12.i.i, 2
  %doublehalf_0.i.i = shl nsw i64 %half_0.i.i, 1
  %even_0.i.i = icmp eq i64 %x_12.i.i, %doublehalf_0.i.i
  %x_3.i.i = mul i64 %x_12.i.i, 3
  %x_4.i.i = add i64 %x_3.i.i, 1
  %x_1.be.i.i = select i1 %even_0.i.i, i64 %half_0.i.i, i64 %x_4.i.i
  %4 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %x_1.be.i.i) #3
  %5 = tail call i32 @putchar(i32 10) #3
  %eq_one_0.i.i = icmp eq i64 %x_1.be.i.i, 1
  br i1 %eq_one_0.i.i, label %__orig_main.exit.i, label %loop.i.i

__orig_main.exit.i:                               ; preds = %loop.i.i
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 10000
  br i1 %exitcond.not.i, label %__main.exit, label %loop.i.preheader.i

__main.exit:                                      ; preds = %__orig_main.exit.i
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
