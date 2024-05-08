; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpDospFO/compile.ll'
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

; Function Attrs: argmemonly mustprogress nofree norecurse nosync nounwind willreturn
define dso_local i64 @__rand(i64* nocapture %seq, i64 %max) local_unnamed_addr #3 {
pre_entry:
  %x_0 = load i64, i64* %seq, align 8
  %ax_0 = mul i64 %x_0, 25214903917
  %axpc_0 = add i64 %ax_0, 11
  %next_2 = srem i64 %axpc_0, 281474976710656
  store i64 %next_2, i64* %seq, align 8
  %0 = srem i64 %next_2, %max
  ret i64 %0
}

; Function Attrs: nofree nounwind
define dso_local noalias i64* @__randarray(i64 %size, i64* nocapture %rng) local_unnamed_addr #0 {
pre_entry:
  %z0 = shl i64 %size, 3
  %z1 = tail call i8* @malloc(i64 %z0)
  %arr_0 = bitcast i8* %z1 to i64*
  %cond_01 = icmp sgt i64 %size, 0
  br i1 %cond_01, label %body.lr.ph, label %done

body.lr.ph:                                       ; preds = %pre_entry
  %rng.promoted = load i64, i64* %rng, align 8
  br label %body

body:                                             ; preds = %body.lr.ph, %body
  %next_2.i3 = phi i64 [ %rng.promoted, %body.lr.ph ], [ %next_2.i, %body ]
  %i_12 = phi i64 [ 0, %body.lr.ph ], [ %i_2, %body ]
  %ax_0.i = mul i64 %next_2.i3, 25214903917
  %axpc_0.i = add i64 %ax_0.i, 11
  %next_2.i = srem i64 %axpc_0.i, 281474976710656
  %0 = srem i64 %next_2.i, 1000
  %loc_0 = getelementptr inbounds i64, i64* %arr_0, i64 %i_12
  store i64 %0, i64* %loc_0, align 8
  %i_2 = add nuw nsw i64 %i_12, 1
  %exitcond.not = icmp eq i64 %i_2, %size
  br i1 %exitcond.not, label %loop.done_crit_edge, label %body

loop.done_crit_edge:                              ; preds = %body
  store i64 %next_2.i, i64* %rng, align 8
  br label %done

done:                                             ; preds = %loop.done_crit_edge, %pre_entry
  ret i64* %arr_0
}

; Function Attrs: nofree norecurse nosync nounwind readnone
define dso_local void @__main() local_unnamed_addr #4 {
b0:
  ret void
}

; Function Attrs: nounwind
define dso_local void @__orig_main(i64 %size) local_unnamed_addr #5 {
pre_entry:
  ret void
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %2, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #7
  ret i32 0

2:                                                ; preds = %0
  ret i32 0
}

; Function Attrs: cold minsize noreturn
define internal void @main.cold.1(i32 %0) #6 {
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
attributes #3 = { argmemonly mustprogress nofree norecurse nosync nounwind willreturn }
attributes #4 = { nofree norecurse nosync nounwind readnone }
attributes #5 = { nounwind }
attributes #6 = { cold minsize noreturn }
attributes #7 = { noinline }
