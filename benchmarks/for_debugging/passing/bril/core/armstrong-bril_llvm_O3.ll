; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpUnJ9Gy/armstrong-init.ll'
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
  br label %else.i.i.preheader

else.i.i.preheader:                               ; preds = %__orig_main.exit, %b0
  %loop_counter_12 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %__orig_main.exit ]
  br label %else.i.i

else.i.i:                                         ; preds = %else.i.i.preheader, %else.i.i
  %n.tr4.i.i = phi i64 [ %div_0.i.i, %else.i.i ], [ %loop_counter_12, %else.i.i.preheader ]
  %accumulator.tr3.i.i = phi i64 [ %res_0.i.i, %else.i.i ], [ 0, %else.i.i.preheader ]
  %div_0.i.i = sdiv i64 %n.tr4.i.i, 10
  %res_0.i.i = add i64 %accumulator.tr3.i.i, 1
  %n.off.i.i = add nsw i64 %div_0.i.i, 9
  %cond_0.i.i = icmp ult i64 %n.off.i.i, 19
  br i1 %cond_0.i.i, label %__getDigits.exit.i, label %else.i.i

__getDigits.exit.i:                               ; preds = %else.i.i
  %0 = add i64 %accumulator.tr3.i.i, 2
  %b_01.i.i = icmp eq i64 %0, 0
  br i1 %b_01.i.i, label %body.us.i, label %body.i.preheader

body.i.preheader:                                 ; preds = %__getDigits.exit.i
  %min.iters.check = icmp eq i64 %0, 1
  %n.vec = and i64 %0, -2
  %ind.end = and i64 %accumulator.tr3.i.i, 1
  %cmp.n = icmp eq i64 %0, %n.vec
  br label %body.i

body.us.i:                                        ; preds = %__getDigits.exit.i, %body.us.i
  %sum_13.us.i = phi i64 [ %sum_2.us.i, %body.us.i ], [ 0, %__getDigits.exit.i ]
  %tmp_12.us.i = phi i64 [ %tmp_2.us.i, %body.us.i ], [ %loop_counter_12, %__getDigits.exit.i ]
  %sum_2.us.i = add i64 %sum_13.us.i, 1
  %tmp_2.us.i = udiv i64 %tmp_12.us.i, 10
  %b_0.us.not.i = icmp ult i64 %tmp_12.us.i, 10
  br i1 %b_0.us.not.i, label %__orig_main.exit, label %body.us.i

body.i:                                           ; preds = %body.i.preheader, %__power.exit.loopexit.i
  %sum_13.i = phi i64 [ %sum_2.i, %__power.exit.loopexit.i ], [ 0, %body.i.preheader ]
  %tmp_12.i = phi i64 [ %tmp_2.i, %__power.exit.loopexit.i ], [ %loop_counter_12, %body.i.preheader ]
  %tmp_12.i.frozen = freeze i64 %tmp_12.i
  %tmp_2.i = udiv i64 %tmp_12.i.frozen, 10
  %1 = mul i64 %tmp_2.i, 10
  %.decomposed = sub i64 %tmp_12.i.frozen, %1
  br i1 %min.iters.check, label %body.i.i.preheader, label %vector.body

vector.body:                                      ; preds = %body.i, %vector.body
  %index = phi i64 [ %index.next, %vector.body ], [ 0, %body.i ]
  %vec.phi = phi i64 [ %2, %vector.body ], [ 1, %body.i ]
  %vec.phi7 = phi i64 [ %3, %vector.body ], [ 1, %body.i ]
  %2 = mul i64 %vec.phi, %.decomposed
  %3 = mul i64 %vec.phi7, %.decomposed
  %index.next = add nuw i64 %index, 2
  %4 = icmp eq i64 %index.next, %n.vec
  br i1 %4, label %middle.block, label %vector.body, !llvm.loop !0

middle.block:                                     ; preds = %vector.body
  %bin.rdx = mul i64 %3, %2
  br i1 %cmp.n, label %__power.exit.loopexit.i, label %body.i.i.preheader

body.i.i.preheader:                               ; preds = %body.i, %middle.block
  %exp_13.i.i.ph = phi i64 [ %0, %body.i ], [ %ind.end, %middle.block ]
  %res_12.i.i.ph = phi i64 [ 1, %body.i ], [ %bin.rdx, %middle.block ]
  br label %body.i.i

body.i.i:                                         ; preds = %body.i.i.preheader, %body.i.i
  %exp_13.i.i = phi i64 [ %exp_2.i.i, %body.i.i ], [ %exp_13.i.i.ph, %body.i.i.preheader ]
  %res_12.i.i = phi i64 [ %res_2.i.i, %body.i.i ], [ %res_12.i.i.ph, %body.i.i.preheader ]
  %res_2.i.i = mul i64 %res_12.i.i, %.decomposed
  %exp_2.i.i = add i64 %exp_13.i.i, -1
  %b_0.i.i = icmp eq i64 %exp_2.i.i, 0
  br i1 %b_0.i.i, label %__power.exit.loopexit.i, label %body.i.i, !llvm.loop !3

__power.exit.loopexit.i:                          ; preds = %body.i.i, %middle.block
  %res_2.i.i.lcssa = phi i64 [ %bin.rdx, %middle.block ], [ %res_2.i.i, %body.i.i ]
  %sum_2.i = add i64 %res_2.i.i.lcssa, %sum_13.i
  %b_0.not.i = icmp ult i64 %tmp_12.i, 10
  br i1 %b_0.not.i, label %__orig_main.exit, label %body.i

__orig_main.exit:                                 ; preds = %__power.exit.loopexit.i, %body.us.i
  %sum_1.lcssa.i = phi i64 [ %sum_2.us.i, %body.us.i ], [ %sum_2.i, %__power.exit.loopexit.i ]
  %res_0.i = icmp eq i64 %sum_1.lcssa.i, %loop_counter_12
  %.str..str.1.i.i = select i1 %res_0.i, ptr @.str, ptr @.str.1
  %5 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) %.str..str.1.i.i)
  %6 = tail call i32 @putchar(i32 10)
  %loop_counter_2 = add nuw nsw i64 %loop_counter_12, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 1000000
  br i1 %exitcond.not, label %loop_done, label %else.i.i.preheader

loop_done:                                        ; preds = %__orig_main.exit
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %input) local_unnamed_addr #0 {
pre_entry:
  %input.fr = freeze i64 %input
  %n.off1.i = add i64 %input.fr, 9
  %cond_02.i = icmp ult i64 %n.off1.i, 19
  br i1 %cond_02.i, label %__getDigits.exit.thread, label %else.i

else.i:                                           ; preds = %pre_entry, %else.i
  %n.tr4.i = phi i64 [ %div_0.i, %else.i ], [ %input.fr, %pre_entry ]
  %accumulator.tr3.i = phi i64 [ %res_0.i, %else.i ], [ 0, %pre_entry ]
  %div_0.i = sdiv i64 %n.tr4.i, 10
  %res_0.i = add i64 %accumulator.tr3.i, 1
  %n.off.i = add nsw i64 %div_0.i, 9
  %cond_0.i = icmp ult i64 %n.off.i, 19
  br i1 %cond_0.i, label %__getDigits.exit, label %else.i

__getDigits.exit:                                 ; preds = %else.i
  %b_01 = icmp sgt i64 %input.fr, 0
  br i1 %b_01, label %body.lr.ph, label %done

__getDigits.exit.thread:                          ; preds = %pre_entry
  %b_016 = icmp sgt i64 %input.fr, 0
  br i1 %b_016, label %body.preheader, label %done

body.lr.ph:                                       ; preds = %__getDigits.exit
  %0 = add i64 %accumulator.tr3.i, 2
  %b_01.i = icmp eq i64 %0, 0
  br i1 %b_01.i, label %body.us, label %body.preheader

body.preheader:                                   ; preds = %__getDigits.exit.thread, %body.lr.ph
  %accumulator.tr.lcssa.i710 = phi i64 [ %0, %body.lr.ph ], [ 1, %__getDigits.exit.thread ]
  %min.iters.check = icmp ult i64 %accumulator.tr.lcssa.i710, 2
  %n.vec = and i64 %accumulator.tr.lcssa.i710, -2
  %ind.end = and i64 %accumulator.tr.lcssa.i710, 1
  %cmp.n = icmp eq i64 %accumulator.tr.lcssa.i710, %n.vec
  br label %body

body.us:                                          ; preds = %body.lr.ph, %body.us
  %sum_13.us = phi i64 [ %sum_2.us, %body.us ], [ 0, %body.lr.ph ]
  %tmp_12.us = phi i64 [ %tmp_2.us, %body.us ], [ %input.fr, %body.lr.ph ]
  %sum_2.us = add i64 %sum_13.us, 1
  %tmp_2.us = udiv i64 %tmp_12.us, 10
  %b_0.us.not = icmp ult i64 %tmp_12.us, 10
  br i1 %b_0.us.not, label %done, label %body.us

body:                                             ; preds = %body.preheader, %__power.exit.loopexit
  %sum_13 = phi i64 [ %sum_2, %__power.exit.loopexit ], [ 0, %body.preheader ]
  %tmp_12 = phi i64 [ %tmp_2, %__power.exit.loopexit ], [ %input.fr, %body.preheader ]
  %tmp_12.frozen = freeze i64 %tmp_12
  %tmp_2 = udiv i64 %tmp_12.frozen, 10
  %1 = mul i64 %tmp_2, 10
  %.decomposed = sub i64 %tmp_12.frozen, %1
  br i1 %min.iters.check, label %body.i.preheader, label %vector.body

vector.body:                                      ; preds = %body, %vector.body
  %index = phi i64 [ %index.next, %vector.body ], [ 0, %body ]
  %vec.phi = phi i64 [ %2, %vector.body ], [ 1, %body ]
  %vec.phi13 = phi i64 [ %3, %vector.body ], [ 1, %body ]
  %2 = mul i64 %vec.phi, %.decomposed
  %3 = mul i64 %vec.phi13, %.decomposed
  %index.next = add nuw i64 %index, 2
  %4 = icmp eq i64 %index.next, %n.vec
  br i1 %4, label %middle.block, label %vector.body, !llvm.loop !4

middle.block:                                     ; preds = %vector.body
  %bin.rdx = mul i64 %3, %2
  br i1 %cmp.n, label %__power.exit.loopexit, label %body.i.preheader

body.i.preheader:                                 ; preds = %body, %middle.block
  %exp_13.i.ph = phi i64 [ %accumulator.tr.lcssa.i710, %body ], [ %ind.end, %middle.block ]
  %res_12.i.ph = phi i64 [ 1, %body ], [ %bin.rdx, %middle.block ]
  br label %body.i

body.i:                                           ; preds = %body.i.preheader, %body.i
  %exp_13.i = phi i64 [ %exp_2.i, %body.i ], [ %exp_13.i.ph, %body.i.preheader ]
  %res_12.i = phi i64 [ %res_2.i, %body.i ], [ %res_12.i.ph, %body.i.preheader ]
  %res_2.i = mul i64 %res_12.i, %.decomposed
  %exp_2.i = add i64 %exp_13.i, -1
  %b_0.i = icmp eq i64 %exp_2.i, 0
  br i1 %b_0.i, label %__power.exit.loopexit, label %body.i, !llvm.loop !5

__power.exit.loopexit:                            ; preds = %body.i, %middle.block
  %res_2.i.lcssa = phi i64 [ %bin.rdx, %middle.block ], [ %res_2.i, %body.i ]
  %sum_2 = add i64 %res_2.i.lcssa, %sum_13
  %b_0.not = icmp ult i64 %tmp_12, 10
  br i1 %b_0.not, label %done, label %body

done:                                             ; preds = %body.us, %__power.exit.loopexit, %__getDigits.exit.thread, %__getDigits.exit
  %sum_1.lcssa = phi i64 [ 0, %__getDigits.exit ], [ 0, %__getDigits.exit.thread ], [ %sum_2, %__power.exit.loopexit ], [ %sum_2.us, %body.us ]
  %res_0 = icmp eq i64 %sum_1.lcssa, %input.fr
  %.str..str.1.i = select i1 %res_0, ptr @.str, ptr @.str.1
  %5 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) %.str..str.1.i)
  %6 = tail call i32 @putchar(i32 10)
  ret void
}

; Function Attrs: nofree norecurse nosync nounwind memory(none)
define dso_local i64 @__getDigits(i64 %n) local_unnamed_addr #2 {
pre_entry:
  %n.off1 = add i64 %n, 9
  %cond_02 = icmp ult i64 %n.off1, 19
  br i1 %cond_02, label %common.ret, label %else

common.ret.loopexit:                              ; preds = %else
  %0 = add i64 %accumulator.tr3, 2
  br label %common.ret

common.ret:                                       ; preds = %common.ret.loopexit, %pre_entry
  %accumulator.tr.lcssa = phi i64 [ 1, %pre_entry ], [ %0, %common.ret.loopexit ]
  ret i64 %accumulator.tr.lcssa

else:                                             ; preds = %pre_entry, %else
  %n.tr4 = phi i64 [ %div_0, %else ], [ %n, %pre_entry ]
  %accumulator.tr3 = phi i64 [ %res_0, %else ], [ 0, %pre_entry ]
  %div_0 = sdiv i64 %n.tr4, 10
  %res_0 = add i64 %accumulator.tr3, 1
  %n.off = add nsw i64 %div_0, 9
  %cond_0 = icmp ult i64 %n.off, 19
  br i1 %cond_0, label %common.ret.loopexit, label %else
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define dso_local i64 @__mod(i64 %a, i64 %b) local_unnamed_addr #3 {
pre_entry:
  %a.fr = freeze i64 %a
  %0 = srem i64 %a.fr, %b
  ret i64 %0
}

; Function Attrs: nofree norecurse nosync nounwind memory(none)
define dso_local i64 @__power(i64 %base, i64 %exp) local_unnamed_addr #2 {
pre_entry:
  %b_01 = icmp eq i64 %exp, 0
  br i1 %b_01, label %done, label %body.preheader

body.preheader:                                   ; preds = %pre_entry
  %min.iters.check = icmp ult i64 %exp, 4
  br i1 %min.iters.check, label %body.preheader9, label %vector.ph

vector.ph:                                        ; preds = %body.preheader
  %n.vec = and i64 %exp, -4
  %ind.end = and i64 %exp, 3
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 1, %vector.ph ], [ %0, %vector.body ]
  %vec.phi4 = phi i64 [ 1, %vector.ph ], [ %1, %vector.body ]
  %vec.phi5 = phi i64 [ 1, %vector.ph ], [ %2, %vector.body ]
  %vec.phi6 = phi i64 [ 1, %vector.ph ], [ %3, %vector.body ]
  %0 = mul i64 %vec.phi, %base
  %1 = mul i64 %vec.phi4, %base
  %2 = mul i64 %vec.phi5, %base
  %3 = mul i64 %vec.phi6, %base
  %index.next = add nuw i64 %index, 4
  %4 = icmp eq i64 %index.next, %n.vec
  br i1 %4, label %middle.block, label %vector.body, !llvm.loop !6

middle.block:                                     ; preds = %vector.body
  %bin.rdx = mul i64 %1, %0
  %bin.rdx7 = mul i64 %2, %bin.rdx
  %bin.rdx8 = mul i64 %3, %bin.rdx7
  %cmp.n = icmp eq i64 %n.vec, %exp
  br i1 %cmp.n, label %done, label %body.preheader9

body.preheader9:                                  ; preds = %body.preheader, %middle.block
  %exp_13.ph = phi i64 [ %exp, %body.preheader ], [ %ind.end, %middle.block ]
  %res_12.ph = phi i64 [ 1, %body.preheader ], [ %bin.rdx8, %middle.block ]
  br label %body

body:                                             ; preds = %body.preheader9, %body
  %exp_13 = phi i64 [ %exp_2, %body ], [ %exp_13.ph, %body.preheader9 ]
  %res_12 = phi i64 [ %res_2, %body ], [ %res_12.ph, %body.preheader9 ]
  %res_2 = mul i64 %res_12, %base
  %exp_2 = add i64 %exp_13, -1
  %b_0 = icmp eq i64 %exp_2, 0
  br i1 %b_0, label %done, label %body, !llvm.loop !7

done:                                             ; preds = %body, %middle.block, %pre_entry
  %res_1.lcssa = phi i64 [ 1, %pre_entry ], [ %bin.rdx8, %middle.block ], [ %res_2, %body ]
  ret i64 %res_1.lcssa
}

define dso_local noundef i32 @main(i32 %argc, ptr nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %else.i.i.preheader.i, label %2

2:                                                ; preds = %0
  %3 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.4, i32 0, i32 %1)
  tail call void @exit(i32 2)
  unreachable

else.i.i.preheader.i:                             ; preds = %0, %__orig_main.exit.i
  %loop_counter_12.i = phi i64 [ %loop_counter_2.i, %__orig_main.exit.i ], [ 10, %0 ]
  br label %else.i.i.i

else.i.i.i:                                       ; preds = %else.i.i.i, %else.i.i.preheader.i
  %n.tr4.i.i.i = phi i64 [ %div_0.i.i.i, %else.i.i.i ], [ %loop_counter_12.i, %else.i.i.preheader.i ]
  %accumulator.tr3.i.i.i = phi i64 [ %res_0.i.i.i, %else.i.i.i ], [ 0, %else.i.i.preheader.i ]
  %div_0.i.i.i = sdiv i64 %n.tr4.i.i.i, 10
  %res_0.i.i.i = add i64 %accumulator.tr3.i.i.i, 1
  %n.off.i.i.i = add nsw i64 %div_0.i.i.i, 9
  %cond_0.i.i.i = icmp ult i64 %n.off.i.i.i, 19
  br i1 %cond_0.i.i.i, label %__getDigits.exit.i.i, label %else.i.i.i

__getDigits.exit.i.i:                             ; preds = %else.i.i.i
  %4 = add i64 %accumulator.tr3.i.i.i, 2
  %b_01.i.i.i = icmp eq i64 %4, 0
  br i1 %b_01.i.i.i, label %body.us.i.i, label %body.i.i.preheader

body.i.i.preheader:                               ; preds = %__getDigits.exit.i.i
  %min.iters.check = icmp eq i64 %4, 1
  %n.vec = and i64 %4, -2
  %ind.end = and i64 %accumulator.tr3.i.i.i, 1
  %cmp.n = icmp eq i64 %4, %n.vec
  br label %body.i.i

body.us.i.i:                                      ; preds = %__getDigits.exit.i.i, %body.us.i.i
  %sum_13.us.i.i = phi i64 [ %sum_2.us.i.i, %body.us.i.i ], [ 0, %__getDigits.exit.i.i ]
  %tmp_12.us.i.i = phi i64 [ %tmp_2.us.i.i, %body.us.i.i ], [ %loop_counter_12.i, %__getDigits.exit.i.i ]
  %sum_2.us.i.i = add i64 %sum_13.us.i.i, 1
  %tmp_2.us.i.i = udiv i64 %tmp_12.us.i.i, 10
  %b_0.us.not.i.i = icmp ult i64 %tmp_12.us.i.i, 10
  br i1 %b_0.us.not.i.i, label %__orig_main.exit.i, label %body.us.i.i

body.i.i:                                         ; preds = %body.i.i.preheader, %__power.exit.loopexit.i.i
  %sum_13.i.i = phi i64 [ %sum_2.i.i, %__power.exit.loopexit.i.i ], [ 0, %body.i.i.preheader ]
  %tmp_12.i.i = phi i64 [ %tmp_2.i.i, %__power.exit.loopexit.i.i ], [ %loop_counter_12.i, %body.i.i.preheader ]
  %tmp_12.i.i.frozen = freeze i64 %tmp_12.i.i
  %tmp_2.i.i = udiv i64 %tmp_12.i.i.frozen, 10
  %5 = mul i64 %tmp_2.i.i, 10
  %.decomposed = sub i64 %tmp_12.i.i.frozen, %5
  br i1 %min.iters.check, label %body.i.i.i.preheader, label %vector.body

vector.body:                                      ; preds = %body.i.i, %vector.body
  %index = phi i64 [ %index.next, %vector.body ], [ 0, %body.i.i ]
  %vec.phi = phi i64 [ %6, %vector.body ], [ 1, %body.i.i ]
  %vec.phi6 = phi i64 [ %7, %vector.body ], [ 1, %body.i.i ]
  %6 = mul i64 %vec.phi, %.decomposed
  %7 = mul i64 %vec.phi6, %.decomposed
  %index.next = add nuw i64 %index, 2
  %8 = icmp eq i64 %index.next, %n.vec
  br i1 %8, label %middle.block, label %vector.body, !llvm.loop !8

middle.block:                                     ; preds = %vector.body
  %bin.rdx = mul i64 %7, %6
  br i1 %cmp.n, label %__power.exit.loopexit.i.i, label %body.i.i.i.preheader

body.i.i.i.preheader:                             ; preds = %body.i.i, %middle.block
  %exp_13.i.i.i.ph = phi i64 [ %4, %body.i.i ], [ %ind.end, %middle.block ]
  %res_12.i.i.i.ph = phi i64 [ 1, %body.i.i ], [ %bin.rdx, %middle.block ]
  br label %body.i.i.i

body.i.i.i:                                       ; preds = %body.i.i.i.preheader, %body.i.i.i
  %exp_13.i.i.i = phi i64 [ %exp_2.i.i.i, %body.i.i.i ], [ %exp_13.i.i.i.ph, %body.i.i.i.preheader ]
  %res_12.i.i.i = phi i64 [ %res_2.i.i.i, %body.i.i.i ], [ %res_12.i.i.i.ph, %body.i.i.i.preheader ]
  %res_2.i.i.i = mul i64 %res_12.i.i.i, %.decomposed
  %exp_2.i.i.i = add i64 %exp_13.i.i.i, -1
  %b_0.i.i.i = icmp eq i64 %exp_2.i.i.i, 0
  br i1 %b_0.i.i.i, label %__power.exit.loopexit.i.i, label %body.i.i.i, !llvm.loop !9

__power.exit.loopexit.i.i:                        ; preds = %body.i.i.i, %middle.block
  %res_2.i.i.i.lcssa = phi i64 [ %bin.rdx, %middle.block ], [ %res_2.i.i.i, %body.i.i.i ]
  %sum_2.i.i = add i64 %res_2.i.i.i.lcssa, %sum_13.i.i
  %b_0.not.i.i = icmp ult i64 %tmp_12.i.i, 10
  br i1 %b_0.not.i.i, label %__orig_main.exit.i, label %body.i.i

__orig_main.exit.i:                               ; preds = %__power.exit.loopexit.i.i, %body.us.i.i
  %sum_1.lcssa.i.i = phi i64 [ %sum_2.us.i.i, %body.us.i.i ], [ %sum_2.i.i, %__power.exit.loopexit.i.i ]
  %res_0.i.i = icmp eq i64 %sum_1.lcssa.i.i, %loop_counter_12.i
  %.str..str.1.i.i.i = select i1 %res_0.i.i, ptr @.str, ptr @.str.1
  %9 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) %.str..str.1.i.i.i)
  %10 = tail call i32 @putchar(i32 10)
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_12.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 1000000
  br i1 %exitcond.not.i, label %__main.exit, label %else.i.i.preheader.i

__main.exit:                                      ; preds = %__orig_main.exit.i
  ret i32 0
}

attributes #0 = { nofree nounwind }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read) }
attributes #2 = { nofree norecurse nosync nounwind memory(none) }
attributes #3 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) }

!0 = distinct !{!0, !1, !2}
!1 = !{!"llvm.loop.isvectorized", i32 1}
!2 = !{!"llvm.loop.unroll.runtime.disable"}
!3 = distinct !{!3, !1}
!4 = distinct !{!4, !1, !2}
!5 = distinct !{!5, !1}
!6 = distinct !{!6, !1, !2}
!7 = distinct !{!7, !1}
!8 = distinct !{!8, !1, !2}
!9 = distinct !{!9, !1}
