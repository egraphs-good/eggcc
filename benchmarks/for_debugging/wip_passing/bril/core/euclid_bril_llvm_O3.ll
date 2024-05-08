; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpvZTIlR/compile.ll'
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
  %loop_counter_13 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %inner_done ]
  br label %for.body.5.i.i.preheader

for.body.5.i.i.preheader:                         ; preds = %__orig_main.exit, %inner_cond.preheader
  %inner_counter_11 = phi i64 [ 10, %inner_cond.preheader ], [ %inner_counter_2, %__orig_main.exit ]
  br label %for.body.5.i.i

for.body.5.i.i:                                   ; preds = %for.body.5.i.i.preheader, %for.body.5.i.i
  %a_13.i.i = phi i64 [ %b_12.i.i, %for.body.5.i.i ], [ %loop_counter_13, %for.body.5.i.i.preheader ]
  %b_12.i.i = phi i64 [ %0, %for.body.5.i.i ], [ %inner_counter_11, %for.body.5.i.i.preheader ]
  %0 = srem i64 %a_13.i.i, %b_12.i.i
  %cond_1.not.i.i = icmp eq i64 %0, 0
  br i1 %cond_1.not.i.i, label %__orig_main.exit, label %for.body.5.i.i

__orig_main.exit:                                 ; preds = %for.body.5.i.i
  %1 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %b_12.i.i) #5
  %2 = tail call i32 @putchar(i32 10) #5
  %inner_counter_2 = add nuw nsw i64 %inner_counter_11, 1
  %exitcond.not = icmp eq i64 %inner_counter_2, 1000
  br i1 %exitcond.not, label %inner_done, label %for.body.5.i.i.preheader

inner_done:                                       ; preds = %__orig_main.exit
  %loop_counter_2 = add nuw nsw i64 %loop_counter_13, 1
  %exitcond4.not = icmp eq i64 %loop_counter_2, 1000
  br i1 %exitcond4.not, label %loop_done, label %inner_cond.preheader

loop_done:                                        ; preds = %inner_done
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %v0, i64 %v1) local_unnamed_addr #0 {
pre_entry:
  %cond_1.not1.i = icmp eq i64 %v1, 0
  br i1 %cond_1.not1.i, label %__gcd.exit, label %for.body.5.i

for.body.5.i:                                     ; preds = %pre_entry, %for.body.5.i
  %a_13.i = phi i64 [ %b_12.i, %for.body.5.i ], [ %v0, %pre_entry ]
  %b_12.i = phi i64 [ %0, %for.body.5.i ], [ %v1, %pre_entry ]
  %0 = srem i64 %a_13.i, %b_12.i
  %cond_1.not.i = icmp eq i64 %0, 0
  br i1 %cond_1.not.i, label %__gcd.exit, label %for.body.5.i

__gcd.exit:                                       ; preds = %for.body.5.i, %pre_entry
  %a_1.lcssa.i = phi i64 [ %v0, %pre_entry ], [ %b_12.i, %for.body.5.i ]
  %1 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %a_1.lcssa.i) #5
  %2 = tail call i32 @putchar(i32 10) #5
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind readnone willreturn
define dso_local i64 @__mod(i64 %r, i64 %s) local_unnamed_addr #2 {
pre_entry:
  %0 = srem i64 %r, %s
  ret i64 %0
}

; Function Attrs: nofree norecurse nosync nounwind readnone
define dso_local i64 @__gcd(i64 %a, i64 %b) local_unnamed_addr #3 {
pre_entry:
  %cond_1.not1 = icmp eq i64 %b, 0
  br i1 %cond_1.not1, label %for.end.5, label %for.body.5

for.body.5:                                       ; preds = %pre_entry, %for.body.5
  %a_13 = phi i64 [ %b_12, %for.body.5 ], [ %a, %pre_entry ]
  %b_12 = phi i64 [ %0, %for.body.5 ], [ %b, %pre_entry ]
  %0 = srem i64 %a_13, %b_12
  %cond_1.not = icmp eq i64 %0, 0
  br i1 %cond_1.not, label %for.end.5, label %for.body.5

for.end.5:                                        ; preds = %for.body.5, %pre_entry
  %a_1.lcssa = phi i64 [ %a, %pre_entry ], [ %b_12, %for.body.5 ]
  ret i64 %a_1.lcssa
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %inner_cond.preheader.i, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #6
  ret i32 0

inner_cond.preheader.i:                           ; preds = %0, %inner_done.i
  %loop_counter_13.i = phi i64 [ %loop_counter_2.i, %inner_done.i ], [ 10, %0 ]
  br label %for.body.5.i.i.preheader.i

for.body.5.i.i.preheader.i:                       ; preds = %__orig_main.exit.i, %inner_cond.preheader.i
  %inner_counter_11.i = phi i64 [ 10, %inner_cond.preheader.i ], [ %inner_counter_2.i, %__orig_main.exit.i ]
  br label %for.body.5.i.i.i

for.body.5.i.i.i:                                 ; preds = %for.body.5.i.i.i, %for.body.5.i.i.preheader.i
  %a_13.i.i.i = phi i64 [ %b_12.i.i.i, %for.body.5.i.i.i ], [ %loop_counter_13.i, %for.body.5.i.i.preheader.i ]
  %b_12.i.i.i = phi i64 [ %2, %for.body.5.i.i.i ], [ %inner_counter_11.i, %for.body.5.i.i.preheader.i ]
  %2 = srem i64 %a_13.i.i.i, %b_12.i.i.i
  %cond_1.not.i.i.i = icmp eq i64 %2, 0
  br i1 %cond_1.not.i.i.i, label %__orig_main.exit.i, label %for.body.5.i.i.i

__orig_main.exit.i:                               ; preds = %for.body.5.i.i.i
  %3 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %b_12.i.i.i) #5
  %4 = tail call i32 @putchar(i32 10) #5
  %inner_counter_2.i = add nuw nsw i64 %inner_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %inner_counter_2.i, 1000
  br i1 %exitcond.not.i, label %inner_done.i, label %for.body.5.i.i.preheader.i

inner_done.i:                                     ; preds = %__orig_main.exit.i
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_13.i, 1
  %exitcond4.not.i = icmp eq i64 %loop_counter_2.i, 1000
  br i1 %exitcond4.not.i, label %__main.exit, label %inner_cond.preheader.i

__main.exit:                                      ; preds = %inner_done.i
  ret i32 0
}

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
attributes #3 = { nofree norecurse nosync nounwind readnone }
attributes #4 = { cold minsize noreturn }
attributes #5 = { nounwind }
attributes #6 = { noinline }
