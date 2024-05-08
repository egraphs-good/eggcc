; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpNttRYC/compile.ll'
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
  br label %body.body_crit_edge.i.preheader

body.body_crit_edge.i.preheader:                  ; preds = %b0, %__orig_main.exit
  %loop_counter_16 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %__orig_main.exit ]
  %z0.i = shl i64 %loop_counter_16, 3
  %z1.i = tail call i8* @malloc(i64 %z0.i) #4
  %vals_0.i = bitcast i8* %z1.i to i64*
  %0 = bitcast i8* %z1.i to <2 x i64>*
  store <2 x i64> <i64 0, i64 1>, <2 x i64>* %0, align 8
  %vals_i_2.i1 = getelementptr inbounds i64, i64* %vals_0.i, i64 2
  store i64 1, i64* %vals_i_2.i1, align 8
  %1 = add nsw i64 %loop_counter_16, -4
  br label %body.body_crit_edge.i

body.body_crit_edge.i:                            ; preds = %body.body_crit_edge.i, %body.body_crit_edge.i.preheader
  %tmp_0.pre.i = phi i64 [ 1, %body.body_crit_edge.i.preheader ], [ %2, %body.body_crit_edge.i ]
  %i_minus_one_2.i5 = phi i64 [ 2, %body.body_crit_edge.i.preheader ], [ %i_minus_one_2.i, %body.body_crit_edge.i ]
  %i_minus_two_12.i4 = phi i64 [ 0, %body.body_crit_edge.i.preheader ], [ %i_minus_two_2.i, %body.body_crit_edge.i ]
  %i_minus_two_2.i = add nuw nsw i64 %i_minus_two_12.i4, 1
  %i_minus_one_2.i = add nuw nsw i64 %i_minus_one_2.i5, 1
  %vals_i_minus_two_0.phi.trans.insert.i = getelementptr inbounds i64, i64* %vals_0.i, i64 %i_minus_two_2.i
  %tmp2_0.pre.i = load i64, i64* %vals_i_minus_two_0.phi.trans.insert.i, align 8
  %2 = add i64 %tmp2_0.pre.i, %tmp_0.pre.i
  %vals_i_2.i = getelementptr inbounds i64, i64* %vals_0.i, i64 %i_minus_one_2.i
  store i64 %2, i64* %vals_i_2.i, align 8
  %exitcond.not.i = icmp eq i64 %i_minus_two_12.i4, %1
  br i1 %exitcond.not.i, label %__orig_main.exit, label %body.body_crit_edge.i

__orig_main.exit:                                 ; preds = %body.body_crit_edge.i
  %3 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %2) #4
  %4 = tail call i32 @putchar(i32 10) #4
  tail call void @free(i8* nonnull %z1.i) #4
  %loop_counter_2 = add nuw nsw i64 %loop_counter_16, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 30000
  br i1 %exitcond.not, label %loop_done, label %body.body_crit_edge.i.preheader

loop_done:                                        ; preds = %__orig_main.exit
  ret void
}

; Function Attrs: nounwind
define dso_local void @__orig_main(i64 %input) local_unnamed_addr #4 {
pre_entry:
  %z0 = shl i64 %input, 3
  %z1 = tail call i8* @malloc(i64 %z0)
  %vals_0 = bitcast i8* %z1 to i64*
  %0 = bitcast i8* %z1 to <2 x i64>*
  store <2 x i64> <i64 0, i64 1>, <2 x i64>* %0, align 8
  %cond_01 = icmp sgt i64 %input, 2
  br i1 %cond_01, label %body.preheader, label %done

body.preheader:                                   ; preds = %pre_entry
  %1 = add i64 %input, -3
  %vals_i_28 = getelementptr inbounds i64, i64* %vals_0, i64 2
  store i64 1, i64* %vals_i_28, align 8
  %exitcond.not9 = icmp eq i64 %1, 0
  br i1 %exitcond.not9, label %done.loopexit, label %body.body_crit_edge.lr.ph

body.body_crit_edge.lr.ph:                        ; preds = %body.preheader
  %scevgep = getelementptr i8, i8* %z1, i64 16
  %scevgep13 = bitcast i8* %scevgep to i64*
  %load_initial = load i64, i64* %scevgep13, align 8
  br label %body.body_crit_edge

body.body_crit_edge:                              ; preds = %body.body_crit_edge.lr.ph, %body.body_crit_edge
  %store_forwarded = phi i64 [ %load_initial, %body.body_crit_edge.lr.ph ], [ %2, %body.body_crit_edge ]
  %i_minus_one_212 = phi i64 [ 2, %body.body_crit_edge.lr.ph ], [ %i_minus_one_2, %body.body_crit_edge ]
  %i_minus_two_1211 = phi i64 [ 0, %body.body_crit_edge.lr.ph ], [ %i_minus_two_2, %body.body_crit_edge ]
  %i_1410 = phi i64 [ 2, %body.body_crit_edge.lr.ph ], [ %i_2, %body.body_crit_edge ]
  %i_minus_two_2 = add nuw nsw i64 %i_minus_two_1211, 1
  %i_2 = add nuw nsw i64 %i_1410, 1
  %vals_i_minus_two_0.phi.trans.insert = getelementptr inbounds i64, i64* %vals_0, i64 %i_minus_two_2
  %tmp2_0.pre = load i64, i64* %vals_i_minus_two_0.phi.trans.insert, align 8
  %2 = add i64 %tmp2_0.pre, %store_forwarded
  %vals_i_2 = getelementptr inbounds i64, i64* %vals_0, i64 %i_2
  store i64 %2, i64* %vals_i_2, align 8
  %i_minus_one_2 = add nuw nsw i64 %i_minus_one_212, 1
  %exitcond.not = icmp eq i64 %i_minus_two_2, %1
  br i1 %exitcond.not, label %done.loopexit, label %body.body_crit_edge

done.loopexit:                                    ; preds = %body.body_crit_edge, %body.preheader
  %i_minus_one_2.lcssa = phi i64 [ 2, %body.preheader ], [ %i_minus_one_2, %body.body_crit_edge ]
  %last_0.phi.trans.insert = getelementptr inbounds i64, i64* %vals_0, i64 %i_minus_one_2.lcssa
  %tmp_2.pre = load i64, i64* %last_0.phi.trans.insert, align 8
  br label %done

done:                                             ; preds = %done.loopexit, %pre_entry
  %tmp_2 = phi i64 [ 1, %pre_entry ], [ %tmp_2.pre, %done.loopexit ]
  %3 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %tmp_2) #4
  %4 = tail call i32 @putchar(i32 10) #4
  tail call void @free(i8* nonnull %z1)
  ret void
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %body.body_crit_edge.i.preheader.i, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #6
  ret i32 0

body.body_crit_edge.i.preheader.i:                ; preds = %0, %__orig_main.exit.i
  %loop_counter_16.i = phi i64 [ %loop_counter_2.i, %__orig_main.exit.i ], [ 10, %0 ]
  %z0.i.i = shl i64 %loop_counter_16.i, 3
  %z1.i.i = tail call i8* @malloc(i64 %z0.i.i) #4
  %vals_0.i.i = bitcast i8* %z1.i.i to i64*
  %2 = bitcast i8* %z1.i.i to <2 x i64>*
  store <2 x i64> <i64 0, i64 1>, <2 x i64>* %2, align 8
  %vals_i_2.i1.i = getelementptr inbounds i64, i64* %vals_0.i.i, i64 2
  store i64 1, i64* %vals_i_2.i1.i, align 8
  %3 = add nsw i64 %loop_counter_16.i, -4
  br label %body.body_crit_edge.i.i

body.body_crit_edge.i.i:                          ; preds = %body.body_crit_edge.i.i, %body.body_crit_edge.i.preheader.i
  %tmp_0.pre.i.i = phi i64 [ 1, %body.body_crit_edge.i.preheader.i ], [ %4, %body.body_crit_edge.i.i ]
  %i_minus_one_2.i5.i = phi i64 [ 2, %body.body_crit_edge.i.preheader.i ], [ %i_minus_one_2.i.i, %body.body_crit_edge.i.i ]
  %i_minus_two_12.i4.i = phi i64 [ 0, %body.body_crit_edge.i.preheader.i ], [ %i_minus_two_2.i.i, %body.body_crit_edge.i.i ]
  %i_minus_two_2.i.i = add nuw nsw i64 %i_minus_two_12.i4.i, 1
  %i_minus_one_2.i.i = add nuw nsw i64 %i_minus_one_2.i5.i, 1
  %vals_i_minus_two_0.phi.trans.insert.i.i = getelementptr inbounds i64, i64* %vals_0.i.i, i64 %i_minus_two_2.i.i
  %tmp2_0.pre.i.i = load i64, i64* %vals_i_minus_two_0.phi.trans.insert.i.i, align 8
  %4 = add i64 %tmp2_0.pre.i.i, %tmp_0.pre.i.i
  %vals_i_2.i.i = getelementptr inbounds i64, i64* %vals_0.i.i, i64 %i_minus_one_2.i.i
  store i64 %4, i64* %vals_i_2.i.i, align 8
  %exitcond.not.i.i = icmp eq i64 %i_minus_two_12.i4.i, %3
  br i1 %exitcond.not.i.i, label %__orig_main.exit.i, label %body.body_crit_edge.i.i

__orig_main.exit.i:                               ; preds = %body.body_crit_edge.i.i
  %5 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %4) #4
  %6 = tail call i32 @putchar(i32 10) #4
  tail call void @free(i8* nonnull %z1.i.i) #4
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_16.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 30000
  br i1 %exitcond.not.i, label %__main.exit, label %body.body_crit_edge.i.preheader.i

__main.exit:                                      ; preds = %__orig_main.exit.i
  ret i32 0
}

; Function Attrs: cold minsize noreturn
define internal void @main.cold.1(i32 %0) #5 {
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
attributes #5 = { cold minsize noreturn }
attributes #6 = { noinline }
