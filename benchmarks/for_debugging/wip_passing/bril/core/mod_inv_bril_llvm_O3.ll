; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpm6WtH3/compile.ll'
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
  br label %loop_body

loop_body:                                        ; preds = %b0, %loop_body
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %loop_body ]
  %0 = urem i64 %loop_counter_11, 10007
  %v26_0.i = mul i64 %loop_counter_11, %loop_counter_11
  %1 = srem i64 %v26_0.i, 10007
  %v26_0.1.i = mul nsw i64 %1, %1
  %2 = urem i64 %v26_0.1.i, 10007
  %v21_0.2.i = mul nuw nsw i64 %2, %0
  %.lhs.trunc.i = trunc i64 %v21_0.2.i to i32
  %3 = urem i32 %.lhs.trunc.i, 10007
  %v26_0.2.i = mul nuw nsw i64 %2, %2
  %.lhs.trunc6.i = trunc i64 %v26_0.2.i to i32
  %4 = urem i32 %.lhs.trunc6.i, 10007
  %v26_0.3.i = mul nuw nsw i32 %4, %4
  %5 = urem i32 %v26_0.3.i, 10007
  %v21_0.4.i = mul nuw nsw i32 %5, %3
  %6 = urem i32 %v21_0.4.i, 10007
  %v26_0.4.i = mul nuw nsw i32 %5, %5
  %7 = urem i32 %v26_0.4.i, 10007
  %v26_0.5.i = mul nuw nsw i32 %7, %7
  %8 = urem i32 %v26_0.5.i, 10007
  %v26_0.6.i = mul nuw nsw i32 %8, %8
  %9 = urem i32 %v26_0.6.i, 10007
  %v26_0.7.i = mul nuw nsw i32 %9, %9
  %10 = urem i32 %v26_0.7.i, 10007
  %v21_0.8.i = mul nuw nsw i32 %10, %6
  %11 = urem i32 %v21_0.8.i, 10007
  %v26_0.8.i = mul nuw nsw i32 %10, %10
  %12 = urem i32 %v26_0.8.i, 10007
  %v21_0.9.i = mul nuw nsw i32 %11, %12
  %13 = urem i32 %v21_0.9.i, 10007
  %v26_0.9.i = mul nuw nsw i32 %12, %12
  %14 = urem i32 %v26_0.9.i, 10007
  %v21_0.10.i = mul nuw nsw i32 %13, %14
  %15 = urem i32 %v21_0.10.i, 10007
  %v26_0.10.i = mul nuw nsw i32 %14, %14
  %16 = urem i32 %v26_0.10.i, 10007
  %v26_0.11.i = mul nuw nsw i32 %16, %16
  %17 = urem i32 %v26_0.11.i, 10007
  %v26_0.12.i = mul nuw nsw i32 %17, %17
  %18 = urem i32 %v26_0.12.i, 10007
  %v21_0.13.i = mul nuw nsw i32 %18, %15
  %19 = urem i32 %v21_0.13.i, 10007
  %.sext36.i = zext i32 %19 to i64
  %20 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %.sext36.i) #4
  %21 = tail call i32 @putchar(i32 10) #4
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 1000000
  br i1 %exitcond.not, label %loop_done, label %loop_body

loop_done:                                        ; preds = %loop_body
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %n) local_unnamed_addr #0 {
endif.11.13:
  %0 = srem i64 %n, 10007
  %v26_0 = mul i64 %n, %n
  %1 = srem i64 %v26_0, 10007
  %v26_0.1 = mul nsw i64 %1, %1
  %2 = urem i64 %v26_0.1, 10007
  %v21_0.2 = mul nsw i64 %0, %2
  %.lhs.trunc = trunc i64 %v21_0.2 to i32
  %3 = srem i32 %.lhs.trunc, 10007
  %v26_0.2 = mul nuw nsw i64 %2, %2
  %.lhs.trunc6 = trunc i64 %v26_0.2 to i32
  %4 = urem i32 %.lhs.trunc6, 10007
  %v26_0.3 = mul nuw nsw i32 %4, %4
  %5 = urem i32 %v26_0.3, 10007
  %v21_0.4 = mul nsw i32 %3, %5
  %6 = srem i32 %v21_0.4, 10007
  %v26_0.4 = mul nuw nsw i32 %5, %5
  %7 = urem i32 %v26_0.4, 10007
  %v26_0.5 = mul nuw nsw i32 %7, %7
  %8 = urem i32 %v26_0.5, 10007
  %v26_0.6 = mul nuw nsw i32 %8, %8
  %9 = urem i32 %v26_0.6, 10007
  %v26_0.7 = mul nuw nsw i32 %9, %9
  %10 = urem i32 %v26_0.7, 10007
  %v21_0.8 = mul nsw i32 %6, %10
  %11 = srem i32 %v21_0.8, 10007
  %v26_0.8 = mul nuw nsw i32 %10, %10
  %12 = urem i32 %v26_0.8, 10007
  %v21_0.9 = mul nsw i32 %11, %12
  %13 = srem i32 %v21_0.9, 10007
  %v26_0.9 = mul nuw nsw i32 %12, %12
  %14 = urem i32 %v26_0.9, 10007
  %v21_0.10 = mul nsw i32 %13, %14
  %15 = srem i32 %v21_0.10, 10007
  %v26_0.10 = mul nuw nsw i32 %14, %14
  %16 = urem i32 %v26_0.10, 10007
  %v26_0.11 = mul nuw nsw i32 %16, %16
  %17 = urem i32 %v26_0.11, 10007
  %v26_0.12 = mul nuw nsw i32 %17, %17
  %18 = urem i32 %v26_0.12, 10007
  %v21_0.13 = mul nsw i32 %15, %18
  %19 = srem i32 %v21_0.13, 10007
  %.sext36 = sext i32 %19 to i64
  %20 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %.sext36) #4
  %21 = tail call i32 @putchar(i32 10) #4
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind readnone willreturn
define dso_local i64 @__mod(i64 %n, i64 %p) local_unnamed_addr #2 {
pre_entry:
  %0 = srem i64 %n, %p
  ret i64 %0
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %2, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #5
  ret i32 0

2:                                                ; preds = %0
  tail call void @__main()
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
