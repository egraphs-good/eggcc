; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpzUkgQY/compile.ll'
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

; Function Attrs: inaccessiblemem_or_argmemonly mustprogress nounwind willreturn
declare dso_local void @free(i8* nocapture noundef) local_unnamed_addr #2

; Function Attrs: argmemonly mustprogress nofree norecurse nosync nounwind readonly willreturn
define dso_local i32 @btoi(i8* nocapture readonly %0) local_unnamed_addr #3 {
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

; Function Attrs: nounwind
define dso_local void @__main() local_unnamed_addr #4 {
b0:
  br label %loop_body

loop_body:                                        ; preds = %b0, %loop_body
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %loop_body ]
  tail call void @__orig_main(i64 %loop_counter_11)
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 500000
  br i1 %exitcond.not, label %loop_done, label %loop_body

loop_done:                                        ; preds = %loop_body
  ret void
}

; Function Attrs: nounwind
define dso_local void @__orig_main(i64 %x) local_unnamed_addr #4 {
pre_entry:
  %n1_0 = add i64 %x, 94
  %z1.i = tail call dereferenceable_or_null(48) i8* @malloc(i64 48) #4
  %array_0.i = bitcast i8* %z1.i to i64*
  store i64 %n1_0, i64* %array_0.i, align 8
  %loc_1.i = getelementptr inbounds i64, i64* %array_0.i, i64 1
  %loc_2.i = getelementptr inbounds i64, i64* %array_0.i, i64 2
  %0 = bitcast i64* %loc_1.i to <2 x i64>*
  store <2 x i64> <i64 21, i64 5>, <2 x i64>* %0, align 8
  %loc_3.i = getelementptr inbounds i64, i64* %array_0.i, i64 3
  %loc_4.i = getelementptr inbounds i64, i64* %array_0.i, i64 4
  %1 = bitcast i64* %loc_3.i to <2 x i64>*
  store <2 x i64> <i64 6, i64 82>, <2 x i64>* %1, align 8
  %loc_5.i = getelementptr inbounds i64, i64* %array_0.i, i64 5
  store i64 46, i64* %loc_5.i, align 8
  tail call void @__qsort(i64* nonnull %array_0.i, i64 0, i64 5)
  %val_0.i = load i64, i64* %array_0.i, align 8
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0.i) #4
  %3 = tail call i32 @putchar(i32 10) #4
  %val_0.i.1 = load i64, i64* %loc_1.i, align 8
  %4 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0.i.1) #4
  %5 = tail call i32 @putchar(i32 10) #4
  %val_0.i.2 = load i64, i64* %loc_2.i, align 8
  %6 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0.i.2) #4
  %7 = tail call i32 @putchar(i32 10) #4
  %val_0.i.3 = load i64, i64* %loc_3.i, align 8
  %8 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0.i.3) #4
  %9 = tail call i32 @putchar(i32 10) #4
  %val_0.i.4 = load i64, i64* %loc_4.i, align 8
  %10 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0.i.4) #4
  %11 = tail call i32 @putchar(i32 10) #4
  %val_0.i.5 = load i64, i64* %loc_5.i, align 8
  %12 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0.i.5) #4
  %13 = tail call i32 @putchar(i32 10) #4
  tail call void @free(i8* nonnull %z1.i)
  ret void
}

; Function Attrs: argmemonly nofree nosync nounwind
define dso_local void @__qsort(i64* %array, i64 %l, i64 %r) local_unnamed_addr #5 {
pre_entry:
  %neg_r_0 = icmp slt i64 %r, 0
  %l_ge_r_02 = icmp sge i64 %l, %r
  %ret_cond_03 = or i1 %neg_r_0, %l_ge_r_02
  br i1 %ret_cond_03, label %done, label %continue.lr.ph

continue.lr.ph:                                   ; preds = %pre_entry
  %pivot_loc_0.i = getelementptr inbounds i64, i64* %array, i64 %r
  br label %continue

continue:                                         ; preds = %continue.lr.ph, %__partition.exit
  %l.tr4 = phi i64 [ %l, %continue.lr.ph ], [ %p_plus_one_0, %__partition.exit ]
  %pivot_0.i = load i64, i64* %pivot_loc_0.i, align 8
  %i_0.i = add i64 %l.tr4, -1
  br label %loop.init.outer.i

loop.init.outer.i:                                ; preds = %swap.i, %continue
  %j_1.ph.i = phi i64 [ %i_0.i, %continue ], [ %j_2.i, %swap.i ]
  %i_1.ph.i = phi i64 [ %i_0.i, %continue ], [ %i_2.i, %swap.i ]
  br label %loop.init.i

loop.init.i:                                      ; preds = %body.i, %loop.init.outer.i
  %j_1.i = phi i64 [ %j_2.i, %body.i ], [ %j_1.ph.i, %loop.init.outer.i ]
  %j_2.i = add i64 %j_1.i, 1
  %cond_0.i = icmp slt i64 %j_2.i, %r
  br i1 %cond_0.i, label %body.i, label %__partition.exit

body.i:                                           ; preds = %loop.init.i
  %j_loc_0.i = getelementptr inbounds i64, i64* %array, i64 %j_2.i
  %a_j_0.i = load i64, i64* %j_loc_0.i, align 8
  %swap_cond_0.not.i = icmp sgt i64 %a_j_0.i, %pivot_0.i
  br i1 %swap_cond_0.not.i, label %loop.init.i, label %swap.i

swap.i:                                           ; preds = %body.i
  %j_loc_0.i.le = getelementptr inbounds i64, i64* %array, i64 %j_2.i
  %i_2.i = add i64 %i_1.ph.i, 1
  %i_loc_0.i = getelementptr inbounds i64, i64* %array, i64 %i_2.i
  %a_i_0.i = load i64, i64* %i_loc_0.i, align 8
  store i64 %a_i_0.i, i64* %j_loc_0.i.le, align 8
  store i64 %a_j_0.i, i64* %i_loc_0.i, align 8
  br label %loop.init.outer.i

__partition.exit:                                 ; preds = %loop.init.i
  %i_3.i = add i64 %i_1.ph.i, 1
  %i_loc_1.i = getelementptr inbounds i64, i64* %array, i64 %i_3.i
  %a_i_1.i = load i64, i64* %i_loc_1.i, align 8
  store i64 %pivot_0.i, i64* %i_loc_1.i, align 8
  store i64 %a_i_1.i, i64* %pivot_loc_0.i, align 8
  %p_plus_one_0 = add i64 %i_1.ph.i, 2
  tail call void @__qsort(i64* nonnull %array, i64 %l.tr4, i64 %i_1.ph.i)
  %l_ge_r_0.not = icmp slt i64 %p_plus_one_0, %r
  br i1 %l_ge_r_0.not, label %continue, label %done

done:                                             ; preds = %__partition.exit, %pre_entry
  ret void
}

; Function Attrs: argmemonly nofree norecurse nosync nounwind
define dso_local i64 @__partition(i64* nocapture %array, i64 %l, i64 %r) local_unnamed_addr #6 {
pre_entry:
  %pivot_loc_0 = getelementptr inbounds i64, i64* %array, i64 %r
  %pivot_0 = load i64, i64* %pivot_loc_0, align 8
  %i_0 = add i64 %l, -1
  br label %loop.init.outer

loop.init.outer:                                  ; preds = %pre_entry, %swap
  %j_1.ph = phi i64 [ %i_0, %pre_entry ], [ %j_2, %swap ]
  %i_1.ph = phi i64 [ %i_0, %pre_entry ], [ %i_2, %swap ]
  br label %loop.init

loop.init:                                        ; preds = %loop.init.outer, %body
  %j_1 = phi i64 [ %j_2, %body ], [ %j_1.ph, %loop.init.outer ]
  %j_2 = add i64 %j_1, 1
  %cond_0 = icmp slt i64 %j_2, %r
  br i1 %cond_0, label %body, label %post.loop

body:                                             ; preds = %loop.init
  %j_loc_0 = getelementptr inbounds i64, i64* %array, i64 %j_2
  %a_j_0 = load i64, i64* %j_loc_0, align 8
  %swap_cond_0.not = icmp sgt i64 %a_j_0, %pivot_0
  br i1 %swap_cond_0.not, label %loop.init, label %swap

swap:                                             ; preds = %body
  %j_loc_0.le = getelementptr inbounds i64, i64* %array, i64 %j_2
  %i_2 = add i64 %i_1.ph, 1
  %i_loc_0 = getelementptr inbounds i64, i64* %array, i64 %i_2
  %a_i_0 = load i64, i64* %i_loc_0, align 8
  store i64 %a_i_0, i64* %j_loc_0.le, align 8
  store i64 %a_j_0, i64* %i_loc_0, align 8
  br label %loop.init.outer

post.loop:                                        ; preds = %loop.init
  %i_3 = add i64 %i_1.ph, 1
  %i_loc_1 = getelementptr inbounds i64, i64* %array, i64 %i_3
  %a_i_1 = load i64, i64* %i_loc_1, align 8
  store i64 %pivot_0, i64* %i_loc_1, align 8
  store i64 %a_i_1, i64* %pivot_loc_0, align 8
  ret i64 %i_3
}

; Function Attrs: mustprogress nofree nounwind willreturn
define dso_local noalias i64* @__pack(i64 %size, i64 %n1, i64 %n2, i64 %n3, i64 %n4, i64 %n5, i64 %n6) local_unnamed_addr #7 {
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
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0) #4
  %1 = tail call i32 @putchar(i32 10) #4
  %i_2 = add nuw nsw i64 %i_12, 1
  %exitcond.not = icmp eq i64 %i_2, %size
  br i1 %exitcond.not, label %done, label %body

done:                                             ; preds = %body, %pre_entry
  ret void
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %loop_body.i, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #9
  ret i32 0

loop_body.i:                                      ; preds = %0, %loop_body.i
  %loop_counter_11.i = phi i64 [ %loop_counter_2.i, %loop_body.i ], [ 10, %0 ]
  tail call void @__orig_main(i64 %loop_counter_11.i) #4
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 500000
  br i1 %exitcond.not.i, label %__main.exit, label %loop_body.i

__main.exit:                                      ; preds = %loop_body.i
  ret i32 0
}

; Function Attrs: cold minsize noreturn
define internal void @main.cold.1(i32 %0) #8 {
newFuncRoot:
  br label %1

1:                                                ; preds = %newFuncRoot
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([33 x i8], [33 x i8]* @.str.4, i64 0, i64 0), i32 0, i32 %0)
  tail call void @exit(i32 2)
  unreachable
}

attributes #0 = { nofree nounwind }
attributes #1 = { inaccessiblememonly mustprogress nofree nounwind willreturn allocsize(0) }
attributes #2 = { inaccessiblemem_or_argmemonly mustprogress nounwind willreturn }
attributes #3 = { argmemonly mustprogress nofree norecurse nosync nounwind readonly willreturn }
attributes #4 = { nounwind }
attributes #5 = { argmemonly nofree nosync nounwind }
attributes #6 = { argmemonly nofree norecurse nosync nounwind }
attributes #7 = { mustprogress nofree nounwind willreturn }
attributes #8 = { cold minsize noreturn }
attributes #9 = { noinline }
