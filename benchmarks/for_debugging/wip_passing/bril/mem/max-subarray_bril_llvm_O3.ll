; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmplx4ro8/compile.ll'
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

; Function Attrs: argmemonly mustprogress nofree norecurse nosync nounwind readonly willreturn
define dso_local i32 @btoi(i8* nocapture readonly %0) local_unnamed_addr #2 {
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
define dso_local noalias i64* @__pack(i64 %size, i64 %n1, i64 %n2, i64 %n3, i64 %n4, i64 %n5, i64 %n6, i64 %n7, i64 %n8, i64 %n9, i64 %n10) local_unnamed_addr #3 {
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
  %loc_5 = getelementptr inbounds i64, i64* %array_0, i64 5
  store i64 %n6, i64* %loc_5, align 8
  %loc_6 = getelementptr inbounds i64, i64* %array_0, i64 6
  store i64 %n7, i64* %loc_6, align 8
  %loc_7 = getelementptr inbounds i64, i64* %array_0, i64 7
  store i64 %n8, i64* %loc_7, align 8
  %loc_8 = getelementptr inbounds i64, i64* %array_0, i64 8
  store i64 %n9, i64* %loc_8, align 8
  %loc_9 = getelementptr inbounds i64, i64* %array_0, i64 9
  store i64 %n10, i64* %loc_9, align 8
  ret i64* %array_0
}

; Function Attrs: mustprogress nofree nosync nounwind readnone willreturn
define dso_local i64 @__max(i64 %n, i64 %m) local_unnamed_addr #4 {
pre_entry:
  %0 = tail call i64 @llvm.smax.i64(i64 %n, i64 %m)
  ret i64 %0
}

; Function Attrs: nofree nounwind
define dso_local void @__main() local_unnamed_addr #0 {
b0:
  br label %loop_body

loop_body:                                        ; preds = %b0, %loop_body
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %loop_body ]
  %val_0.i = add nuw nsw i64 %loop_counter_11, 100
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0.i) #5
  %1 = tail call i32 @putchar(i32 10) #5
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 1000000
  br i1 %exitcond.not, label %loop_done, label %loop_body

loop_done:                                        ; preds = %loop_body
  ret void
}

; Function Attrs: nounwind
define dso_local void @__orig_main(i64 %x) local_unnamed_addr #5 {
pre_entry:
  %val_0 = add i64 %x, 100
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0) #5
  %1 = tail call i32 @putchar(i32 10) #5
  ret void
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %loop_body.i, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #8
  ret i32 0

loop_body.i:                                      ; preds = %0, %loop_body.i
  %loop_counter_11.i = phi i64 [ %loop_counter_2.i, %loop_body.i ], [ 10, %0 ]
  %val_0.i.i = add nuw nsw i64 %loop_counter_11.i, 100
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0.i.i) #5
  %3 = tail call i32 @putchar(i32 10) #5
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 1000000
  br i1 %exitcond.not.i, label %__main.exit, label %loop_body.i

__main.exit:                                      ; preds = %loop_body.i
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
attributes #2 = { argmemonly mustprogress nofree norecurse nosync nounwind readonly willreturn }
attributes #3 = { mustprogress nofree nounwind willreturn }
attributes #4 = { mustprogress nofree nosync nounwind readnone willreturn }
attributes #5 = { nounwind }
attributes #6 = { nocallback nofree nosync nounwind readnone speculatable willreturn }
attributes #7 = { cold minsize noreturn }
attributes #8 = { noinline }
