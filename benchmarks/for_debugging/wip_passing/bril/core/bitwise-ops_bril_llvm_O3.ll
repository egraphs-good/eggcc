; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpzvyapv/compile.ll'
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

; Function Attrs: mustprogress nofree norecurse nosync nounwind readnone willreturn
define dso_local i1 @__mod2(i64 %a) local_unnamed_addr #2 {
pre_entry:
  %0 = and i64 %a, -9223372036854775807
  %ans_0 = icmp eq i64 %0, 1
  ret i1 %ans_0
}

; Function Attrs: nofree norecurse nosync nounwind readnone
define dso_local i64 @__loop_subroutine(i64 %a, i64 %b, i1 %c) local_unnamed_addr #3 {
pre_entry:
  br label %here

here:                                             ; preds = %pre_entry, %here
  %a_16 = phi i64 [ %a, %pre_entry ], [ %a_2, %here ]
  %b_15 = phi i64 [ %b, %pre_entry ], [ %b_2, %here ]
  %i_14 = phi i64 [ 0, %pre_entry ], [ %i_2, %here ]
  %ans_13 = phi i64 [ 0, %pre_entry ], [ %ans_3, %here ]
  %to_add_12 = phi i64 [ 1, %pre_entry ], [ %to_add_2, %here ]
  %0 = and i64 %a_16, -9223372036854775807
  %ans_0.i = icmp eq i64 %0, 1
  %1 = and i64 %b_15, -9223372036854775807
  %ans_0.i1 = icmp eq i64 %1, 1
  %cond_add_0 = and i1 %ans_0.i1, %ans_0.i
  %cond_add_1 = or i1 %ans_0.i1, %ans_0.i
  %spec.select = select i1 %c, i1 %cond_add_1, i1 %cond_add_0
  %ans_2 = select i1 %spec.select, i64 %to_add_12, i64 0
  %ans_3 = add i64 %ans_2, %ans_13
  %a_2 = sdiv i64 %a_16, 2
  %b_2 = sdiv i64 %b_15, 2
  %to_add_2 = shl i64 %to_add_12, 1
  %i_2 = add nuw nsw i64 %i_14, 1
  %exitcond.not = icmp eq i64 %i_2, 64
  br i1 %exitcond.not, label %end, label %here

end:                                              ; preds = %here
  ret i64 %ans_3
}

; Function Attrs: nofree norecurse nosync nounwind readnone
define dso_local i64 @__OR(i64 %a, i64 %b) local_unnamed_addr #3 {
pre_entry:
  br label %here.i

here.i:                                           ; preds = %here.i, %pre_entry
  %a_16.i = phi i64 [ %a, %pre_entry ], [ %a_2.i, %here.i ]
  %b_15.i = phi i64 [ %b, %pre_entry ], [ %b_2.i, %here.i ]
  %i_14.i = phi i64 [ 0, %pre_entry ], [ %i_2.i, %here.i ]
  %ans_13.i = phi i64 [ 0, %pre_entry ], [ %ans_3.i, %here.i ]
  %to_add_12.i = phi i64 [ 1, %pre_entry ], [ %to_add_2.i, %here.i ]
  %0 = and i64 %a_16.i, -9223372036854775807
  %ans_0.i.i = icmp eq i64 %0, 1
  %1 = and i64 %b_15.i, -9223372036854775807
  %ans_0.i1.i = icmp eq i64 %1, 1
  %cond_add_1.i = or i1 %ans_0.i.i, %ans_0.i1.i
  %ans_2.i = select i1 %cond_add_1.i, i64 %to_add_12.i, i64 0
  %ans_3.i = add i64 %ans_2.i, %ans_13.i
  %a_2.i = sdiv i64 %a_16.i, 2
  %b_2.i = sdiv i64 %b_15.i, 2
  %to_add_2.i = shl i64 %to_add_12.i, 1
  %i_2.i = add nuw nsw i64 %i_14.i, 1
  %exitcond.not.i = icmp eq i64 %i_2.i, 64
  br i1 %exitcond.not.i, label %__loop_subroutine.exit, label %here.i

__loop_subroutine.exit:                           ; preds = %here.i
  ret i64 %ans_3.i
}

; Function Attrs: nofree norecurse nosync nounwind readnone
define dso_local i64 @__AND(i64 %a, i64 %b) local_unnamed_addr #3 {
pre_entry:
  br label %here.i

here.i:                                           ; preds = %here.i, %pre_entry
  %a_16.i = phi i64 [ %a, %pre_entry ], [ %a_2.i, %here.i ]
  %b_15.i = phi i64 [ %b, %pre_entry ], [ %b_2.i, %here.i ]
  %i_14.i = phi i64 [ 0, %pre_entry ], [ %i_2.i, %here.i ]
  %ans_13.i = phi i64 [ 0, %pre_entry ], [ %ans_3.i, %here.i ]
  %to_add_12.i = phi i64 [ 1, %pre_entry ], [ %to_add_2.i, %here.i ]
  %0 = and i64 %a_16.i, -9223372036854775807
  %ans_0.i.i = icmp eq i64 %0, 1
  %1 = and i64 %b_15.i, -9223372036854775807
  %ans_0.i1.i = icmp eq i64 %1, 1
  %cond_add_0.i = and i1 %ans_0.i.i, %ans_0.i1.i
  %ans_2.i = select i1 %cond_add_0.i, i64 %to_add_12.i, i64 0
  %ans_3.i = add i64 %ans_2.i, %ans_13.i
  %a_2.i = sdiv i64 %a_16.i, 2
  %b_2.i = sdiv i64 %b_15.i, 2
  %to_add_2.i = shl i64 %to_add_12.i, 1
  %i_2.i = add nuw nsw i64 %i_14.i, 1
  %exitcond.not.i = icmp eq i64 %i_2.i, 64
  br i1 %exitcond.not.i, label %__loop_subroutine.exit, label %here.i

__loop_subroutine.exit:                           ; preds = %here.i
  ret i64 %ans_3.i
}

; Function Attrs: nofree norecurse nosync nounwind readnone
define dso_local i64 @__XOR(i64 %a, i64 %b) local_unnamed_addr #3 {
pre_entry:
  br label %here.i.i

here.i.i:                                         ; preds = %here.i.i, %pre_entry
  %a_16.i.i = phi i64 [ %a, %pre_entry ], [ %a_2.i.i, %here.i.i ]
  %b_15.i.i = phi i64 [ %b, %pre_entry ], [ %b_2.i.i, %here.i.i ]
  %i_14.i.i = phi i64 [ 0, %pre_entry ], [ %i_2.i.i, %here.i.i ]
  %ans_13.i.i = phi i64 [ 0, %pre_entry ], [ %ans_3.i.i, %here.i.i ]
  %to_add_12.i.i = phi i64 [ 1, %pre_entry ], [ %to_add_2.i.i, %here.i.i ]
  %0 = and i64 %a_16.i.i, -9223372036854775807
  %ans_0.i.i.i = icmp eq i64 %0, 1
  %1 = and i64 %b_15.i.i, -9223372036854775807
  %ans_0.i1.i.i = icmp eq i64 %1, 1
  %cond_add_0.i.i = and i1 %ans_0.i.i.i, %ans_0.i1.i.i
  %ans_2.i.i = select i1 %cond_add_0.i.i, i64 %to_add_12.i.i, i64 0
  %ans_3.i.i = add i64 %ans_2.i.i, %ans_13.i.i
  %a_2.i.i = sdiv i64 %a_16.i.i, 2
  %b_2.i.i = sdiv i64 %b_15.i.i, 2
  %to_add_2.i.i = shl i64 %to_add_12.i.i, 1
  %i_2.i.i = add nuw nsw i64 %i_14.i.i, 1
  %exitcond.not.i.i = icmp eq i64 %i_2.i.i, 64
  br i1 %exitcond.not.i.i, label %here.i.i15, label %here.i.i

here.i.i15:                                       ; preds = %here.i.i, %here.i.i15
  %a_16.i.i1 = phi i64 [ %a_2.i.i10, %here.i.i15 ], [ %a, %here.i.i ]
  %b_15.i.i2 = phi i64 [ %b_2.i.i11, %here.i.i15 ], [ %b, %here.i.i ]
  %i_14.i.i3 = phi i64 [ %i_2.i.i13, %here.i.i15 ], [ 0, %here.i.i ]
  %ans_13.i.i4 = phi i64 [ %ans_3.i.i9, %here.i.i15 ], [ 0, %here.i.i ]
  %to_add_12.i.i5 = phi i64 [ %to_add_2.i.i12, %here.i.i15 ], [ 1, %here.i.i ]
  %2 = and i64 %a_16.i.i1, -9223372036854775807
  %ans_0.i.i.i6 = icmp eq i64 %2, 1
  %3 = and i64 %b_15.i.i2, -9223372036854775807
  %ans_0.i1.i.i7 = icmp eq i64 %3, 1
  %cond_add_1.i.i = or i1 %ans_0.i.i.i6, %ans_0.i1.i.i7
  %ans_2.i.i8 = select i1 %cond_add_1.i.i, i64 %to_add_12.i.i5, i64 0
  %ans_3.i.i9 = add i64 %ans_2.i.i8, %ans_13.i.i4
  %a_2.i.i10 = sdiv i64 %a_16.i.i1, 2
  %b_2.i.i11 = sdiv i64 %b_15.i.i2, 2
  %to_add_2.i.i12 = shl i64 %to_add_12.i.i5, 1
  %i_2.i.i13 = add nuw nsw i64 %i_14.i.i3, 1
  %exitcond.not.i.i14 = icmp eq i64 %i_2.i.i13, 64
  br i1 %exitcond.not.i.i14, label %__OR.exit, label %here.i.i15

__OR.exit:                                        ; preds = %here.i.i15
  %ans_0 = sub i64 %ans_3.i.i9, %ans_3.i.i
  ret i64 %ans_0
}

; Function Attrs: nofree nounwind
define dso_local void @__main() local_unnamed_addr #0 {
b0:
  br label %loop2_cond.preheader

loop2_cond.preheader:                             ; preds = %b0, %loop2_done
  %loop_counter_14 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %loop2_done ]
  br label %loop3_cond.preheader

loop3_cond.preheader:                             ; preds = %loop2_cond.preheader, %loop3_done
  %loop2_counter_13 = phi i64 [ 10, %loop2_cond.preheader ], [ %loop2_counter_2, %loop3_done ]
  br label %here.i.i.i.i.preheader

here.i.i.i.i.preheader:                           ; preds = %loop3_cond.preheader, %__orig_main.exit
  %loop3_counter_12 = phi i64 [ 10, %loop3_cond.preheader ], [ %loop3_counter_2, %__orig_main.exit ]
  br label %here.i.i.i.i

here.i.i.i.i:                                     ; preds = %here.i.i.i.i.preheader, %here.i.i.i.i
  %a_16.i.i.i.i = phi i64 [ %a_2.i.i.i.i, %here.i.i.i.i ], [ %loop_counter_14, %here.i.i.i.i.preheader ]
  %b_15.i.i.i.i = phi i64 [ %b_2.i.i.i.i, %here.i.i.i.i ], [ %loop2_counter_13, %here.i.i.i.i.preheader ]
  %i_14.i.i.i.i = phi i64 [ %i_2.i.i.i.i, %here.i.i.i.i ], [ 0, %here.i.i.i.i.preheader ]
  %ans_13.i.i.i.i = phi i64 [ %ans_3.i.i.i.i, %here.i.i.i.i ], [ 0, %here.i.i.i.i.preheader ]
  %to_add_12.i.i.i.i = phi i64 [ %to_add_2.i.i.i.i, %here.i.i.i.i ], [ 1, %here.i.i.i.i.preheader ]
  %0 = and i64 %a_16.i.i.i.i, -9223372036854775807
  %ans_0.i.i.i.i.i = icmp eq i64 %0, 1
  %1 = and i64 %b_15.i.i.i.i, -9223372036854775807
  %ans_0.i1.i.i.i.i = icmp eq i64 %1, 1
  %cond_add_0.i.i.i.i = and i1 %ans_0.i.i.i.i.i, %ans_0.i1.i.i.i.i
  %ans_2.i.i.i.i = select i1 %cond_add_0.i.i.i.i, i64 %to_add_12.i.i.i.i, i64 0
  %ans_3.i.i.i.i = add i64 %ans_2.i.i.i.i, %ans_13.i.i.i.i
  %a_2.i.i.i.i = sdiv i64 %a_16.i.i.i.i, 2
  %b_2.i.i.i.i = sdiv i64 %b_15.i.i.i.i, 2
  %to_add_2.i.i.i.i = shl i64 %to_add_12.i.i.i.i, 1
  %i_2.i.i.i.i = add nuw nsw i64 %i_14.i.i.i.i, 1
  %exitcond.not.i.i.i.i = icmp eq i64 %i_2.i.i.i.i, 64
  br i1 %exitcond.not.i.i.i.i, label %here.i.i15.i.i, label %here.i.i.i.i

here.i.i15.i.i:                                   ; preds = %here.i.i.i.i, %here.i.i15.i.i
  %a_16.i.i1.i.i = phi i64 [ %a_2.i.i10.i.i, %here.i.i15.i.i ], [ %loop_counter_14, %here.i.i.i.i ]
  %b_15.i.i2.i.i = phi i64 [ %b_2.i.i11.i.i, %here.i.i15.i.i ], [ %loop2_counter_13, %here.i.i.i.i ]
  %i_14.i.i3.i.i = phi i64 [ %i_2.i.i13.i.i, %here.i.i15.i.i ], [ 0, %here.i.i.i.i ]
  %ans_13.i.i4.i.i = phi i64 [ %ans_3.i.i9.i.i, %here.i.i15.i.i ], [ 0, %here.i.i.i.i ]
  %to_add_12.i.i5.i.i = phi i64 [ %to_add_2.i.i12.i.i, %here.i.i15.i.i ], [ 1, %here.i.i.i.i ]
  %2 = and i64 %a_16.i.i1.i.i, -9223372036854775807
  %ans_0.i.i.i6.i.i = icmp eq i64 %2, 1
  %3 = and i64 %b_15.i.i2.i.i, -9223372036854775807
  %ans_0.i1.i.i7.i.i = icmp eq i64 %3, 1
  %cond_add_1.i.i.i.i = or i1 %ans_0.i.i.i6.i.i, %ans_0.i1.i.i7.i.i
  %ans_2.i.i8.i.i = select i1 %cond_add_1.i.i.i.i, i64 %to_add_12.i.i5.i.i, i64 0
  %ans_3.i.i9.i.i = add i64 %ans_2.i.i8.i.i, %ans_13.i.i4.i.i
  %a_2.i.i10.i.i = sdiv i64 %a_16.i.i1.i.i, 2
  %b_2.i.i11.i.i = sdiv i64 %b_15.i.i2.i.i, 2
  %to_add_2.i.i12.i.i = shl i64 %to_add_12.i.i5.i.i, 1
  %i_2.i.i13.i.i = add nuw nsw i64 %i_14.i.i3.i.i, 1
  %exitcond.not.i.i14.i.i = icmp eq i64 %i_2.i.i13.i.i, 64
  br i1 %exitcond.not.i.i14.i.i, label %__orig_main.exit, label %here.i.i15.i.i

__orig_main.exit:                                 ; preds = %here.i.i15.i.i
  %ans_0.i.i = sub i64 %ans_3.i.i9.i.i, %ans_3.i.i.i.i
  %4 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %ans_0.i.i) #5
  %5 = tail call i32 @putchar(i32 10) #5
  %loop3_counter_2 = add nuw nsw i64 %loop3_counter_12, 1
  %exitcond.not = icmp eq i64 %loop3_counter_2, 100
  br i1 %exitcond.not, label %loop3_done, label %here.i.i.i.i.preheader

loop3_done:                                       ; preds = %__orig_main.exit
  %loop2_counter_2 = add nuw nsw i64 %loop2_counter_13, 1
  %exitcond6.not = icmp eq i64 %loop2_counter_2, 100
  br i1 %exitcond6.not, label %loop2_done, label %loop3_cond.preheader

loop2_done:                                       ; preds = %loop3_done
  %loop_counter_2 = add nuw nsw i64 %loop_counter_14, 1
  %exitcond7.not = icmp eq i64 %loop_counter_2, 100
  br i1 %exitcond7.not, label %loop_done, label %loop2_cond.preheader

loop_done:                                        ; preds = %loop2_done
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %a, i64 %b, i64 %c) local_unnamed_addr #0 {
pre_entry:
  %sel_0 = add i64 %c, -1
  %less_0 = icmp slt i64 %sel_0, 0
  br i1 %less_0, label %here.i.i, label %useless_lbl

useless_lbl:                                      ; preds = %pre_entry
  %equal_0 = icmp eq i64 %sel_0, 0
  br i1 %equal_0, label %here.i.i15, label %here.i.i.i

here.i.i:                                         ; preds = %pre_entry, %here.i.i
  %a_16.i.i = phi i64 [ %a_2.i.i, %here.i.i ], [ %a, %pre_entry ]
  %b_15.i.i = phi i64 [ %b_2.i.i, %here.i.i ], [ %b, %pre_entry ]
  %i_14.i.i = phi i64 [ %i_2.i.i, %here.i.i ], [ 0, %pre_entry ]
  %ans_13.i.i = phi i64 [ %ans_3.i.i, %here.i.i ], [ 0, %pre_entry ]
  %to_add_12.i.i = phi i64 [ %to_add_2.i.i, %here.i.i ], [ 1, %pre_entry ]
  %0 = and i64 %a_16.i.i, -9223372036854775807
  %ans_0.i.i.i = icmp eq i64 %0, 1
  %1 = and i64 %b_15.i.i, -9223372036854775807
  %ans_0.i1.i.i = icmp eq i64 %1, 1
  %cond_add_0.i.i = and i1 %ans_0.i.i.i, %ans_0.i1.i.i
  %ans_2.i.i = select i1 %cond_add_0.i.i, i64 %to_add_12.i.i, i64 0
  %ans_3.i.i = add i64 %ans_2.i.i, %ans_13.i.i
  %a_2.i.i = sdiv i64 %a_16.i.i, 2
  %b_2.i.i = sdiv i64 %b_15.i.i, 2
  %to_add_2.i.i = shl i64 %to_add_12.i.i, 1
  %i_2.i.i = add nuw nsw i64 %i_14.i.i, 1
  %exitcond.not.i.i = icmp eq i64 %i_2.i.i, 64
  br i1 %exitcond.not.i.i, label %end, label %here.i.i

here.i.i15:                                       ; preds = %useless_lbl, %here.i.i15
  %a_16.i.i1 = phi i64 [ %a_2.i.i10, %here.i.i15 ], [ %a, %useless_lbl ]
  %b_15.i.i2 = phi i64 [ %b_2.i.i11, %here.i.i15 ], [ %b, %useless_lbl ]
  %i_14.i.i3 = phi i64 [ %i_2.i.i13, %here.i.i15 ], [ 0, %useless_lbl ]
  %ans_13.i.i4 = phi i64 [ %ans_3.i.i9, %here.i.i15 ], [ 0, %useless_lbl ]
  %to_add_12.i.i5 = phi i64 [ %to_add_2.i.i12, %here.i.i15 ], [ 1, %useless_lbl ]
  %2 = and i64 %a_16.i.i1, -9223372036854775807
  %ans_0.i.i.i6 = icmp eq i64 %2, 1
  %3 = and i64 %b_15.i.i2, -9223372036854775807
  %ans_0.i1.i.i7 = icmp eq i64 %3, 1
  %cond_add_1.i.i = or i1 %ans_0.i.i.i6, %ans_0.i1.i.i7
  %ans_2.i.i8 = select i1 %cond_add_1.i.i, i64 %to_add_12.i.i5, i64 0
  %ans_3.i.i9 = add i64 %ans_2.i.i8, %ans_13.i.i4
  %a_2.i.i10 = sdiv i64 %a_16.i.i1, 2
  %b_2.i.i11 = sdiv i64 %b_15.i.i2, 2
  %to_add_2.i.i12 = shl i64 %to_add_12.i.i5, 1
  %i_2.i.i13 = add nuw nsw i64 %i_14.i.i3, 1
  %exitcond.not.i.i14 = icmp eq i64 %i_2.i.i13, 64
  br i1 %exitcond.not.i.i14, label %end, label %here.i.i15

here.i.i.i:                                       ; preds = %useless_lbl, %here.i.i.i
  %a_16.i.i.i = phi i64 [ %a_2.i.i.i, %here.i.i.i ], [ %a, %useless_lbl ]
  %b_15.i.i.i = phi i64 [ %b_2.i.i.i, %here.i.i.i ], [ %b, %useless_lbl ]
  %i_14.i.i.i = phi i64 [ %i_2.i.i.i, %here.i.i.i ], [ 0, %useless_lbl ]
  %ans_13.i.i.i = phi i64 [ %ans_3.i.i.i, %here.i.i.i ], [ 0, %useless_lbl ]
  %to_add_12.i.i.i = phi i64 [ %to_add_2.i.i.i, %here.i.i.i ], [ 1, %useless_lbl ]
  %4 = and i64 %a_16.i.i.i, -9223372036854775807
  %ans_0.i.i.i.i = icmp eq i64 %4, 1
  %5 = and i64 %b_15.i.i.i, -9223372036854775807
  %ans_0.i1.i.i.i = icmp eq i64 %5, 1
  %cond_add_0.i.i.i = and i1 %ans_0.i.i.i.i, %ans_0.i1.i.i.i
  %ans_2.i.i.i = select i1 %cond_add_0.i.i.i, i64 %to_add_12.i.i.i, i64 0
  %ans_3.i.i.i = add i64 %ans_2.i.i.i, %ans_13.i.i.i
  %a_2.i.i.i = sdiv i64 %a_16.i.i.i, 2
  %b_2.i.i.i = sdiv i64 %b_15.i.i.i, 2
  %to_add_2.i.i.i = shl i64 %to_add_12.i.i.i, 1
  %i_2.i.i.i = add nuw nsw i64 %i_14.i.i.i, 1
  %exitcond.not.i.i.i = icmp eq i64 %i_2.i.i.i, 64
  br i1 %exitcond.not.i.i.i, label %here.i.i15.i, label %here.i.i.i

here.i.i15.i:                                     ; preds = %here.i.i.i, %here.i.i15.i
  %a_16.i.i1.i = phi i64 [ %a_2.i.i10.i, %here.i.i15.i ], [ %a, %here.i.i.i ]
  %b_15.i.i2.i = phi i64 [ %b_2.i.i11.i, %here.i.i15.i ], [ %b, %here.i.i.i ]
  %i_14.i.i3.i = phi i64 [ %i_2.i.i13.i, %here.i.i15.i ], [ 0, %here.i.i.i ]
  %ans_13.i.i4.i = phi i64 [ %ans_3.i.i9.i, %here.i.i15.i ], [ 0, %here.i.i.i ]
  %to_add_12.i.i5.i = phi i64 [ %to_add_2.i.i12.i, %here.i.i15.i ], [ 1, %here.i.i.i ]
  %6 = and i64 %a_16.i.i1.i, -9223372036854775807
  %ans_0.i.i.i6.i = icmp eq i64 %6, 1
  %7 = and i64 %b_15.i.i2.i, -9223372036854775807
  %ans_0.i1.i.i7.i = icmp eq i64 %7, 1
  %cond_add_1.i.i.i = or i1 %ans_0.i.i.i6.i, %ans_0.i1.i.i7.i
  %ans_2.i.i8.i = select i1 %cond_add_1.i.i.i, i64 %to_add_12.i.i5.i, i64 0
  %ans_3.i.i9.i = add i64 %ans_2.i.i8.i, %ans_13.i.i4.i
  %a_2.i.i10.i = sdiv i64 %a_16.i.i1.i, 2
  %b_2.i.i11.i = sdiv i64 %b_15.i.i2.i, 2
  %to_add_2.i.i12.i = shl i64 %to_add_12.i.i5.i, 1
  %i_2.i.i13.i = add nuw nsw i64 %i_14.i.i3.i, 1
  %exitcond.not.i.i14.i = icmp eq i64 %i_2.i.i13.i, 64
  br i1 %exitcond.not.i.i14.i, label %__XOR.exit, label %here.i.i15.i

__XOR.exit:                                       ; preds = %here.i.i15.i
  %ans_0.i = sub i64 %ans_3.i.i9.i, %ans_3.i.i.i
  br label %end

end:                                              ; preds = %here.i.i15, %here.i.i, %__XOR.exit
  %ans_4 = phi i64 [ %ans_0.i, %__XOR.exit ], [ %ans_3.i.i, %here.i.i ], [ %ans_3.i.i9, %here.i.i15 ]
  %8 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %ans_4) #5
  %9 = tail call i32 @putchar(i32 10) #5
  ret void
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %loop2_cond.preheader.i, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #6
  ret i32 0

loop2_cond.preheader.i:                           ; preds = %0, %loop2_done.i
  %loop_counter_14.i = phi i64 [ %loop_counter_2.i, %loop2_done.i ], [ 10, %0 ]
  br label %loop3_cond.preheader.i

loop3_cond.preheader.i:                           ; preds = %loop3_done.i, %loop2_cond.preheader.i
  %loop2_counter_13.i = phi i64 [ 10, %loop2_cond.preheader.i ], [ %loop2_counter_2.i, %loop3_done.i ]
  br label %here.i.i.i.i.preheader.i

here.i.i.i.i.preheader.i:                         ; preds = %__orig_main.exit.i, %loop3_cond.preheader.i
  %loop3_counter_12.i = phi i64 [ 10, %loop3_cond.preheader.i ], [ %loop3_counter_2.i, %__orig_main.exit.i ]
  br label %here.i.i.i.i.i

here.i.i.i.i.i:                                   ; preds = %here.i.i.i.i.i, %here.i.i.i.i.preheader.i
  %a_16.i.i.i.i.i = phi i64 [ %a_2.i.i.i.i.i, %here.i.i.i.i.i ], [ %loop_counter_14.i, %here.i.i.i.i.preheader.i ]
  %b_15.i.i.i.i.i = phi i64 [ %b_2.i.i.i.i.i, %here.i.i.i.i.i ], [ %loop2_counter_13.i, %here.i.i.i.i.preheader.i ]
  %i_14.i.i.i.i.i = phi i64 [ %i_2.i.i.i.i.i, %here.i.i.i.i.i ], [ 0, %here.i.i.i.i.preheader.i ]
  %ans_13.i.i.i.i.i = phi i64 [ %ans_3.i.i.i.i.i, %here.i.i.i.i.i ], [ 0, %here.i.i.i.i.preheader.i ]
  %to_add_12.i.i.i.i.i = phi i64 [ %to_add_2.i.i.i.i.i, %here.i.i.i.i.i ], [ 1, %here.i.i.i.i.preheader.i ]
  %2 = and i64 %a_16.i.i.i.i.i, -9223372036854775807
  %ans_0.i.i.i.i.i.i = icmp eq i64 %2, 1
  %3 = and i64 %b_15.i.i.i.i.i, -9223372036854775807
  %ans_0.i1.i.i.i.i.i = icmp eq i64 %3, 1
  %cond_add_0.i.i.i.i.i = and i1 %ans_0.i.i.i.i.i.i, %ans_0.i1.i.i.i.i.i
  %ans_2.i.i.i.i.i = select i1 %cond_add_0.i.i.i.i.i, i64 %to_add_12.i.i.i.i.i, i64 0
  %ans_3.i.i.i.i.i = add i64 %ans_2.i.i.i.i.i, %ans_13.i.i.i.i.i
  %a_2.i.i.i.i.i = sdiv i64 %a_16.i.i.i.i.i, 2
  %b_2.i.i.i.i.i = sdiv i64 %b_15.i.i.i.i.i, 2
  %to_add_2.i.i.i.i.i = shl i64 %to_add_12.i.i.i.i.i, 1
  %i_2.i.i.i.i.i = add nuw nsw i64 %i_14.i.i.i.i.i, 1
  %exitcond.not.i.i.i.i.i = icmp eq i64 %i_2.i.i.i.i.i, 64
  br i1 %exitcond.not.i.i.i.i.i, label %here.i.i15.i.i.i, label %here.i.i.i.i.i

here.i.i15.i.i.i:                                 ; preds = %here.i.i.i.i.i, %here.i.i15.i.i.i
  %a_16.i.i1.i.i.i = phi i64 [ %a_2.i.i10.i.i.i, %here.i.i15.i.i.i ], [ %loop_counter_14.i, %here.i.i.i.i.i ]
  %b_15.i.i2.i.i.i = phi i64 [ %b_2.i.i11.i.i.i, %here.i.i15.i.i.i ], [ %loop2_counter_13.i, %here.i.i.i.i.i ]
  %i_14.i.i3.i.i.i = phi i64 [ %i_2.i.i13.i.i.i, %here.i.i15.i.i.i ], [ 0, %here.i.i.i.i.i ]
  %ans_13.i.i4.i.i.i = phi i64 [ %ans_3.i.i9.i.i.i, %here.i.i15.i.i.i ], [ 0, %here.i.i.i.i.i ]
  %to_add_12.i.i5.i.i.i = phi i64 [ %to_add_2.i.i12.i.i.i, %here.i.i15.i.i.i ], [ 1, %here.i.i.i.i.i ]
  %4 = and i64 %a_16.i.i1.i.i.i, -9223372036854775807
  %ans_0.i.i.i6.i.i.i = icmp eq i64 %4, 1
  %5 = and i64 %b_15.i.i2.i.i.i, -9223372036854775807
  %ans_0.i1.i.i7.i.i.i = icmp eq i64 %5, 1
  %cond_add_1.i.i.i.i.i = or i1 %ans_0.i.i.i6.i.i.i, %ans_0.i1.i.i7.i.i.i
  %ans_2.i.i8.i.i.i = select i1 %cond_add_1.i.i.i.i.i, i64 %to_add_12.i.i5.i.i.i, i64 0
  %ans_3.i.i9.i.i.i = add i64 %ans_2.i.i8.i.i.i, %ans_13.i.i4.i.i.i
  %a_2.i.i10.i.i.i = sdiv i64 %a_16.i.i1.i.i.i, 2
  %b_2.i.i11.i.i.i = sdiv i64 %b_15.i.i2.i.i.i, 2
  %to_add_2.i.i12.i.i.i = shl i64 %to_add_12.i.i5.i.i.i, 1
  %i_2.i.i13.i.i.i = add nuw nsw i64 %i_14.i.i3.i.i.i, 1
  %exitcond.not.i.i14.i.i.i = icmp eq i64 %i_2.i.i13.i.i.i, 64
  br i1 %exitcond.not.i.i14.i.i.i, label %__orig_main.exit.i, label %here.i.i15.i.i.i

__orig_main.exit.i:                               ; preds = %here.i.i15.i.i.i
  %ans_0.i.i.i = sub i64 %ans_3.i.i9.i.i.i, %ans_3.i.i.i.i.i
  %6 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %ans_0.i.i.i) #5
  %7 = tail call i32 @putchar(i32 10) #5
  %loop3_counter_2.i = add nuw nsw i64 %loop3_counter_12.i, 1
  %exitcond.not.i = icmp eq i64 %loop3_counter_2.i, 100
  br i1 %exitcond.not.i, label %loop3_done.i, label %here.i.i.i.i.preheader.i

loop3_done.i:                                     ; preds = %__orig_main.exit.i
  %loop2_counter_2.i = add nuw nsw i64 %loop2_counter_13.i, 1
  %exitcond6.not.i = icmp eq i64 %loop2_counter_2.i, 100
  br i1 %exitcond6.not.i, label %loop2_done.i, label %loop3_cond.preheader.i

loop2_done.i:                                     ; preds = %loop3_done.i
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_14.i, 1
  %exitcond7.not.i = icmp eq i64 %loop_counter_2.i, 100
  br i1 %exitcond7.not.i, label %__main.exit, label %loop2_cond.preheader.i

__main.exit:                                      ; preds = %loop2_done.i
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
