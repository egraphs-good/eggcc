; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmp4jpLVJ/factors-init.ll'
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

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read)
define dso_local i32 @btoi(ptr nocapture readonly %0) local_unnamed_addr #1 {
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

; Function Attrs: nofree nounwind
define dso_local void @__main() local_unnamed_addr #0 {
b0:
  br label %loopbody.lr.ph.i.preheader

loopbody.lr.ph.i.preheader:                       ; preds = %b0, %__orig_main.exit
  %loop_counter_12 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %__orig_main.exit ]
  br label %loopbody.lr.ph.i

loopbody.lr.ph.i:                                 ; preds = %loopbody.lr.ph.i.preheader, %ifno.i
  %num_1.ph6.i = phi i64 [ %num_13.i, %ifno.i ], [ %loop_counter_12, %loopbody.lr.ph.i.preheader ]
  %fac_1.ph5.i = phi i64 [ %fac_2.i, %ifno.i ], [ 2, %loopbody.lr.ph.i.preheader ]
  br label %loopbody.i

loopbody.i:                                       ; preds = %ifyes.i, %loopbody.lr.ph.i
  %num_13.i = phi i64 [ %num_1.ph6.i, %loopbody.lr.ph.i ], [ %quo_0.i, %ifyes.i ]
  %quo_0.i = sdiv i64 %num_13.i, %fac_1.ph5.i
  %tmp_0.i = mul i64 %quo_0.i, %fac_1.ph5.i
  %iszero_0.i = icmp eq i64 %num_13.i, %tmp_0.i
  br i1 %iszero_0.i, label %ifyes.i, label %ifno.i

ifyes.i:                                          ; preds = %loopbody.i
  %0 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %fac_1.ph5.i)
  %1 = tail call i32 @putchar(i32 10)
  %ispos_0.i = icmp sgt i64 %quo_0.i, 1
  br i1 %ispos_0.i, label %loopbody.i, label %__orig_main.exit

ifno.i:                                           ; preds = %loopbody.i
  %fac_2.i = add i64 %fac_1.ph5.i, 1
  br label %loopbody.lr.ph.i

__orig_main.exit:                                 ; preds = %ifyes.i
  %loop_counter_2 = add nuw nsw i64 %loop_counter_12, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 100000
  br i1 %exitcond.not, label %loop_done, label %loopbody.lr.ph.i.preheader

loop_done:                                        ; preds = %__orig_main.exit
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %num) local_unnamed_addr #0 {
pre_entry:
  %ispos_024 = icmp sgt i64 %num, 1
  br i1 %ispos_024, label %loopbody.lr.ph, label %loopend

loopbody.lr.ph:                                   ; preds = %pre_entry, %ifno
  %num_1.ph6 = phi i64 [ %num_13, %ifno ], [ %num, %pre_entry ]
  %fac_1.ph5 = phi i64 [ %fac_2, %ifno ], [ 2, %pre_entry ]
  br label %loopbody

loopbody:                                         ; preds = %loopbody.lr.ph, %ifyes
  %num_13 = phi i64 [ %num_1.ph6, %loopbody.lr.ph ], [ %quo_0, %ifyes ]
  %quo_0 = sdiv i64 %num_13, %fac_1.ph5
  %tmp_0 = mul i64 %quo_0, %fac_1.ph5
  %iszero_0 = icmp eq i64 %num_13, %tmp_0
  br i1 %iszero_0, label %ifyes, label %ifno

ifyes:                                            ; preds = %loopbody
  %0 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %fac_1.ph5)
  %1 = tail call i32 @putchar(i32 10)
  %ispos_0 = icmp sgt i64 %quo_0, 1
  br i1 %ispos_0, label %loopbody, label %loopend

ifno:                                             ; preds = %loopbody
  %fac_2 = add i64 %fac_1.ph5, 1
  br label %loopbody.lr.ph

loopend:                                          ; preds = %ifyes, %pre_entry
  ret void
}

define dso_local noundef i32 @main(i32 %argc, ptr nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %loopbody.lr.ph.i.preheader.i, label %2

2:                                                ; preds = %0
  %3 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.4, i32 0, i32 %1)
  tail call void @exit(i32 2)
  unreachable

loopbody.lr.ph.i.preheader.i:                     ; preds = %0, %__orig_main.exit.i
  %loop_counter_12.i = phi i64 [ %loop_counter_2.i, %__orig_main.exit.i ], [ 10, %0 ]
  br label %loopbody.lr.ph.i.i

loopbody.lr.ph.i.i:                               ; preds = %ifno.i.i, %loopbody.lr.ph.i.preheader.i
  %num_1.ph6.i.i = phi i64 [ %num_13.i.i, %ifno.i.i ], [ %loop_counter_12.i, %loopbody.lr.ph.i.preheader.i ]
  %fac_1.ph5.i.i = phi i64 [ %fac_2.i.i, %ifno.i.i ], [ 2, %loopbody.lr.ph.i.preheader.i ]
  br label %loopbody.i.i

loopbody.i.i:                                     ; preds = %ifyes.i.i, %loopbody.lr.ph.i.i
  %num_13.i.i = phi i64 [ %num_1.ph6.i.i, %loopbody.lr.ph.i.i ], [ %quo_0.i.i, %ifyes.i.i ]
  %quo_0.i.i = sdiv i64 %num_13.i.i, %fac_1.ph5.i.i
  %tmp_0.i.i = mul i64 %quo_0.i.i, %fac_1.ph5.i.i
  %iszero_0.i.i = icmp eq i64 %num_13.i.i, %tmp_0.i.i
  br i1 %iszero_0.i.i, label %ifyes.i.i, label %ifno.i.i

ifyes.i.i:                                        ; preds = %loopbody.i.i
  %4 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %fac_1.ph5.i.i)
  %5 = tail call i32 @putchar(i32 10)
  %ispos_0.i.i = icmp sgt i64 %quo_0.i.i, 1
  br i1 %ispos_0.i.i, label %loopbody.i.i, label %__orig_main.exit.i

ifno.i.i:                                         ; preds = %loopbody.i.i
  %fac_2.i.i = add i64 %fac_1.ph5.i.i, 1
  br label %loopbody.lr.ph.i.i

__orig_main.exit.i:                               ; preds = %ifyes.i.i
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_12.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 100000
  br i1 %exitcond.not.i, label %__main.exit, label %loopbody.lr.ph.i.preheader.i

__main.exit:                                      ; preds = %__orig_main.exit.i
  ret i32 0
}

attributes #0 = { nofree nounwind }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read) }
