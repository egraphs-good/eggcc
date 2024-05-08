; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpPHHvpY/compile.ll'
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
  br label %body.i.preheader

body.i.preheader:                                 ; preds = %__orig_main.exit, %b0
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %__orig_main.exit ]
  br label %body.i

body.i:                                           ; preds = %body.i.preheader, %body.i
  %input_13.i = phi i64 [ %quotient_0.i.i, %body.i ], [ %loop_counter_11, %body.i.preheader ]
  %sum_12.i = phi i64 [ %sum_2.i, %body.i ], [ 0, %body.i.preheader ]
  %quotient_0.i.i = sdiv i64 %input_13.i, 2
  %prod_0.neg.i.i = mul i64 %quotient_0.i.i, -2
  %diff_0.i.i = add i64 %sum_12.i, %input_13.i
  %sum_2.i = add i64 %diff_0.i.i, %prod_0.neg.i.i
  %input_13.off.i = add nsw i64 %input_13.i, 1
  %0 = icmp ult i64 %input_13.off.i, 3
  br i1 %0, label %__orig_main.exit, label %body.i

__orig_main.exit:                                 ; preds = %body.i
  %1 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %sum_2.i) #4
  %2 = tail call i32 @putchar(i32 10) #4
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 1000000
  br i1 %exitcond.not, label %loop_done, label %body.i.preheader

loop_done:                                        ; preds = %__orig_main.exit
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %input) local_unnamed_addr #0 {
pre_entry:
  %cond_01 = icmp eq i64 %input, 0
  br i1 %cond_01, label %done, label %body

body:                                             ; preds = %pre_entry, %body
  %input_13 = phi i64 [ %quotient_0.i, %body ], [ %input, %pre_entry ]
  %sum_12 = phi i64 [ %sum_2, %body ], [ 0, %pre_entry ]
  %quotient_0.i = sdiv i64 %input_13, 2
  %prod_0.neg.i = mul i64 %quotient_0.i, -2
  %diff_0.i = add i64 %input_13, %sum_12
  %sum_2 = add i64 %diff_0.i, %prod_0.neg.i
  %input_13.off = add i64 %input_13, 1
  %0 = icmp ult i64 %input_13.off, 3
  br i1 %0, label %done, label %body

done:                                             ; preds = %body, %pre_entry
  %sum_1.lcssa = phi i64 [ 0, %pre_entry ], [ %sum_2, %body ]
  %1 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %sum_1.lcssa) #4
  %2 = tail call i32 @putchar(i32 10) #4
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind readnone willreturn
define dso_local i64 @__mod(i64 %dividend, i64 %divisor) local_unnamed_addr #2 {
pre_entry:
  %quotient_0 = sdiv i64 %dividend, %divisor
  %prod_0.neg = mul i64 %quotient_0, -2
  %diff_0 = add i64 %prod_0.neg, %dividend
  ret i64 %diff_0
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %body.i.preheader.i, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #5
  ret i32 0

body.i.preheader.i:                               ; preds = %0, %__orig_main.exit.i
  %loop_counter_11.i = phi i64 [ %loop_counter_2.i, %__orig_main.exit.i ], [ 10, %0 ]
  br label %body.i.i

body.i.i:                                         ; preds = %body.i.i, %body.i.preheader.i
  %input_13.i.i = phi i64 [ %quotient_0.i.i.i, %body.i.i ], [ %loop_counter_11.i, %body.i.preheader.i ]
  %sum_12.i.i = phi i64 [ %sum_2.i.i, %body.i.i ], [ 0, %body.i.preheader.i ]
  %quotient_0.i.i.i = sdiv i64 %input_13.i.i, 2
  %prod_0.neg.i.i.i = mul i64 %quotient_0.i.i.i, -2
  %diff_0.i.i.i = add i64 %sum_12.i.i, %input_13.i.i
  %sum_2.i.i = add i64 %diff_0.i.i.i, %prod_0.neg.i.i.i
  %input_13.off.i.i = add nsw i64 %input_13.i.i, 1
  %2 = icmp ult i64 %input_13.off.i.i, 3
  br i1 %2, label %__orig_main.exit.i, label %body.i.i

__orig_main.exit.i:                               ; preds = %body.i.i
  %3 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %sum_2.i.i) #4
  %4 = tail call i32 @putchar(i32 10) #4
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 1000000
  br i1 %exitcond.not.i, label %__main.exit, label %body.i.preheader.i

__main.exit:                                      ; preds = %__orig_main.exit.i
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
