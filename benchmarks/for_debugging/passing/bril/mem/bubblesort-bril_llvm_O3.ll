; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpSpyMeJ/bubblesort-init.ll'
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
declare dso_local noundef i32 @printf(ptr nocapture noundef readonly, ...) local_unnamed_addr #0

declare dso_local void @exit(i32) local_unnamed_addr

; Function Attrs: mustprogress nofree nounwind willreturn allockind("alloc,uninitialized") allocsize(0) memory(inaccessiblemem: readwrite)
declare dso_local noalias noundef ptr @malloc(i64 noundef) local_unnamed_addr #1

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read)
define dso_local i32 @btoi(ptr nocapture readonly %0) local_unnamed_addr #2 {
  %2 = load i8, ptr %0, align 1
  %3 = icmp eq i8 %2, 116
  %4 = zext i1 %3 to i32
  ret i32 %4
}

; Function Attrs: nofree nounwind
define dso_local void @print_bool(i1 %0) local_unnamed_addr #0 {
  %.str..str.1 = select i1 %0, ptr @.str, ptr @.str.1
  %2 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) %.str..str.1)
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
  %2 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %0)
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @print_ptr(ptr nocapture readnone %0) local_unnamed_addr #0 {
  %2 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.3)
  ret void
}

; Function Attrs: mustprogress nofree nounwind willreturn memory(write, argmem: none, inaccessiblemem: readwrite)
define dso_local noalias noundef ptr @__pack(i64 %size, i64 %n1, i64 %n2, i64 %n3, i64 %n4, i64 %n5) local_unnamed_addr #3 {
pre_entry:
  %z0 = shl i64 %size, 3
  %z1 = tail call ptr @malloc(i64 %z0)
  store i64 %n1, ptr %z1, align 8
  %loc_1 = getelementptr inbounds i64, ptr %z1, i64 1
  store i64 %n2, ptr %loc_1, align 8
  %loc_2 = getelementptr inbounds i64, ptr %z1, i64 2
  store i64 %n3, ptr %loc_2, align 8
  %loc_3 = getelementptr inbounds i64, ptr %z1, i64 3
  store i64 %n4, ptr %loc_3, align 8
  %loc_4 = getelementptr inbounds i64, ptr %z1, i64 4
  store i64 %n5, ptr %loc_4, align 8
  ret ptr %z1
}

; Function Attrs: nofree nounwind
define dso_local void @__print_array(ptr nocapture readonly %array, i64 %size) local_unnamed_addr #0 {
pre_entry:
  %cond_01 = icmp sgt i64 %size, 0
  br i1 %cond_01, label %body, label %done

body:                                             ; preds = %pre_entry, %body
  %i_12 = phi i64 [ %i_2, %body ], [ 0, %pre_entry ]
  %loc_0 = getelementptr inbounds i64, ptr %array, i64 %i_12
  %val_0 = load i64, ptr %loc_0, align 8
  %0 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %val_0)
  %1 = tail call i32 @putchar(i32 10)
  %i_2 = add nuw nsw i64 %i_12, 1
  %exitcond.not = icmp eq i64 %i_2, %size
  br i1 %exitcond.not, label %done, label %body

done:                                             ; preds = %body, %pre_entry
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: readwrite)
define dso_local void @__swap_cond(ptr nocapture %array, i64 %j) local_unnamed_addr #4 {
pre_entry:
  %loc_0 = getelementptr inbounds i64, ptr %array, i64 %j
  %loc_next_0 = getelementptr i64, ptr %loc_0, i64 1
  %elem_a_0 = load i64, ptr %loc_0, align 8
  %elem_b_0 = load i64, ptr %loc_next_0, align 8
  %cond_0 = icmp sgt i64 %elem_a_0, %elem_b_0
  br i1 %cond_0, label %swap, label %done

swap:                                             ; preds = %pre_entry
  store i64 %elem_b_0, ptr %loc_0, align 8
  store i64 %elem_a_0, ptr %loc_next_0, align 8
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
  %spec.select.i = tail call i64 @llvm.smin.i64(i64 %loop_counter_15, i64 %loop2_counter_14)
  %spec.select61.i = tail call i64 @llvm.smax.i64(i64 %loop_counter_15, i64 %loop2_counter_14)
  br label %loop4_cond.preheader

loop4_cond.preheader:                             ; preds = %loop3_cond.preheader, %loop4_done
  %loop3_counter_13 = phi i64 [ 10, %loop3_cond.preheader ], [ %loop3_counter_2, %loop4_done ]
  %elem_b_0.i.133.i = tail call i64 @llvm.smin.i64(i64 %spec.select61.i, i64 %loop3_counter_13)
  %elem_a_0.i.214.i = tail call i64 @llvm.smax.i64(i64 %spec.select61.i, i64 %loop3_counter_13)
  %cond_0.i.1.i = icmp ugt i64 %spec.select.i, %loop3_counter_13
  %elem_a_0.i.241.i = select i1 %cond_0.i.1.i, i64 %elem_b_0.i.133.i, i64 %spec.select.i
  %elem_a_0.i.1.1.i = select i1 %cond_0.i.1.i, i64 %spec.select.i, i64 %elem_b_0.i.133.i
  br label %loop5_cond.preheader

loop5_cond.preheader:                             ; preds = %loop4_cond.preheader, %loop5_done
  %loop4_counter_12 = phi i64 [ 10, %loop4_cond.preheader ], [ %loop4_counter_2, %loop5_done ]
  %elem_b_0.i.1.136.i = tail call i64 @llvm.smin.i64(i64 %elem_a_0.i.214.i, i64 %loop4_counter_12)
  %elem_a_0.i.321.i = tail call i64 @llvm.smax.i64(i64 %elem_a_0.i.214.i, i64 %loop4_counter_12)
  %elem_b_0.i.243.i = tail call i64 @llvm.smin.i64(i64 %elem_a_0.i.1.1.i, i64 %elem_b_0.i.1.136.i)
  %elem_a_0.i.1.2.i = tail call i64 @llvm.smax.i64(i64 %elem_a_0.i.1.1.i, i64 %elem_b_0.i.1.136.i)
  %elem_a_0.i.348.i = tail call i64 @llvm.smin.i64(i64 %elem_a_0.i.241.i, i64 %elem_b_0.i.243.i)
  %elem_a_0.i.2.1.i = tail call i64 @llvm.smax.i64(i64 %elem_a_0.i.241.i, i64 %elem_b_0.i.243.i)
  br label %loop5_body

loop5_body:                                       ; preds = %loop5_cond.preheader, %loop5_body
  %loop5_counter_11 = phi i64 [ 10, %loop5_cond.preheader ], [ %loop5_counter_2, %loop5_body ]
  %val_0.i.460.i = tail call i64 @llvm.smax.i64(i64 %elem_a_0.i.321.i, i64 %loop5_counter_11)
  %elem_b_0.i.1.239.i = tail call i64 @llvm.smin.i64(i64 %elem_a_0.i.321.i, i64 %loop5_counter_11)
  %val_0.i.358.i = tail call i64 @llvm.smax.i64(i64 %elem_a_0.i.1.2.i, i64 %elem_b_0.i.1.239.i)
  %elem_b_0.i.2.146.i = tail call i64 @llvm.smin.i64(i64 %elem_a_0.i.1.2.i, i64 %elem_b_0.i.1.239.i)
  %val_0.i.256.i = tail call i64 @llvm.smax.i64(i64 %elem_a_0.i.2.1.i, i64 %elem_b_0.i.2.146.i)
  %elem_b_0.i.350.i = tail call i64 @llvm.smin.i64(i64 %elem_a_0.i.2.1.i, i64 %elem_b_0.i.2.146.i)
  %cond_0.i.3.i = icmp ult i64 %elem_b_0.i.2.146.i, %elem_a_0.i.348.i
  %val_0.i.154.i = select i1 %cond_0.i.3.i, i64 %elem_a_0.i.348.i, i64 %elem_b_0.i.350.i
  %val_0.i52.i = select i1 %cond_0.i.3.i, i64 %elem_b_0.i.350.i, i64 %elem_a_0.i.348.i
  %0 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %val_0.i52.i)
  %1 = tail call i32 @putchar(i32 10)
  %2 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %val_0.i.154.i)
  %3 = tail call i32 @putchar(i32 10)
  %4 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %val_0.i.256.i)
  %5 = tail call i32 @putchar(i32 10)
  %6 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %val_0.i.358.i)
  %7 = tail call i32 @putchar(i32 10)
  %8 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %val_0.i.460.i)
  %9 = tail call i32 @putchar(i32 10)
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

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %n1, i64 %n2, i64 %n3, i64 %n4, i64 %n5) local_unnamed_addr #0 {
bodyj.preheader:
  %spec.select = tail call i64 @llvm.smin.i64(i64 %n1, i64 %n2)
  %spec.select61 = tail call i64 @llvm.smax.i64(i64 %n1, i64 %n2)
  %elem_b_0.i.133 = tail call i64 @llvm.smin.i64(i64 %spec.select61, i64 %n3)
  %elem_a_0.i.214 = tail call i64 @llvm.smax.i64(i64 %spec.select61, i64 %n3)
  %elem_b_0.i.1.136 = tail call i64 @llvm.smin.i64(i64 %elem_a_0.i.214, i64 %n4)
  %elem_a_0.i.321 = tail call i64 @llvm.smax.i64(i64 %elem_a_0.i.214, i64 %n4)
  %val_0.i.460 = tail call i64 @llvm.smax.i64(i64 %elem_a_0.i.321, i64 %n5)
  %elem_b_0.i.1.239 = tail call i64 @llvm.smin.i64(i64 %elem_a_0.i.321, i64 %n5)
  %cond_0.i.1 = icmp sgt i64 %spec.select, %n3
  %elem_a_0.i.241 = select i1 %cond_0.i.1, i64 %elem_b_0.i.133, i64 %spec.select
  %elem_a_0.i.1.1 = select i1 %cond_0.i.1, i64 %spec.select, i64 %elem_b_0.i.133
  %elem_b_0.i.243 = tail call i64 @llvm.smin.i64(i64 %elem_a_0.i.1.1, i64 %elem_b_0.i.1.136)
  %elem_a_0.i.1.2 = tail call i64 @llvm.smax.i64(i64 %elem_a_0.i.1.1, i64 %elem_b_0.i.1.136)
  %val_0.i.358 = tail call i64 @llvm.smax.i64(i64 %elem_a_0.i.1.2, i64 %elem_b_0.i.1.239)
  %elem_b_0.i.2.146 = tail call i64 @llvm.smin.i64(i64 %elem_a_0.i.1.2, i64 %elem_b_0.i.1.239)
  %elem_a_0.i.348 = tail call i64 @llvm.smin.i64(i64 %elem_a_0.i.241, i64 %elem_b_0.i.243)
  %elem_a_0.i.2.1 = tail call i64 @llvm.smax.i64(i64 %elem_a_0.i.241, i64 %elem_b_0.i.243)
  %val_0.i.256 = tail call i64 @llvm.smax.i64(i64 %elem_a_0.i.2.1, i64 %elem_b_0.i.2.146)
  %elem_b_0.i.350 = tail call i64 @llvm.smin.i64(i64 %elem_a_0.i.2.1, i64 %elem_b_0.i.2.146)
  %cond_0.i.3 = icmp slt i64 %elem_b_0.i.2.146, %elem_a_0.i.348
  %val_0.i.154 = select i1 %cond_0.i.3, i64 %elem_a_0.i.348, i64 %elem_b_0.i.350
  %val_0.i52 = select i1 %cond_0.i.3, i64 %elem_b_0.i.350, i64 %elem_a_0.i.348
  %0 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %val_0.i52)
  %1 = tail call i32 @putchar(i32 10)
  %2 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %val_0.i.154)
  %3 = tail call i32 @putchar(i32 10)
  %4 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %val_0.i.256)
  %5 = tail call i32 @putchar(i32 10)
  %6 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %val_0.i.358)
  %7 = tail call i32 @putchar(i32 10)
  %8 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %val_0.i.460)
  %9 = tail call i32 @putchar(i32 10)
  ret void
}

define dso_local noundef i32 @main(i32 %argc, ptr nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %4, label %2

2:                                                ; preds = %0
  %3 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.4, i32 0, i32 %1)
  tail call void @exit(i32 2)
  unreachable

4:                                                ; preds = %0
  tail call void @__main()
  ret i32 0
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.smin.i64(i64, i64) #5

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.smax.i64(i64, i64) #5

attributes #0 = { nofree nounwind }
attributes #1 = { mustprogress nofree nounwind willreturn allockind("alloc,uninitialized") allocsize(0) memory(inaccessiblemem: readwrite) "alloc-family"="malloc" }
attributes #2 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read) }
attributes #3 = { mustprogress nofree nounwind willreturn memory(write, argmem: none, inaccessiblemem: readwrite) }
attributes #4 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: readwrite) }
attributes #5 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
