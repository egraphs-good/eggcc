; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmp1I2HWQ/compile.ll'
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
  br label %for.body.3.lr.ph.i.i

for.body.3.lr.ph.i.i:                             ; preds = %__orig_main.exit, %b0
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %v10_0.i.i, %__orig_main.exit ]
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 1) #3
  %1 = tail call i32 @putchar(i32 10) #3
  %v10_0.i.i = add nuw nsw i64 %loop_counter_11, 1
  br label %for.body.3.i.i

for.body.3.i.i:                                   ; preds = %for.body.3.i.i, %for.body.3.lr.ph.i.i
  %prev_13.i.i = phi i64 [ 1, %for.body.3.lr.ph.i.i ], [ %v16_0.i.i, %for.body.3.i.i ]
  %i_12.i.i = phi i64 [ 0, %for.body.3.lr.ph.i.i ], [ %v22_0.i.i, %for.body.3.i.i ]
  %v13_0.i.i = sub nsw i64 %v10_0.i.i, %i_12.i.i
  %v16_0.i.i = mul i64 %v13_0.i.i, %prev_13.i.i
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %v16_0.i.i) #3
  %3 = tail call i32 @putchar(i32 10) #3
  %v22_0.i.i = add nuw nsw i64 %i_12.i.i, 1
  %exitcond.not.i.i = icmp eq i64 %v22_0.i.i, %loop_counter_11
  br i1 %exitcond.not.i.i, label %__orig_main.exit, label %for.body.3.i.i

__orig_main.exit:                                 ; preds = %for.body.3.i.i
  %exitcond.not = icmp eq i64 %v10_0.i.i, 1000
  br i1 %exitcond.not, label %loop_done, label %for.body.3.lr.ph.i.i

loop_done:                                        ; preds = %__orig_main.exit
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %v0) local_unnamed_addr #0 {
pre_entry:
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 1) #3
  %1 = tail call i32 @putchar(i32 10) #3
  %v7_01.i = icmp sgt i64 %v0, 0
  br i1 %v7_01.i, label %for.body.3.lr.ph.i, label %__generateNthRow.exit

for.body.3.lr.ph.i:                               ; preds = %pre_entry
  %v10_0.i = add nuw i64 %v0, 1
  br label %for.body.3.i

for.body.3.i:                                     ; preds = %for.body.3.i, %for.body.3.lr.ph.i
  %prev_13.i = phi i64 [ 1, %for.body.3.lr.ph.i ], [ %v16_0.i, %for.body.3.i ]
  %i_12.i = phi i64 [ 0, %for.body.3.lr.ph.i ], [ %v22_0.i, %for.body.3.i ]
  %v13_0.i = sub i64 %v10_0.i, %i_12.i
  %v16_0.i = mul i64 %v13_0.i, %prev_13.i
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %v16_0.i) #3
  %3 = tail call i32 @putchar(i32 10) #3
  %v22_0.i = add nuw nsw i64 %i_12.i, 1
  %exitcond.not.i = icmp eq i64 %v22_0.i, %v0
  br i1 %exitcond.not.i, label %__generateNthRow.exit, label %for.body.3.i

__generateNthRow.exit:                            ; preds = %for.body.3.i, %pre_entry
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__generateNthRow(i64 %x) local_unnamed_addr #0 {
pre_entry:
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 1) #3
  %1 = tail call i32 @putchar(i32 10) #3
  %v7_01 = icmp sgt i64 %x, 0
  br i1 %v7_01, label %for.body.3.lr.ph, label %for.end.3

for.body.3.lr.ph:                                 ; preds = %pre_entry
  %v10_0 = add nuw i64 %x, 1
  br label %for.body.3

for.body.3:                                       ; preds = %for.body.3.lr.ph, %for.body.3
  %prev_13 = phi i64 [ 1, %for.body.3.lr.ph ], [ %v16_0, %for.body.3 ]
  %i_12 = phi i64 [ 0, %for.body.3.lr.ph ], [ %v22_0, %for.body.3 ]
  %v13_0 = sub i64 %v10_0, %i_12
  %v16_0 = mul i64 %prev_13, %v13_0
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %v16_0) #3
  %3 = tail call i32 @putchar(i32 10) #3
  %v22_0 = add nuw nsw i64 %i_12, 1
  %exitcond.not = icmp eq i64 %v22_0, %x
  br i1 %exitcond.not, label %for.end.3, label %for.body.3

for.end.3:                                        ; preds = %for.body.3, %pre_entry
  ret void
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %for.body.3.lr.ph.i.i.i, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #4
  ret i32 0

for.body.3.lr.ph.i.i.i:                           ; preds = %0, %__orig_main.exit.i
  %loop_counter_11.i = phi i64 [ %v10_0.i.i.i, %__orig_main.exit.i ], [ 10, %0 ]
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 1) #3
  %3 = tail call i32 @putchar(i32 10) #3
  %v10_0.i.i.i = add nuw nsw i64 %loop_counter_11.i, 1
  br label %for.body.3.i.i.i

for.body.3.i.i.i:                                 ; preds = %for.body.3.i.i.i, %for.body.3.lr.ph.i.i.i
  %prev_13.i.i.i = phi i64 [ 1, %for.body.3.lr.ph.i.i.i ], [ %v16_0.i.i.i, %for.body.3.i.i.i ]
  %i_12.i.i.i = phi i64 [ 0, %for.body.3.lr.ph.i.i.i ], [ %v22_0.i.i.i, %for.body.3.i.i.i ]
  %v13_0.i.i.i = sub nsw i64 %v10_0.i.i.i, %i_12.i.i.i
  %v16_0.i.i.i = mul i64 %v13_0.i.i.i, %prev_13.i.i.i
  %4 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %v16_0.i.i.i) #3
  %5 = tail call i32 @putchar(i32 10) #3
  %v22_0.i.i.i = add nuw nsw i64 %i_12.i.i.i, 1
  %exitcond.not.i.i.i = icmp eq i64 %v22_0.i.i.i, %loop_counter_11.i
  br i1 %exitcond.not.i.i.i, label %__orig_main.exit.i, label %for.body.3.i.i.i

__orig_main.exit.i:                               ; preds = %for.body.3.i.i.i
  %exitcond.not.i = icmp eq i64 %v10_0.i.i.i, 1000
  br i1 %exitcond.not.i, label %__main.exit, label %for.body.3.lr.ph.i.i.i

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
