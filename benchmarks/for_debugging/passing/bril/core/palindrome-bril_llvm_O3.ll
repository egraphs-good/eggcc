; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpMM8GIm/palindrome-init.ll'
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
  br label %for.cond.pow.outer.i.preheader.i.preheader

for.cond.pow.outer.i.preheader.i.preheader:       ; preds = %b0, %__orig_main.exit
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %__orig_main.exit ]
  br label %for.cond.pow.outer.i.preheader.i

for.cond.pow.outer.i.preheader.i:                 ; preds = %for.cond.pow.outer.i.preheader.i.preheader, %__pow.exit.i
  %index_16.i = phi i64 [ %spec.select1.i, %__pow.exit.i ], [ 1, %for.cond.pow.outer.i.preheader.i.preheader ]
  switch i64 %index_16.i, label %vector.ph6 [
    i64 0, label %__pow.exit.i
    i64 1, label %if.false.pow.i.i.preheader
  ]

vector.ph6:                                       ; preds = %for.cond.pow.outer.i.preheader.i
  %n.vec8 = and i64 %index_16.i, -2
  %ind.end9 = and i64 %index_16.i, 1
  br label %vector.body12

vector.body12:                                    ; preds = %vector.body12, %vector.ph6
  %index13 = phi i64 [ 0, %vector.ph6 ], [ %index.next16, %vector.body12 ]
  %vec.phi14 = phi i64 [ 1, %vector.ph6 ], [ %0, %vector.body12 ]
  %vec.phi15 = phi i64 [ 1, %vector.ph6 ], [ %1, %vector.body12 ]
  %0 = mul i64 %vec.phi14, 10
  %1 = mul i64 %vec.phi15, 10
  %index.next16 = add nuw i64 %index13, 2
  %2 = icmp eq i64 %index.next16, %n.vec8
  br i1 %2, label %middle.block3, label %vector.body12, !llvm.loop !0

middle.block3:                                    ; preds = %vector.body12
  %bin.rdx17 = mul i64 %1, %0
  %cmp.n11 = icmp eq i64 %index_16.i, %n.vec8
  br i1 %cmp.n11, label %__pow.exit.i, label %if.false.pow.i.i.preheader

if.false.pow.i.i.preheader:                       ; preds = %for.cond.pow.outer.i.preheader.i, %middle.block3
  %exp_1.ph.i5.i.ph = phi i64 [ %index_16.i, %for.cond.pow.outer.i.preheader.i ], [ %ind.end9, %middle.block3 ]
  %res_1.ph.i4.i.ph = phi i64 [ %index_16.i, %for.cond.pow.outer.i.preheader.i ], [ %bin.rdx17, %middle.block3 ]
  br label %if.false.pow.i.i

if.false.pow.i.i:                                 ; preds = %if.false.pow.i.i.preheader, %if.false.pow.i.i
  %exp_1.ph.i5.i = phi i64 [ %exp_2.i.i, %if.false.pow.i.i ], [ %exp_1.ph.i5.i.ph, %if.false.pow.i.i.preheader ]
  %res_1.ph.i4.i = phi i64 [ %res_2.i.i, %if.false.pow.i.i ], [ %res_1.ph.i4.i.ph, %if.false.pow.i.i.preheader ]
  %res_2.i.i = mul i64 %res_1.ph.i4.i, 10
  %exp_2.i.i = add i64 %exp_1.ph.i5.i, -1
  %finished_0.i.i = icmp eq i64 %exp_2.i.i, 0
  br i1 %finished_0.i.i, label %__pow.exit.i, label %if.false.pow.i.i, !llvm.loop !3

__pow.exit.i:                                     ; preds = %if.false.pow.i.i, %middle.block3, %for.cond.pow.outer.i.preheader.i
  %res_1.ph.i.lcssa.i = phi i64 [ 1, %for.cond.pow.outer.i.preheader.i ], [ %bin.rdx17, %middle.block3 ], [ %res_2.i.i, %if.false.pow.i.i ]
  %d_0.i = sdiv i64 %loop_counter_11, %res_1.ph.i.lcssa.i
  %check_0.i = icmp ne i64 %d_0.i, 0
  %index_2.i = zext i1 %check_0.i to i64
  %spec.select1.i = add i64 %index_16.i, %index_2.i
  br i1 %check_0.i, label %for.cond.pow.outer.i.preheader.i, label %for.end.i

for.end.i:                                        ; preds = %__pow.exit.i
  %exp_0.i = add i64 %spec.select1.i, -1
  %check_05.i.i = icmp slt i64 %exp_0.i, 1
  br i1 %check_05.i.i, label %__orig_main.exit, label %for.cond.pow.outer.i.preheader.i.i

for.cond.pow.outer.i.preheader.i.i:               ; preds = %for.end.i, %if.true.mirror.i.i
  %in.fr7.i.i = phi i64 [ %next_in_0.i.i, %if.true.mirror.i.i ], [ %loop_counter_11, %for.end.i ]
  %len.tr6.i.i = phi i64 [ %next_len_0.i.i, %if.true.mirror.i.i ], [ %exp_0.i, %for.end.i ]
  %min.iters.check = icmp ult i64 %len.tr6.i.i, 2
  br i1 %min.iters.check, label %if.false.pow.i.i.i.preheader, label %vector.ph

vector.ph:                                        ; preds = %for.cond.pow.outer.i.preheader.i.i
  %n.vec = and i64 %len.tr6.i.i, -2
  %ind.end = and i64 %len.tr6.i.i, 1
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 1, %vector.ph ], [ %3, %vector.body ]
  %vec.phi2 = phi i64 [ 1, %vector.ph ], [ %4, %vector.body ]
  %3 = mul i64 %vec.phi, 10
  %4 = mul i64 %vec.phi2, 10
  %index.next = add nuw i64 %index, 2
  %5 = icmp eq i64 %index.next, %n.vec
  br i1 %5, label %middle.block, label %vector.body, !llvm.loop !4

middle.block:                                     ; preds = %vector.body
  %bin.rdx = mul i64 %4, %3
  %cmp.n = icmp eq i64 %len.tr6.i.i, %n.vec
  br i1 %cmp.n, label %__pow.exit.i.i, label %if.false.pow.i.i.i.preheader

if.false.pow.i.i.i.preheader:                     ; preds = %for.cond.pow.outer.i.preheader.i.i, %middle.block
  %exp_1.ph.i3.i.i.ph = phi i64 [ %len.tr6.i.i, %for.cond.pow.outer.i.preheader.i.i ], [ %ind.end, %middle.block ]
  %res_1.ph.i2.i.i.ph = phi i64 [ 1, %for.cond.pow.outer.i.preheader.i.i ], [ %bin.rdx, %middle.block ]
  br label %if.false.pow.i.i.i

if.false.pow.i.i.i:                               ; preds = %if.false.pow.i.i.i.preheader, %if.false.pow.i.i.i
  %exp_1.ph.i3.i.i = phi i64 [ %exp_2.i.i.i, %if.false.pow.i.i.i ], [ %exp_1.ph.i3.i.i.ph, %if.false.pow.i.i.i.preheader ]
  %res_1.ph.i2.i.i = phi i64 [ %res_2.i.i.i, %if.false.pow.i.i.i ], [ %res_1.ph.i2.i.i.ph, %if.false.pow.i.i.i.preheader ]
  %res_2.i.i.i = mul i64 %res_1.ph.i2.i.i, 10
  %exp_2.i.i.i = add i64 %exp_1.ph.i3.i.i, -1
  %finished_0.i.i.i = icmp eq i64 %exp_2.i.i.i, 0
  br i1 %finished_0.i.i.i, label %__pow.exit.i.i, label %if.false.pow.i.i.i, !llvm.loop !5

__pow.exit.i.i:                                   ; preds = %if.false.pow.i.i.i, %middle.block
  %res_2.i.i.i.lcssa = phi i64 [ %bin.rdx, %middle.block ], [ %res_2.i.i.i, %if.false.pow.i.i.i ]
  %left_0.i.i = sdiv i64 %in.fr7.i.i, %res_2.i.i.i.lcssa
  %6 = srem i64 %in.fr7.i.i, 10
  %is_equal_0.i.i = icmp eq i64 %left_0.i.i, %6
  br i1 %is_equal_0.i.i, label %if.true.mirror.i.i, label %__orig_main.exit

if.true.mirror.i.i:                               ; preds = %__pow.exit.i.i
  %temp_09.neg.i.i = xor i64 %res_2.i.i.i.lcssa, -1
  %.neg.i.i = mul i64 %left_0.i.i, %temp_09.neg.i.i
  %.fr.neg.i.i = freeze i64 %.neg.i.i
  %temp_2.i.i = add i64 %.fr.neg.i.i, %in.fr7.i.i
  %next_in_0.i.i = sdiv i64 %temp_2.i.i, 10
  %next_len_0.i.i = add i64 %len.tr6.i.i, -2
  %check_0.i.i = icmp slt i64 %len.tr6.i.i, 3
  br i1 %check_0.i.i, label %__orig_main.exit, label %for.cond.pow.outer.i.preheader.i.i

__orig_main.exit:                                 ; preds = %__pow.exit.i.i, %if.true.mirror.i.i, %for.end.i
  %7 = phi ptr [ @.str, %for.end.i ], [ @.str.1, %__pow.exit.i.i ], [ @.str, %if.true.mirror.i.i ]
  %8 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) %7)
  %9 = tail call i32 @putchar(i32 10)
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 1000000
  br i1 %exitcond.not, label %loop_done, label %for.cond.pow.outer.i.preheader.i.preheader

loop_done:                                        ; preds = %__orig_main.exit
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %in) local_unnamed_addr #0 {
pre_entry:
  %in.fr4.i = freeze i64 %in
  br label %for.cond.pow.outer.i.preheader

for.cond.pow.outer.i.preheader:                   ; preds = %pre_entry, %__pow.exit
  %index_16 = phi i64 [ 1, %pre_entry ], [ %spec.select1, %__pow.exit ]
  switch i64 %index_16, label %vector.ph [
    i64 0, label %__pow.exit
    i64 1, label %if.false.pow.i.preheader
  ]

vector.ph:                                        ; preds = %for.cond.pow.outer.i.preheader
  %n.vec = and i64 %index_16, -2
  %ind.end = and i64 %index_16, 1
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 1, %vector.ph ], [ %0, %vector.body ]
  %vec.phi7 = phi i64 [ 1, %vector.ph ], [ %1, %vector.body ]
  %0 = mul i64 %vec.phi, 10
  %1 = mul i64 %vec.phi7, 10
  %index.next = add nuw i64 %index, 2
  %2 = icmp eq i64 %index.next, %n.vec
  br i1 %2, label %middle.block, label %vector.body, !llvm.loop !6

middle.block:                                     ; preds = %vector.body
  %bin.rdx = mul i64 %1, %0
  %cmp.n = icmp eq i64 %index_16, %n.vec
  br i1 %cmp.n, label %__pow.exit, label %if.false.pow.i.preheader

if.false.pow.i.preheader:                         ; preds = %for.cond.pow.outer.i.preheader, %middle.block
  %exp_1.ph.i5.ph = phi i64 [ %index_16, %for.cond.pow.outer.i.preheader ], [ %ind.end, %middle.block ]
  %res_1.ph.i4.ph = phi i64 [ %index_16, %for.cond.pow.outer.i.preheader ], [ %bin.rdx, %middle.block ]
  br label %if.false.pow.i

if.false.pow.i:                                   ; preds = %if.false.pow.i.preheader, %if.false.pow.i
  %exp_1.ph.i5 = phi i64 [ %exp_2.i, %if.false.pow.i ], [ %exp_1.ph.i5.ph, %if.false.pow.i.preheader ]
  %res_1.ph.i4 = phi i64 [ %res_2.i, %if.false.pow.i ], [ %res_1.ph.i4.ph, %if.false.pow.i.preheader ]
  %res_2.i = mul i64 %res_1.ph.i4, 10
  %exp_2.i = add i64 %exp_1.ph.i5, -1
  %finished_0.i = icmp eq i64 %exp_2.i, 0
  br i1 %finished_0.i, label %__pow.exit, label %if.false.pow.i, !llvm.loop !7

__pow.exit:                                       ; preds = %if.false.pow.i, %middle.block, %for.cond.pow.outer.i.preheader
  %res_1.ph.i.lcssa = phi i64 [ 1, %for.cond.pow.outer.i.preheader ], [ %bin.rdx, %middle.block ], [ %res_2.i, %if.false.pow.i ]
  %d_0 = sdiv i64 %in.fr4.i, %res_1.ph.i.lcssa
  %check_0 = icmp ne i64 %d_0, 0
  %index_2 = zext i1 %check_0 to i64
  %spec.select1 = add i64 %index_16, %index_2
  br i1 %check_0, label %for.cond.pow.outer.i.preheader, label %for.end

for.end:                                          ; preds = %__pow.exit
  %exp_0 = add i64 %spec.select1, -1
  %check_05.i = icmp slt i64 %exp_0, 1
  br i1 %check_05.i, label %__palindrome.exit, label %for.cond.pow.outer.i.preheader.i

for.cond.pow.outer.i.preheader.i:                 ; preds = %for.end, %if.true.mirror.i
  %in.fr7.i = phi i64 [ %next_in_0.i, %if.true.mirror.i ], [ %in.fr4.i, %for.end ]
  %len.tr6.i = phi i64 [ %next_len_0.i, %if.true.mirror.i ], [ %exp_0, %for.end ]
  %min.iters.check10 = icmp ult i64 %len.tr6.i, 2
  br i1 %min.iters.check10, label %if.false.pow.i.i.preheader, label %vector.ph11

vector.ph11:                                      ; preds = %for.cond.pow.outer.i.preheader.i
  %n.vec13 = and i64 %len.tr6.i, -2
  %ind.end14 = and i64 %len.tr6.i, 1
  br label %vector.body17

vector.body17:                                    ; preds = %vector.body17, %vector.ph11
  %index18 = phi i64 [ 0, %vector.ph11 ], [ %index.next21, %vector.body17 ]
  %vec.phi19 = phi i64 [ 1, %vector.ph11 ], [ %3, %vector.body17 ]
  %vec.phi20 = phi i64 [ 1, %vector.ph11 ], [ %4, %vector.body17 ]
  %3 = mul i64 %vec.phi19, 10
  %4 = mul i64 %vec.phi20, 10
  %index.next21 = add nuw i64 %index18, 2
  %5 = icmp eq i64 %index.next21, %n.vec13
  br i1 %5, label %middle.block8, label %vector.body17, !llvm.loop !8

middle.block8:                                    ; preds = %vector.body17
  %bin.rdx22 = mul i64 %4, %3
  %cmp.n16 = icmp eq i64 %len.tr6.i, %n.vec13
  br i1 %cmp.n16, label %__pow.exit.i, label %if.false.pow.i.i.preheader

if.false.pow.i.i.preheader:                       ; preds = %for.cond.pow.outer.i.preheader.i, %middle.block8
  %exp_1.ph.i3.i.ph = phi i64 [ %len.tr6.i, %for.cond.pow.outer.i.preheader.i ], [ %ind.end14, %middle.block8 ]
  %res_1.ph.i2.i.ph = phi i64 [ 1, %for.cond.pow.outer.i.preheader.i ], [ %bin.rdx22, %middle.block8 ]
  br label %if.false.pow.i.i

if.false.pow.i.i:                                 ; preds = %if.false.pow.i.i.preheader, %if.false.pow.i.i
  %exp_1.ph.i3.i = phi i64 [ %exp_2.i.i, %if.false.pow.i.i ], [ %exp_1.ph.i3.i.ph, %if.false.pow.i.i.preheader ]
  %res_1.ph.i2.i = phi i64 [ %res_2.i.i, %if.false.pow.i.i ], [ %res_1.ph.i2.i.ph, %if.false.pow.i.i.preheader ]
  %res_2.i.i = mul i64 %res_1.ph.i2.i, 10
  %exp_2.i.i = add i64 %exp_1.ph.i3.i, -1
  %finished_0.i.i = icmp eq i64 %exp_2.i.i, 0
  br i1 %finished_0.i.i, label %__pow.exit.i, label %if.false.pow.i.i, !llvm.loop !9

__pow.exit.i:                                     ; preds = %if.false.pow.i.i, %middle.block8
  %res_2.i.i.lcssa = phi i64 [ %bin.rdx22, %middle.block8 ], [ %res_2.i.i, %if.false.pow.i.i ]
  %left_0.i = sdiv i64 %in.fr7.i, %res_2.i.i.lcssa
  %6 = srem i64 %in.fr7.i, 10
  %is_equal_0.i = icmp eq i64 %left_0.i, %6
  br i1 %is_equal_0.i, label %if.true.mirror.i, label %__palindrome.exit

if.true.mirror.i:                                 ; preds = %__pow.exit.i
  %temp_09.neg.i = xor i64 %res_2.i.i.lcssa, -1
  %.neg.i = mul i64 %left_0.i, %temp_09.neg.i
  %.fr.neg.i = freeze i64 %.neg.i
  %temp_2.i = add i64 %.fr.neg.i, %in.fr7.i
  %next_in_0.i = sdiv i64 %temp_2.i, 10
  %next_len_0.i = add i64 %len.tr6.i, -2
  %check_0.i = icmp slt i64 %len.tr6.i, 3
  br i1 %check_0.i, label %__palindrome.exit, label %for.cond.pow.outer.i.preheader.i

__palindrome.exit:                                ; preds = %if.true.mirror.i, %__pow.exit.i, %for.end
  %7 = phi ptr [ @.str, %for.end ], [ @.str, %if.true.mirror.i ], [ @.str.1, %__pow.exit.i ]
  %8 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) %7)
  %9 = tail call i32 @putchar(i32 10)
  ret void
}

; Function Attrs: nofree norecurse nosync nounwind memory(none)
define dso_local i64 @__pow(i64 %base, i64 %exp) local_unnamed_addr #2 {
pre_entry:
  %finished_03 = icmp eq i64 %exp, 0
  br i1 %finished_03, label %for.end.pow, label %if.false.pow.lr.ph

if.false.pow.lr.ph:                               ; preds = %pre_entry
  %min.iters.check = icmp ult i64 %exp, 4
  br i1 %min.iters.check, label %if.false.pow.preheader, label %vector.ph

vector.ph:                                        ; preds = %if.false.pow.lr.ph
  %n.vec = and i64 %exp, -4
  %ind.end = and i64 %exp, 3
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 1, %vector.ph ], [ %0, %vector.body ]
  %vec.phi6 = phi i64 [ 1, %vector.ph ], [ %1, %vector.body ]
  %vec.phi7 = phi i64 [ 1, %vector.ph ], [ %2, %vector.body ]
  %vec.phi8 = phi i64 [ 1, %vector.ph ], [ %3, %vector.body ]
  %0 = mul i64 %vec.phi, %base
  %1 = mul i64 %vec.phi6, %base
  %2 = mul i64 %vec.phi7, %base
  %3 = mul i64 %vec.phi8, %base
  %index.next = add nuw i64 %index, 4
  %4 = icmp eq i64 %index.next, %n.vec
  br i1 %4, label %middle.block, label %vector.body, !llvm.loop !10

middle.block:                                     ; preds = %vector.body
  %bin.rdx = mul i64 %1, %0
  %bin.rdx9 = mul i64 %2, %bin.rdx
  %bin.rdx10 = mul i64 %3, %bin.rdx9
  %cmp.n = icmp eq i64 %n.vec, %exp
  br i1 %cmp.n, label %for.end.pow, label %if.false.pow.preheader

if.false.pow.preheader:                           ; preds = %if.false.pow.lr.ph, %middle.block
  %exp_1.ph5.ph = phi i64 [ %exp, %if.false.pow.lr.ph ], [ %ind.end, %middle.block ]
  %res_1.ph4.ph = phi i64 [ 1, %if.false.pow.lr.ph ], [ %bin.rdx10, %middle.block ]
  br label %if.false.pow

if.false.pow:                                     ; preds = %if.false.pow.preheader, %if.false.pow
  %exp_1.ph5 = phi i64 [ %exp_2, %if.false.pow ], [ %exp_1.ph5.ph, %if.false.pow.preheader ]
  %res_1.ph4 = phi i64 [ %res_2, %if.false.pow ], [ %res_1.ph4.ph, %if.false.pow.preheader ]
  %res_2 = mul i64 %res_1.ph4, %base
  %exp_2 = add i64 %exp_1.ph5, -1
  %finished_0 = icmp eq i64 %exp_2, 0
  br i1 %finished_0, label %for.end.pow, label %if.false.pow, !llvm.loop !11

for.end.pow:                                      ; preds = %if.false.pow, %middle.block, %pre_entry
  %res_1.ph.lcssa = phi i64 [ 1, %pre_entry ], [ %bin.rdx10, %middle.block ], [ %res_2, %if.false.pow ]
  ret i64 %res_1.ph.lcssa
}

; Function Attrs: nofree norecurse nosync nounwind memory(none)
define dso_local noundef i1 @__palindrome(i64 %in, i64 %len) local_unnamed_addr #2 {
pre_entry:
  %check_05 = icmp slt i64 %len, 1
  br i1 %check_05, label %if.end.palindrome, label %for.cond.pow.outer.i.preheader.preheader

for.cond.pow.outer.i.preheader.preheader:         ; preds = %pre_entry
  %in.fr4 = freeze i64 %in
  br label %for.cond.pow.outer.i.preheader

for.cond.pow.outer.i.preheader:                   ; preds = %for.cond.pow.outer.i.preheader.preheader, %if.true.mirror
  %in.fr7 = phi i64 [ %next_in_0, %if.true.mirror ], [ %in.fr4, %for.cond.pow.outer.i.preheader.preheader ]
  %len.tr6 = phi i64 [ %next_len_0, %if.true.mirror ], [ %len, %for.cond.pow.outer.i.preheader.preheader ]
  %min.iters.check = icmp ult i64 %len.tr6, 2
  br i1 %min.iters.check, label %if.false.pow.i.preheader, label %vector.ph

vector.ph:                                        ; preds = %for.cond.pow.outer.i.preheader
  %n.vec = and i64 %len.tr6, -2
  %ind.end = and i64 %len.tr6, 1
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 1, %vector.ph ], [ %0, %vector.body ]
  %vec.phi10 = phi i64 [ 1, %vector.ph ], [ %1, %vector.body ]
  %0 = mul i64 %vec.phi, 10
  %1 = mul i64 %vec.phi10, 10
  %index.next = add nuw i64 %index, 2
  %2 = icmp eq i64 %index.next, %n.vec
  br i1 %2, label %middle.block, label %vector.body, !llvm.loop !12

middle.block:                                     ; preds = %vector.body
  %bin.rdx = mul i64 %1, %0
  %cmp.n = icmp eq i64 %len.tr6, %n.vec
  br i1 %cmp.n, label %__pow.exit, label %if.false.pow.i.preheader

if.false.pow.i.preheader:                         ; preds = %for.cond.pow.outer.i.preheader, %middle.block
  %exp_1.ph.i3.ph = phi i64 [ %len.tr6, %for.cond.pow.outer.i.preheader ], [ %ind.end, %middle.block ]
  %res_1.ph.i2.ph = phi i64 [ 1, %for.cond.pow.outer.i.preheader ], [ %bin.rdx, %middle.block ]
  br label %if.false.pow.i

if.false.pow.i:                                   ; preds = %if.false.pow.i.preheader, %if.false.pow.i
  %exp_1.ph.i3 = phi i64 [ %exp_2.i, %if.false.pow.i ], [ %exp_1.ph.i3.ph, %if.false.pow.i.preheader ]
  %res_1.ph.i2 = phi i64 [ %res_2.i, %if.false.pow.i ], [ %res_1.ph.i2.ph, %if.false.pow.i.preheader ]
  %res_2.i = mul i64 %res_1.ph.i2, 10
  %exp_2.i = add i64 %exp_1.ph.i3, -1
  %finished_0.i = icmp eq i64 %exp_2.i, 0
  br i1 %finished_0.i, label %__pow.exit, label %if.false.pow.i, !llvm.loop !13

__pow.exit:                                       ; preds = %if.false.pow.i, %middle.block
  %res_2.i.lcssa = phi i64 [ %bin.rdx, %middle.block ], [ %res_2.i, %if.false.pow.i ]
  %left_0 = sdiv i64 %in.fr7, %res_2.i.lcssa
  %3 = srem i64 %in.fr7, 10
  %is_equal_0 = icmp eq i64 %left_0, %3
  br i1 %is_equal_0, label %if.true.mirror, label %if.end.palindrome

if.true.mirror:                                   ; preds = %__pow.exit
  %temp_09.neg = xor i64 %res_2.i.lcssa, -1
  %.neg = mul i64 %left_0, %temp_09.neg
  %.fr.neg = freeze i64 %.neg
  %temp_2 = add i64 %.fr.neg, %in.fr7
  %next_in_0 = sdiv i64 %temp_2, 10
  %next_len_0 = add nsw i64 %len.tr6, -2
  %check_0 = icmp slt i64 %len.tr6, 3
  br i1 %check_0, label %if.end.palindrome, label %for.cond.pow.outer.i.preheader

if.end.palindrome:                                ; preds = %if.true.mirror, %__pow.exit, %pre_entry
  %is_palindrome_4 = phi i1 [ true, %pre_entry ], [ %is_equal_0, %__pow.exit ], [ %is_equal_0, %if.true.mirror ]
  ret i1 %is_palindrome_4
}

define dso_local noundef i32 @main(i32 %argc, ptr nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %for.cond.pow.outer.i.preheader.i.preheader.i, label %2

2:                                                ; preds = %0
  %3 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.4, i32 0, i32 %1)
  tail call void @exit(i32 2)
  unreachable

for.cond.pow.outer.i.preheader.i.preheader.i:     ; preds = %0, %__orig_main.exit.i
  %loop_counter_11.i = phi i64 [ %loop_counter_2.i, %__orig_main.exit.i ], [ 10, %0 ]
  br label %for.cond.pow.outer.i.preheader.i.i

for.cond.pow.outer.i.preheader.i.i:               ; preds = %__pow.exit.i.i, %for.cond.pow.outer.i.preheader.i.preheader.i
  %index_16.i.i = phi i64 [ %spec.select1.i.i, %__pow.exit.i.i ], [ 1, %for.cond.pow.outer.i.preheader.i.preheader.i ]
  switch i64 %index_16.i.i, label %vector.ph6 [
    i64 0, label %__pow.exit.i.i
    i64 1, label %if.false.pow.i.i.i.preheader
  ]

vector.ph6:                                       ; preds = %for.cond.pow.outer.i.preheader.i.i
  %n.vec8 = and i64 %index_16.i.i, -2
  %ind.end9 = and i64 %index_16.i.i, 1
  br label %vector.body12

vector.body12:                                    ; preds = %vector.body12, %vector.ph6
  %index13 = phi i64 [ 0, %vector.ph6 ], [ %index.next16, %vector.body12 ]
  %vec.phi14 = phi i64 [ 1, %vector.ph6 ], [ %4, %vector.body12 ]
  %vec.phi15 = phi i64 [ 1, %vector.ph6 ], [ %5, %vector.body12 ]
  %4 = mul i64 %vec.phi14, 10
  %5 = mul i64 %vec.phi15, 10
  %index.next16 = add nuw i64 %index13, 2
  %6 = icmp eq i64 %index.next16, %n.vec8
  br i1 %6, label %middle.block3, label %vector.body12, !llvm.loop !14

middle.block3:                                    ; preds = %vector.body12
  %bin.rdx17 = mul i64 %5, %4
  %cmp.n11 = icmp eq i64 %index_16.i.i, %n.vec8
  br i1 %cmp.n11, label %__pow.exit.i.i, label %if.false.pow.i.i.i.preheader

if.false.pow.i.i.i.preheader:                     ; preds = %for.cond.pow.outer.i.preheader.i.i, %middle.block3
  %exp_1.ph.i5.i.i.ph = phi i64 [ %index_16.i.i, %for.cond.pow.outer.i.preheader.i.i ], [ %ind.end9, %middle.block3 ]
  %res_1.ph.i4.i.i.ph = phi i64 [ %index_16.i.i, %for.cond.pow.outer.i.preheader.i.i ], [ %bin.rdx17, %middle.block3 ]
  br label %if.false.pow.i.i.i

if.false.pow.i.i.i:                               ; preds = %if.false.pow.i.i.i.preheader, %if.false.pow.i.i.i
  %exp_1.ph.i5.i.i = phi i64 [ %exp_2.i.i.i, %if.false.pow.i.i.i ], [ %exp_1.ph.i5.i.i.ph, %if.false.pow.i.i.i.preheader ]
  %res_1.ph.i4.i.i = phi i64 [ %res_2.i.i.i, %if.false.pow.i.i.i ], [ %res_1.ph.i4.i.i.ph, %if.false.pow.i.i.i.preheader ]
  %res_2.i.i.i = mul i64 %res_1.ph.i4.i.i, 10
  %exp_2.i.i.i = add i64 %exp_1.ph.i5.i.i, -1
  %finished_0.i.i.i = icmp eq i64 %exp_2.i.i.i, 0
  br i1 %finished_0.i.i.i, label %__pow.exit.i.i, label %if.false.pow.i.i.i, !llvm.loop !15

__pow.exit.i.i:                                   ; preds = %if.false.pow.i.i.i, %middle.block3, %for.cond.pow.outer.i.preheader.i.i
  %res_1.ph.i.lcssa.i.i = phi i64 [ 1, %for.cond.pow.outer.i.preheader.i.i ], [ %bin.rdx17, %middle.block3 ], [ %res_2.i.i.i, %if.false.pow.i.i.i ]
  %d_0.i.i = sdiv i64 %loop_counter_11.i, %res_1.ph.i.lcssa.i.i
  %check_0.i.i = icmp ne i64 %d_0.i.i, 0
  %index_2.i.i = zext i1 %check_0.i.i to i64
  %spec.select1.i.i = add i64 %index_16.i.i, %index_2.i.i
  br i1 %check_0.i.i, label %for.cond.pow.outer.i.preheader.i.i, label %for.end.i.i

for.end.i.i:                                      ; preds = %__pow.exit.i.i
  %exp_0.i.i = add i64 %spec.select1.i.i, -1
  %check_05.i.i.i = icmp slt i64 %exp_0.i.i, 1
  br i1 %check_05.i.i.i, label %__orig_main.exit.i, label %for.cond.pow.outer.i.preheader.i.i.i

for.cond.pow.outer.i.preheader.i.i.i:             ; preds = %for.end.i.i, %if.true.mirror.i.i.i
  %in.fr7.i.i.i = phi i64 [ %next_in_0.i.i.i, %if.true.mirror.i.i.i ], [ %loop_counter_11.i, %for.end.i.i ]
  %len.tr6.i.i.i = phi i64 [ %next_len_0.i.i.i, %if.true.mirror.i.i.i ], [ %exp_0.i.i, %for.end.i.i ]
  %min.iters.check = icmp ult i64 %len.tr6.i.i.i, 2
  br i1 %min.iters.check, label %if.false.pow.i.i.i.i.preheader, label %vector.ph

vector.ph:                                        ; preds = %for.cond.pow.outer.i.preheader.i.i.i
  %n.vec = and i64 %len.tr6.i.i.i, -2
  %ind.end = and i64 %len.tr6.i.i.i, 1
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 1, %vector.ph ], [ %7, %vector.body ]
  %vec.phi2 = phi i64 [ 1, %vector.ph ], [ %8, %vector.body ]
  %7 = mul i64 %vec.phi, 10
  %8 = mul i64 %vec.phi2, 10
  %index.next = add nuw i64 %index, 2
  %9 = icmp eq i64 %index.next, %n.vec
  br i1 %9, label %middle.block, label %vector.body, !llvm.loop !16

middle.block:                                     ; preds = %vector.body
  %bin.rdx = mul i64 %8, %7
  %cmp.n = icmp eq i64 %len.tr6.i.i.i, %n.vec
  br i1 %cmp.n, label %__pow.exit.i.i.i, label %if.false.pow.i.i.i.i.preheader

if.false.pow.i.i.i.i.preheader:                   ; preds = %for.cond.pow.outer.i.preheader.i.i.i, %middle.block
  %exp_1.ph.i3.i.i.i.ph = phi i64 [ %len.tr6.i.i.i, %for.cond.pow.outer.i.preheader.i.i.i ], [ %ind.end, %middle.block ]
  %res_1.ph.i2.i.i.i.ph = phi i64 [ 1, %for.cond.pow.outer.i.preheader.i.i.i ], [ %bin.rdx, %middle.block ]
  br label %if.false.pow.i.i.i.i

if.false.pow.i.i.i.i:                             ; preds = %if.false.pow.i.i.i.i.preheader, %if.false.pow.i.i.i.i
  %exp_1.ph.i3.i.i.i = phi i64 [ %exp_2.i.i.i.i, %if.false.pow.i.i.i.i ], [ %exp_1.ph.i3.i.i.i.ph, %if.false.pow.i.i.i.i.preheader ]
  %res_1.ph.i2.i.i.i = phi i64 [ %res_2.i.i.i.i, %if.false.pow.i.i.i.i ], [ %res_1.ph.i2.i.i.i.ph, %if.false.pow.i.i.i.i.preheader ]
  %res_2.i.i.i.i = mul i64 %res_1.ph.i2.i.i.i, 10
  %exp_2.i.i.i.i = add i64 %exp_1.ph.i3.i.i.i, -1
  %finished_0.i.i.i.i = icmp eq i64 %exp_2.i.i.i.i, 0
  br i1 %finished_0.i.i.i.i, label %__pow.exit.i.i.i, label %if.false.pow.i.i.i.i, !llvm.loop !17

__pow.exit.i.i.i:                                 ; preds = %if.false.pow.i.i.i.i, %middle.block
  %res_2.i.i.i.i.lcssa = phi i64 [ %bin.rdx, %middle.block ], [ %res_2.i.i.i.i, %if.false.pow.i.i.i.i ]
  %left_0.i.i.i = sdiv i64 %in.fr7.i.i.i, %res_2.i.i.i.i.lcssa
  %10 = srem i64 %in.fr7.i.i.i, 10
  %is_equal_0.i.i.i = icmp eq i64 %left_0.i.i.i, %10
  br i1 %is_equal_0.i.i.i, label %if.true.mirror.i.i.i, label %__orig_main.exit.i

if.true.mirror.i.i.i:                             ; preds = %__pow.exit.i.i.i
  %temp_09.neg.i.i.i = xor i64 %res_2.i.i.i.i.lcssa, -1
  %.neg.i.i.i = mul i64 %left_0.i.i.i, %temp_09.neg.i.i.i
  %.fr.neg.i.i.i = freeze i64 %.neg.i.i.i
  %temp_2.i.i.i = add i64 %.fr.neg.i.i.i, %in.fr7.i.i.i
  %next_in_0.i.i.i = sdiv i64 %temp_2.i.i.i, 10
  %next_len_0.i.i.i = add i64 %len.tr6.i.i.i, -2
  %check_0.i.i.i = icmp slt i64 %len.tr6.i.i.i, 3
  br i1 %check_0.i.i.i, label %__orig_main.exit.i, label %for.cond.pow.outer.i.preheader.i.i.i

__orig_main.exit.i:                               ; preds = %if.true.mirror.i.i.i, %__pow.exit.i.i.i, %for.end.i.i
  %11 = phi ptr [ @.str, %for.end.i.i ], [ @.str, %if.true.mirror.i.i.i ], [ @.str.1, %__pow.exit.i.i.i ]
  %12 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) %11)
  %13 = tail call i32 @putchar(i32 10)
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 1000000
  br i1 %exitcond.not.i, label %__main.exit, label %for.cond.pow.outer.i.preheader.i.preheader.i

__main.exit:                                      ; preds = %__orig_main.exit.i
  ret i32 0
}

attributes #0 = { nofree nounwind }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read) }
attributes #2 = { nofree norecurse nosync nounwind memory(none) }

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
!10 = distinct !{!10, !1, !2}
!11 = distinct !{!11, !1}
!12 = distinct !{!12, !1, !2}
!13 = distinct !{!13, !1}
!14 = distinct !{!14, !1, !2}
!15 = distinct !{!15, !1}
!16 = distinct !{!16, !1, !2}
!17 = distinct !{!17, !1}
