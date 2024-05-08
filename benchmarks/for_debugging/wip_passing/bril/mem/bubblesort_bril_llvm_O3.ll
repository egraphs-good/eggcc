; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpOiCgb0/compile.ll'
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
define dso_local noalias i64* @__pack(i64 %size, i64 %n1, i64 %n2, i64 %n3, i64 %n4, i64 %n5) local_unnamed_addr #3 {
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
  ret i64* %array_0
}

; Function Attrs: nofree nounwind
define dso_local void @__print_array(i64* nocapture readonly %array, i64 %size) local_unnamed_addr #0 {
pre_entry:
  %cond_01 = icmp sgt i64 %size, 0
  br i1 %cond_01, label %body, label %done

body:                                             ; preds = %pre_entry, %body
  %i_12 = phi i64 [ %i_2, %body ], [ 0, %pre_entry ]
  %loc_0 = getelementptr inbounds i64, i64* %array, i64 %i_12
  %val_0 = load i64, i64* %loc_0, align 8
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0) #5
  %1 = tail call i32 @putchar(i32 10) #5
  %i_2 = add nuw nsw i64 %i_12, 1
  %exitcond.not = icmp eq i64 %i_2, %size
  br i1 %exitcond.not, label %done, label %body

done:                                             ; preds = %body, %pre_entry
  ret void
}

; Function Attrs: argmemonly mustprogress nofree norecurse nosync nounwind willreturn
define dso_local void @__swap_cond(i64* nocapture %array, i64 %j) local_unnamed_addr #4 {
pre_entry:
  %j_add_1_0 = add i64 %j, 1
  %loc_0 = getelementptr inbounds i64, i64* %array, i64 %j
  %loc_next_0 = getelementptr inbounds i64, i64* %array, i64 %j_add_1_0
  %elem_a_0 = load i64, i64* %loc_0, align 8
  %elem_b_0 = load i64, i64* %loc_next_0, align 8
  %cond_0 = icmp sgt i64 %elem_a_0, %elem_b_0
  br i1 %cond_0, label %swap, label %done

swap:                                             ; preds = %pre_entry
  store i64 %elem_b_0, i64* %loc_0, align 8
  store i64 %elem_a_0, i64* %loc_next_0, align 8
  br label %done

done:                                             ; preds = %swap, %pre_entry
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__main() local_unnamed_addr #0 {
b0:
  br label %loop2_cond.preheader

loop2_cond.preheader:                             ; preds = %b0, %loop2_done
  %loop_counter_15 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %loop2_done ]
  br label %loop3_cond.preheader

loop3_cond.preheader:                             ; preds = %loop2_cond.preheader, %loop3_done
  %loop2_counter_14 = phi i64 [ 10, %loop2_cond.preheader ], [ %loop2_counter_2, %loop3_done ]
  %0 = tail call i64 @llvm.smin.i64(i64 %loop_counter_15, i64 %loop2_counter_14) #5
  %1 = tail call i64 @llvm.smax.i64(i64 %loop_counter_15, i64 %loop2_counter_14) #5
  br label %loop4_cond.preheader

loop4_cond.preheader:                             ; preds = %loop3_cond.preheader, %loop4_done
  %loop3_counter_13 = phi i64 [ 10, %loop3_cond.preheader ], [ %loop3_counter_2, %loop4_done ]
  %2 = tail call i64 @llvm.smin.i64(i64 %1, i64 %loop3_counter_13) #5
  %3 = tail call i64 @llvm.smax.i64(i64 %1, i64 %loop3_counter_13) #5
  %4 = tail call i64 @llvm.smin.i64(i64 %0, i64 %2) #5
  %5 = tail call i64 @llvm.smax.i64(i64 %0, i64 %2) #5
  br label %loop5_cond.preheader

loop5_cond.preheader:                             ; preds = %loop4_cond.preheader, %loop5_done
  %loop4_counter_12 = phi i64 [ 10, %loop4_cond.preheader ], [ %loop4_counter_2, %loop5_done ]
  %6 = tail call i64 @llvm.smin.i64(i64 %3, i64 %loop4_counter_12) #5
  %7 = tail call i64 @llvm.smax.i64(i64 %3, i64 %loop4_counter_12) #5
  %8 = tail call i64 @llvm.smin.i64(i64 %5, i64 %6) #5
  %9 = tail call i64 @llvm.smax.i64(i64 %5, i64 %6) #5
  %10 = tail call i64 @llvm.smin.i64(i64 %4, i64 %8) #5
  %11 = tail call i64 @llvm.smax.i64(i64 %4, i64 %8) #5
  br label %loop5_body

loop5_body:                                       ; preds = %loop5_cond.preheader, %loop5_body
  %loop5_counter_11 = phi i64 [ 10, %loop5_cond.preheader ], [ %loop5_counter_2, %loop5_body ]
  %12 = tail call i64 @llvm.smax.i64(i64 %7, i64 %loop5_counter_11) #5
  %13 = tail call i64 @llvm.smin.i64(i64 %7, i64 %loop5_counter_11) #5
  %14 = tail call i64 @llvm.smax.i64(i64 %9, i64 %13) #5
  %15 = tail call i64 @llvm.smin.i64(i64 %9, i64 %13) #5
  %16 = tail call i64 @llvm.smax.i64(i64 %11, i64 %15) #5
  %17 = tail call i64 @llvm.smin.i64(i64 %11, i64 %15) #5
  %18 = tail call i64 @llvm.smax.i64(i64 %10, i64 %17) #5
  %19 = tail call i64 @llvm.smin.i64(i64 %10, i64 %17) #5
  %20 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %19) #5
  %21 = tail call i32 @putchar(i32 10) #5
  %22 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %18) #5
  %23 = tail call i32 @putchar(i32 10) #5
  %24 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %16) #5
  %25 = tail call i32 @putchar(i32 10) #5
  %26 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %14) #5
  %27 = tail call i32 @putchar(i32 10) #5
  %28 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %12) #5
  %29 = tail call i32 @putchar(i32 10) #5
  %loop5_counter_2 = add nuw nsw i64 %loop5_counter_11, 1
  %exitcond.not = icmp eq i64 %loop5_counter_2, 25
  br i1 %exitcond.not, label %loop5_done, label %loop5_body

loop5_done:                                       ; preds = %loop5_body
  %loop4_counter_2 = add nuw nsw i64 %loop4_counter_12, 1
  %exitcond6.not = icmp eq i64 %loop4_counter_2, 25
  br i1 %exitcond6.not, label %loop4_done, label %loop5_cond.preheader

loop4_done:                                       ; preds = %loop5_done
  %loop3_counter_2 = add nuw nsw i64 %loop3_counter_13, 1
  %exitcond7.not = icmp eq i64 %loop3_counter_2, 25
  br i1 %exitcond7.not, label %loop3_done, label %loop4_cond.preheader

loop3_done:                                       ; preds = %loop4_done
  %loop2_counter_2 = add nuw nsw i64 %loop2_counter_14, 1
  %exitcond8.not = icmp eq i64 %loop2_counter_2, 25
  br i1 %exitcond8.not, label %loop2_done, label %loop3_cond.preheader

loop2_done:                                       ; preds = %loop3_done
  %loop_counter_2 = add nuw nsw i64 %loop_counter_15, 1
  %exitcond9.not = icmp eq i64 %loop_counter_2, 25
  br i1 %exitcond9.not, label %loop_done, label %loop2_cond.preheader

loop_done:                                        ; preds = %loop2_done
  ret void
}

; Function Attrs: nounwind
define dso_local void @__orig_main(i64 %n1, i64 %n2, i64 %n3, i64 %n4, i64 %n5) local_unnamed_addr #5 {
bodyj.preheader:
  %0 = call i64 @llvm.smin.i64(i64 %n1, i64 %n2)
  %1 = call i64 @llvm.smax.i64(i64 %n1, i64 %n2)
  %2 = call i64 @llvm.smin.i64(i64 %1, i64 %n3)
  %3 = call i64 @llvm.smax.i64(i64 %1, i64 %n3)
  %4 = call i64 @llvm.smin.i64(i64 %3, i64 %n4)
  %5 = call i64 @llvm.smax.i64(i64 %3, i64 %n4)
  %6 = call i64 @llvm.smax.i64(i64 %5, i64 %n5)
  %7 = call i64 @llvm.smin.i64(i64 %5, i64 %n5)
  %8 = call i64 @llvm.smin.i64(i64 %0, i64 %2)
  %9 = call i64 @llvm.smax.i64(i64 %0, i64 %2)
  %10 = call i64 @llvm.smin.i64(i64 %9, i64 %4)
  %11 = call i64 @llvm.smax.i64(i64 %9, i64 %4)
  %12 = call i64 @llvm.smax.i64(i64 %11, i64 %7)
  %13 = call i64 @llvm.smin.i64(i64 %11, i64 %7)
  %14 = call i64 @llvm.smin.i64(i64 %8, i64 %10)
  %15 = call i64 @llvm.smax.i64(i64 %8, i64 %10)
  %16 = call i64 @llvm.smax.i64(i64 %15, i64 %13)
  %17 = call i64 @llvm.smin.i64(i64 %15, i64 %13)
  %18 = call i64 @llvm.smax.i64(i64 %14, i64 %17)
  %19 = call i64 @llvm.smin.i64(i64 %14, i64 %17)
  %20 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %19) #5
  %21 = tail call i32 @putchar(i32 10) #5
  %22 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %18) #5
  %23 = tail call i32 @putchar(i32 10) #5
  %24 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %16) #5
  %25 = tail call i32 @putchar(i32 10) #5
  %26 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %12) #5
  %27 = tail call i32 @putchar(i32 10) #5
  %28 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %6) #5
  %29 = tail call i32 @putchar(i32 10) #5
  ret void
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %2, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #8
  ret i32 0

2:                                                ; preds = %0
  tail call void @__main()
  ret i32 0
}

; Function Attrs: nocallback nofree nosync nounwind readnone speculatable willreturn
declare i64 @llvm.smin.i64(i64, i64) #6

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
attributes #4 = { argmemonly mustprogress nofree norecurse nosync nounwind willreturn }
attributes #5 = { nounwind }
attributes #6 = { nocallback nofree nosync nounwind readnone speculatable willreturn }
attributes #7 = { cold minsize noreturn }
attributes #8 = { noinline }
