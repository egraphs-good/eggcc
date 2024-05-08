; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpWrgUZa/compile.ll'
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

; Function Attrs: nofree nounwind
define dso_local void @__main() local_unnamed_addr #0 {
b0:
  br label %for.cond.pow.outer.i.preheader.i.preheader

for.cond.pow.outer.i.preheader.i.preheader:       ; preds = %b0, %__orig_main.exit
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %__orig_main.exit ]
  br label %for.cond.pow.outer.i.preheader.i

for.cond.pow.outer.i.preheader.i:                 ; preds = %for.cond.pow.outer.i.preheader.i.preheader, %__pow.exit.i
  %index_17.i = phi i64 [ %spec.select1.i, %__pow.exit.i ], [ 1, %for.cond.pow.outer.i.preheader.i.preheader ]
  switch i64 %index_17.i, label %vector.ph7 [
    i64 0, label %__pow.exit.i
    i64 1, label %if.false.pow.i.i.preheader
  ]

vector.ph7:                                       ; preds = %for.cond.pow.outer.i.preheader.i
  %n.vec9 = and i64 %index_17.i, -2
  %ind.end11 = and i64 %index_17.i, 1
  br label %vector.body13

vector.body13:                                    ; preds = %vector.body13, %vector.ph7
  %index14 = phi i64 [ 0, %vector.ph7 ], [ %index.next17, %vector.body13 ]
  %vec.phi15 = phi i64 [ 1, %vector.ph7 ], [ %0, %vector.body13 ]
  %vec.phi16 = phi i64 [ 1, %vector.ph7 ], [ %1, %vector.body13 ]
  %0 = mul i64 %vec.phi15, 10
  %1 = mul i64 %vec.phi16, 10
  %index.next17 = add nuw i64 %index14, 2
  %2 = icmp eq i64 %index.next17, %n.vec9
  br i1 %2, label %middle.block4, label %vector.body13, !llvm.loop !0

middle.block4:                                    ; preds = %vector.body13
  %bin.rdx18 = mul i64 %1, %0
  %cmp.n12 = icmp eq i64 %index_17.i, %n.vec9
  br i1 %cmp.n12, label %__pow.exit.i, label %if.false.pow.i.i.preheader

if.false.pow.i.i.preheader:                       ; preds = %for.cond.pow.outer.i.preheader.i, %middle.block4
  %exp_1.ph.i6.i.ph = phi i64 [ %index_17.i, %for.cond.pow.outer.i.preheader.i ], [ %ind.end11, %middle.block4 ]
  %res_1.ph.i5.i.ph = phi i64 [ %index_17.i, %for.cond.pow.outer.i.preheader.i ], [ %bin.rdx18, %middle.block4 ]
  br label %if.false.pow.i.i

if.false.pow.i.i:                                 ; preds = %if.false.pow.i.i.preheader, %if.false.pow.i.i
  %exp_1.ph.i6.i = phi i64 [ %exp_2.i.i, %if.false.pow.i.i ], [ %exp_1.ph.i6.i.ph, %if.false.pow.i.i.preheader ]
  %res_1.ph.i5.i = phi i64 [ %res_2.i.i, %if.false.pow.i.i ], [ %res_1.ph.i5.i.ph, %if.false.pow.i.i.preheader ]
  %res_2.i.i = mul i64 %res_1.ph.i5.i, 10
  %exp_2.i.i = add i64 %exp_1.ph.i6.i, -1
  %finished_0.i.i = icmp eq i64 %exp_2.i.i, 0
  br i1 %finished_0.i.i, label %__pow.exit.i, label %if.false.pow.i.i, !llvm.loop !2

__pow.exit.i:                                     ; preds = %if.false.pow.i.i, %middle.block4, %for.cond.pow.outer.i.preheader.i
  %res_1.ph.i.lcssa.i = phi i64 [ 1, %for.cond.pow.outer.i.preheader.i ], [ %bin.rdx18, %middle.block4 ], [ %res_2.i.i, %if.false.pow.i.i ]
  %d_0.i = sdiv i64 %loop_counter_11, %res_1.ph.i.lcssa.i
  %check_0.i = icmp ne i64 %d_0.i, 0
  %index_2.i = zext i1 %check_0.i to i64
  %spec.select1.i = add i64 %index_17.i, %index_2.i
  br i1 %check_0.i, label %for.cond.pow.outer.i.preheader.i, label %for.end.i

for.end.i:                                        ; preds = %__pow.exit.i
  %exp_0.i = add i64 %spec.select1.i, -1
  %check_05.i.i = icmp slt i64 %exp_0.i, 1
  br i1 %check_05.i.i, label %__orig_main.exit, label %for.cond.pow.outer.i.preheader.i.i

for.cond.pow.outer.i.preheader.i.i:               ; preds = %for.end.i, %if.true.mirror.i.i
  %len.tr7.i.i = phi i64 [ %next_len_0.i.i, %if.true.mirror.i.i ], [ %exp_0.i, %for.end.i ]
  %in.tr6.i.i = phi i64 [ %next_in_0.i.i, %if.true.mirror.i.i ], [ %loop_counter_11, %for.end.i ]
  %min.iters.check = icmp ult i64 %len.tr7.i.i, 2
  br i1 %min.iters.check, label %if.false.pow.i.i.i.preheader, label %vector.ph

vector.ph:                                        ; preds = %for.cond.pow.outer.i.preheader.i.i
  %n.vec = and i64 %len.tr7.i.i, -2
  %ind.end = and i64 %len.tr7.i.i, 1
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 1, %vector.ph ], [ %3, %vector.body ]
  %vec.phi3 = phi i64 [ 1, %vector.ph ], [ %4, %vector.body ]
  %3 = mul i64 %vec.phi, 10
  %4 = mul i64 %vec.phi3, 10
  %index.next = add nuw i64 %index, 2
  %5 = icmp eq i64 %index.next, %n.vec
  br i1 %5, label %middle.block, label %vector.body, !llvm.loop !3

middle.block:                                     ; preds = %vector.body
  %bin.rdx = mul i64 %4, %3
  %cmp.n = icmp eq i64 %len.tr7.i.i, %n.vec
  br i1 %cmp.n, label %for.cond.pow.outer.i.__pow.exit_crit_edge.i.i, label %if.false.pow.i.i.i.preheader

if.false.pow.i.i.i.preheader:                     ; preds = %for.cond.pow.outer.i.preheader.i.i, %middle.block
  %exp_1.ph.i4.i.i.ph = phi i64 [ 1, %for.cond.pow.outer.i.preheader.i.i ], [ %ind.end, %middle.block ]
  %res_1.ph.i3.i.i.ph = phi i64 [ 1, %for.cond.pow.outer.i.preheader.i.i ], [ %bin.rdx, %middle.block ]
  br label %if.false.pow.i.i.i

if.false.pow.i.i.i:                               ; preds = %if.false.pow.i.i.i.preheader, %if.false.pow.i.i.i
  %exp_1.ph.i4.i.i = phi i64 [ %exp_2.i.i.i, %if.false.pow.i.i.i ], [ %exp_1.ph.i4.i.i.ph, %if.false.pow.i.i.i.preheader ]
  %res_1.ph.i3.i.i = phi i64 [ %res_2.i.i.i, %if.false.pow.i.i.i ], [ %res_1.ph.i3.i.i.ph, %if.false.pow.i.i.i.preheader ]
  %res_2.i.i.i = mul i64 %res_1.ph.i3.i.i, 10
  %exp_2.i.i.i = add i64 %exp_1.ph.i4.i.i, -1
  %finished_0.i.i.i = icmp eq i64 %exp_2.i.i.i, 0
  br i1 %finished_0.i.i.i, label %for.cond.pow.outer.i.__pow.exit_crit_edge.i.i, label %if.false.pow.i.i.i, !llvm.loop !4

for.cond.pow.outer.i.__pow.exit_crit_edge.i.i:    ; preds = %if.false.pow.i.i.i, %middle.block
  %res_2.i.i.i.lcssa = phi i64 [ %bin.rdx, %middle.block ], [ %res_2.i.i.i, %if.false.pow.i.i.i ]
  %left_0.i.i = sdiv i64 %in.tr6.i.i, %res_2.i.i.i.lcssa
  %6 = srem i64 %in.tr6.i.i, 10
  %is_equal_0.i.i = icmp eq i64 %left_0.i.i, %6
  br i1 %is_equal_0.i.i, label %if.true.mirror.i.i, label %__orig_main.exit

if.true.mirror.i.i:                               ; preds = %for.cond.pow.outer.i.__pow.exit_crit_edge.i.i
  %temp_0.i.i = mul i64 %left_0.i.i, %res_2.i.i.i.lcssa
  %7 = add i64 %left_0.i.i, %temp_0.i.i
  %temp_2.i.i = sub i64 %in.tr6.i.i, %7
  %next_in_0.i.i = sdiv i64 %temp_2.i.i, 10
  %next_len_0.i.i = add nsw i64 %len.tr7.i.i, -2
  %check_0.i.i = icmp slt i64 %len.tr7.i.i, 3
  br i1 %check_0.i.i, label %__orig_main.exit, label %for.cond.pow.outer.i.preheader.i.i

__orig_main.exit:                                 ; preds = %for.cond.pow.outer.i.__pow.exit_crit_edge.i.i, %if.true.mirror.i.i, %for.end.i
  %8 = phi i8* [ getelementptr inbounds ([5 x i8], [5 x i8]* @.str, i64 0, i64 0), %for.end.i ], [ getelementptr inbounds ([6 x i8], [6 x i8]* @.str.1, i64 0, i64 0), %for.cond.pow.outer.i.__pow.exit_crit_edge.i.i ], [ getelementptr inbounds ([5 x i8], [5 x i8]* @.str, i64 0, i64 0), %if.true.mirror.i.i ]
  %9 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) %8) #5
  %10 = tail call i32 @putchar(i32 10) #5
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 1000000
  br i1 %exitcond.not, label %loop_done, label %for.cond.pow.outer.i.preheader.i.preheader

loop_done:                                        ; preds = %__orig_main.exit
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %in) local_unnamed_addr #0 {
pre_entry:
  br label %for.cond.pow.outer.i.preheader

for.cond.pow.outer.i.preheader:                   ; preds = %pre_entry, %__pow.exit
  %index_17 = phi i64 [ 1, %pre_entry ], [ %spec.select1, %__pow.exit ]
  switch i64 %index_17, label %vector.ph [
    i64 0, label %__pow.exit
    i64 1, label %if.false.pow.i.preheader
  ]

vector.ph:                                        ; preds = %for.cond.pow.outer.i.preheader
  %n.vec = and i64 %index_17, -2
  %ind.end = and i64 %index_17, 1
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 1, %vector.ph ], [ %0, %vector.body ]
  %vec.phi9 = phi i64 [ 1, %vector.ph ], [ %1, %vector.body ]
  %0 = mul i64 %vec.phi, 10
  %1 = mul i64 %vec.phi9, 10
  %index.next = add nuw i64 %index, 2
  %2 = icmp eq i64 %index.next, %n.vec
  br i1 %2, label %middle.block, label %vector.body, !llvm.loop !5

middle.block:                                     ; preds = %vector.body
  %bin.rdx = mul i64 %1, %0
  %cmp.n = icmp eq i64 %index_17, %n.vec
  br i1 %cmp.n, label %__pow.exit, label %if.false.pow.i.preheader

if.false.pow.i.preheader:                         ; preds = %for.cond.pow.outer.i.preheader, %middle.block
  %exp_1.ph.i6.ph = phi i64 [ %index_17, %for.cond.pow.outer.i.preheader ], [ %ind.end, %middle.block ]
  %res_1.ph.i5.ph = phi i64 [ %index_17, %for.cond.pow.outer.i.preheader ], [ %bin.rdx, %middle.block ]
  br label %if.false.pow.i

if.false.pow.i:                                   ; preds = %if.false.pow.i.preheader, %if.false.pow.i
  %exp_1.ph.i6 = phi i64 [ %exp_2.i, %if.false.pow.i ], [ %exp_1.ph.i6.ph, %if.false.pow.i.preheader ]
  %res_1.ph.i5 = phi i64 [ %res_2.i, %if.false.pow.i ], [ %res_1.ph.i5.ph, %if.false.pow.i.preheader ]
  %res_2.i = mul i64 %res_1.ph.i5, 10
  %exp_2.i = add i64 %exp_1.ph.i6, -1
  %finished_0.i = icmp eq i64 %exp_2.i, 0
  br i1 %finished_0.i, label %__pow.exit, label %if.false.pow.i, !llvm.loop !6

__pow.exit:                                       ; preds = %if.false.pow.i, %middle.block, %for.cond.pow.outer.i.preheader
  %res_1.ph.i.lcssa = phi i64 [ 1, %for.cond.pow.outer.i.preheader ], [ %bin.rdx, %middle.block ], [ %res_2.i, %if.false.pow.i ]
  %d_0 = sdiv i64 %in, %res_1.ph.i.lcssa
  %check_0 = icmp ne i64 %d_0, 0
  %index_2 = zext i1 %check_0 to i64
  %spec.select1 = add i64 %index_17, %index_2
  br i1 %check_0, label %for.cond.pow.outer.i.preheader, label %for.end

for.end:                                          ; preds = %__pow.exit
  %exp_0 = add i64 %spec.select1, -1
  %check_05.i = icmp slt i64 %exp_0, 1
  br i1 %check_05.i, label %__palindrome.exit, label %for.cond.pow.outer.i.preheader.i

for.cond.pow.outer.i.preheader.i:                 ; preds = %for.end, %if.true.mirror.i
  %indvar = phi i64 [ %indvar.next, %if.true.mirror.i ], [ 0, %for.end ]
  %len.tr7.i = phi i64 [ %next_len_0.i, %if.true.mirror.i ], [ %exp_0, %for.end ]
  %in.tr6.i = phi i64 [ %next_in_0.i, %if.true.mirror.i ], [ %in, %for.end ]
  %3 = mul nsw i64 %indvar, -2
  %4 = add i64 %exp_0, %3
  %min.iters.check12 = icmp ult i64 %4, 2
  br i1 %min.iters.check12, label %if.false.pow.i.i.preheader, label %vector.ph13

vector.ph13:                                      ; preds = %for.cond.pow.outer.i.preheader.i
  %n.vec15 = and i64 %4, -2
  %ind.end17 = sub i64 %len.tr7.i, %n.vec15
  br label %vector.body19

vector.body19:                                    ; preds = %vector.body19, %vector.ph13
  %index20 = phi i64 [ 0, %vector.ph13 ], [ %index.next23, %vector.body19 ]
  %vec.phi21 = phi i64 [ 1, %vector.ph13 ], [ %5, %vector.body19 ]
  %vec.phi22 = phi i64 [ 1, %vector.ph13 ], [ %6, %vector.body19 ]
  %5 = mul i64 %vec.phi21, 10
  %6 = mul i64 %vec.phi22, 10
  %index.next23 = add nuw i64 %index20, 2
  %7 = icmp eq i64 %index.next23, %n.vec15
  br i1 %7, label %middle.block10, label %vector.body19, !llvm.loop !7

middle.block10:                                   ; preds = %vector.body19
  %bin.rdx24 = mul i64 %6, %5
  %cmp.n18 = icmp eq i64 %4, %n.vec15
  br i1 %cmp.n18, label %for.cond.pow.outer.i.__pow.exit_crit_edge.i, label %if.false.pow.i.i.preheader

if.false.pow.i.i.preheader:                       ; preds = %for.cond.pow.outer.i.preheader.i, %middle.block10
  %exp_1.ph.i4.i.ph = phi i64 [ %len.tr7.i, %for.cond.pow.outer.i.preheader.i ], [ %ind.end17, %middle.block10 ]
  %res_1.ph.i3.i.ph = phi i64 [ 1, %for.cond.pow.outer.i.preheader.i ], [ %bin.rdx24, %middle.block10 ]
  br label %if.false.pow.i.i

if.false.pow.i.i:                                 ; preds = %if.false.pow.i.i.preheader, %if.false.pow.i.i
  %exp_1.ph.i4.i = phi i64 [ %exp_2.i.i, %if.false.pow.i.i ], [ %exp_1.ph.i4.i.ph, %if.false.pow.i.i.preheader ]
  %res_1.ph.i3.i = phi i64 [ %res_2.i.i, %if.false.pow.i.i ], [ %res_1.ph.i3.i.ph, %if.false.pow.i.i.preheader ]
  %res_2.i.i = mul i64 %res_1.ph.i3.i, 10
  %exp_2.i.i = add i64 %exp_1.ph.i4.i, -1
  %finished_0.i.i = icmp eq i64 %exp_2.i.i, 0
  br i1 %finished_0.i.i, label %for.cond.pow.outer.i.__pow.exit_crit_edge.i, label %if.false.pow.i.i, !llvm.loop !8

for.cond.pow.outer.i.__pow.exit_crit_edge.i:      ; preds = %if.false.pow.i.i, %middle.block10
  %res_2.i.i.lcssa = phi i64 [ %bin.rdx24, %middle.block10 ], [ %res_2.i.i, %if.false.pow.i.i ]
  %left_0.i = sdiv i64 %in.tr6.i, %res_2.i.i.lcssa
  %8 = srem i64 %in.tr6.i, 10
  %is_equal_0.i = icmp eq i64 %left_0.i, %8
  br i1 %is_equal_0.i, label %if.true.mirror.i, label %__palindrome.exit

if.true.mirror.i:                                 ; preds = %for.cond.pow.outer.i.__pow.exit_crit_edge.i
  %temp_0.i = mul i64 %left_0.i, %res_2.i.i.lcssa
  %9 = add i64 %left_0.i, %temp_0.i
  %temp_2.i = sub i64 %in.tr6.i, %9
  %next_in_0.i = sdiv i64 %temp_2.i, 10
  %next_len_0.i = add nsw i64 %len.tr7.i, -2
  %check_0.i = icmp slt i64 %len.tr7.i, 3
  %indvar.next = add i64 %indvar, 1
  br i1 %check_0.i, label %__palindrome.exit, label %for.cond.pow.outer.i.preheader.i

__palindrome.exit:                                ; preds = %if.true.mirror.i, %for.cond.pow.outer.i.__pow.exit_crit_edge.i, %for.end
  %10 = phi i8* [ getelementptr inbounds ([5 x i8], [5 x i8]* @.str, i64 0, i64 0), %for.end ], [ getelementptr inbounds ([5 x i8], [5 x i8]* @.str, i64 0, i64 0), %if.true.mirror.i ], [ getelementptr inbounds ([6 x i8], [6 x i8]* @.str.1, i64 0, i64 0), %for.cond.pow.outer.i.__pow.exit_crit_edge.i ]
  %11 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) %10) #5
  %12 = tail call i32 @putchar(i32 10) #5
  ret void
}

; Function Attrs: nofree norecurse nosync nounwind readnone
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
  br i1 %4, label %middle.block, label %vector.body, !llvm.loop !9

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
  br i1 %finished_0, label %for.end.pow, label %if.false.pow, !llvm.loop !10

for.end.pow:                                      ; preds = %if.false.pow, %middle.block, %pre_entry
  %res_1.ph.lcssa = phi i64 [ 1, %pre_entry ], [ %bin.rdx10, %middle.block ], [ %res_2, %if.false.pow ]
  ret i64 %res_1.ph.lcssa
}

; Function Attrs: nofree nosync nounwind readnone
define dso_local i1 @__palindrome(i64 %in, i64 %len) local_unnamed_addr #3 {
pre_entry:
  %check_05 = icmp slt i64 %len, 1
  br i1 %check_05, label %if.end.palindrome, label %for.cond.pow.outer.i.preheader

for.cond.pow.outer.i.preheader:                   ; preds = %pre_entry, %if.true.mirror
  %len.tr7 = phi i64 [ %next_len_0, %if.true.mirror ], [ %len, %pre_entry ]
  %in.tr6 = phi i64 [ %next_in_0, %if.true.mirror ], [ %in, %pre_entry ]
  %min.iters.check = icmp ult i64 %len.tr7, 2
  br i1 %min.iters.check, label %if.false.pow.i.preheader, label %vector.ph

vector.ph:                                        ; preds = %for.cond.pow.outer.i.preheader
  %n.vec = and i64 %len.tr7, -2
  %ind.end = and i64 %len.tr7, 1
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 1, %vector.ph ], [ %0, %vector.body ]
  %vec.phi10 = phi i64 [ 1, %vector.ph ], [ %1, %vector.body ]
  %0 = mul i64 %vec.phi, 10
  %1 = mul i64 %vec.phi10, 10
  %index.next = add nuw i64 %index, 2
  %2 = icmp eq i64 %index.next, %n.vec
  br i1 %2, label %middle.block, label %vector.body, !llvm.loop !11

middle.block:                                     ; preds = %vector.body
  %bin.rdx = mul i64 %1, %0
  %cmp.n = icmp eq i64 %len.tr7, %n.vec
  br i1 %cmp.n, label %for.cond.pow.outer.i.__pow.exit_crit_edge, label %if.false.pow.i.preheader

if.false.pow.i.preheader:                         ; preds = %for.cond.pow.outer.i.preheader, %middle.block
  %exp_1.ph.i4.ph = phi i64 [ 1, %for.cond.pow.outer.i.preheader ], [ %ind.end, %middle.block ]
  %res_1.ph.i3.ph = phi i64 [ 1, %for.cond.pow.outer.i.preheader ], [ %bin.rdx, %middle.block ]
  br label %if.false.pow.i

if.false.pow.i:                                   ; preds = %if.false.pow.i.preheader, %if.false.pow.i
  %exp_1.ph.i4 = phi i64 [ %exp_2.i, %if.false.pow.i ], [ %exp_1.ph.i4.ph, %if.false.pow.i.preheader ]
  %res_1.ph.i3 = phi i64 [ %res_2.i, %if.false.pow.i ], [ %res_1.ph.i3.ph, %if.false.pow.i.preheader ]
  %res_2.i = mul i64 %res_1.ph.i3, 10
  %exp_2.i = add i64 %exp_1.ph.i4, -1
  %finished_0.i = icmp eq i64 %exp_2.i, 0
  br i1 %finished_0.i, label %for.cond.pow.outer.i.__pow.exit_crit_edge, label %if.false.pow.i, !llvm.loop !12

for.cond.pow.outer.i.__pow.exit_crit_edge:        ; preds = %if.false.pow.i, %middle.block
  %res_2.i.lcssa = phi i64 [ %bin.rdx, %middle.block ], [ %res_2.i, %if.false.pow.i ]
  %left_0 = sdiv i64 %in.tr6, %res_2.i.lcssa
  %3 = srem i64 %in.tr6, 10
  %is_equal_0 = icmp eq i64 %left_0, %3
  br i1 %is_equal_0, label %if.true.mirror, label %if.end.palindrome

if.true.mirror:                                   ; preds = %for.cond.pow.outer.i.__pow.exit_crit_edge
  %temp_0 = mul i64 %left_0, %res_2.i.lcssa
  %4 = add i64 %left_0, %temp_0
  %temp_2 = sub i64 %in.tr6, %4
  %next_in_0 = sdiv i64 %temp_2, 10
  %next_len_0 = add nsw i64 %len.tr7, -2
  %check_0 = icmp slt i64 %len.tr7, 3
  br i1 %check_0, label %if.end.palindrome, label %for.cond.pow.outer.i.preheader

if.end.palindrome:                                ; preds = %if.true.mirror, %for.cond.pow.outer.i.__pow.exit_crit_edge, %pre_entry
  %is_palindrome_4 = phi i1 [ true, %pre_entry ], [ true, %if.true.mirror ], [ %is_equal_0, %for.cond.pow.outer.i.__pow.exit_crit_edge ]
  ret i1 %is_palindrome_4
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %for.cond.pow.outer.i.preheader.i.preheader.i, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #6
  ret i32 0

for.cond.pow.outer.i.preheader.i.preheader.i:     ; preds = %0, %__orig_main.exit.i
  %loop_counter_11.i = phi i64 [ %loop_counter_2.i, %__orig_main.exit.i ], [ 10, %0 ]
  br label %for.cond.pow.outer.i.preheader.i.i

for.cond.pow.outer.i.preheader.i.i:               ; preds = %__pow.exit.i.i, %for.cond.pow.outer.i.preheader.i.preheader.i
  %index_17.i.i = phi i64 [ %spec.select1.i.i, %__pow.exit.i.i ], [ 1, %for.cond.pow.outer.i.preheader.i.preheader.i ]
  switch i64 %index_17.i.i, label %vector.ph6 [
    i64 0, label %__pow.exit.i.i
    i64 1, label %if.false.pow.i.i.i.preheader
  ]

vector.ph6:                                       ; preds = %for.cond.pow.outer.i.preheader.i.i
  %n.vec8 = and i64 %index_17.i.i, -2
  %ind.end10 = and i64 %index_17.i.i, 1
  br label %vector.body12

vector.body12:                                    ; preds = %vector.body12, %vector.ph6
  %index13 = phi i64 [ 0, %vector.ph6 ], [ %index.next16, %vector.body12 ]
  %vec.phi14 = phi i64 [ 1, %vector.ph6 ], [ %2, %vector.body12 ]
  %vec.phi15 = phi i64 [ 1, %vector.ph6 ], [ %3, %vector.body12 ]
  %2 = mul i64 %vec.phi14, 10
  %3 = mul i64 %vec.phi15, 10
  %index.next16 = add nuw i64 %index13, 2
  %4 = icmp eq i64 %index.next16, %n.vec8
  br i1 %4, label %middle.block3, label %vector.body12, !llvm.loop !13

middle.block3:                                    ; preds = %vector.body12
  %bin.rdx17 = mul i64 %3, %2
  %cmp.n11 = icmp eq i64 %index_17.i.i, %n.vec8
  br i1 %cmp.n11, label %__pow.exit.i.i, label %if.false.pow.i.i.i.preheader

if.false.pow.i.i.i.preheader:                     ; preds = %for.cond.pow.outer.i.preheader.i.i, %middle.block3
  %exp_1.ph.i6.i.i.ph = phi i64 [ %index_17.i.i, %for.cond.pow.outer.i.preheader.i.i ], [ %ind.end10, %middle.block3 ]
  %res_1.ph.i5.i.i.ph = phi i64 [ %index_17.i.i, %for.cond.pow.outer.i.preheader.i.i ], [ %bin.rdx17, %middle.block3 ]
  br label %if.false.pow.i.i.i

if.false.pow.i.i.i:                               ; preds = %if.false.pow.i.i.i.preheader, %if.false.pow.i.i.i
  %exp_1.ph.i6.i.i = phi i64 [ %exp_2.i.i.i, %if.false.pow.i.i.i ], [ %exp_1.ph.i6.i.i.ph, %if.false.pow.i.i.i.preheader ]
  %res_1.ph.i5.i.i = phi i64 [ %res_2.i.i.i, %if.false.pow.i.i.i ], [ %res_1.ph.i5.i.i.ph, %if.false.pow.i.i.i.preheader ]
  %res_2.i.i.i = mul i64 %res_1.ph.i5.i.i, 10
  %exp_2.i.i.i = add i64 %exp_1.ph.i6.i.i, -1
  %finished_0.i.i.i = icmp eq i64 %exp_2.i.i.i, 0
  br i1 %finished_0.i.i.i, label %__pow.exit.i.i, label %if.false.pow.i.i.i, !llvm.loop !14

__pow.exit.i.i:                                   ; preds = %if.false.pow.i.i.i, %middle.block3, %for.cond.pow.outer.i.preheader.i.i
  %res_1.ph.i.lcssa.i.i = phi i64 [ 1, %for.cond.pow.outer.i.preheader.i.i ], [ %bin.rdx17, %middle.block3 ], [ %res_2.i.i.i, %if.false.pow.i.i.i ]
  %d_0.i.i = sdiv i64 %loop_counter_11.i, %res_1.ph.i.lcssa.i.i
  %check_0.i.i = icmp ne i64 %d_0.i.i, 0
  %index_2.i.i = zext i1 %check_0.i.i to i64
  %spec.select1.i.i = add i64 %index_17.i.i, %index_2.i.i
  br i1 %check_0.i.i, label %for.cond.pow.outer.i.preheader.i.i, label %for.end.i.i

for.end.i.i:                                      ; preds = %__pow.exit.i.i
  %exp_0.i.i = add i64 %spec.select1.i.i, -1
  %check_05.i.i.i = icmp slt i64 %exp_0.i.i, 1
  br i1 %check_05.i.i.i, label %__orig_main.exit.i, label %for.cond.pow.outer.i.preheader.i.i.i

for.cond.pow.outer.i.preheader.i.i.i:             ; preds = %for.end.i.i, %if.true.mirror.i.i.i
  %len.tr7.i.i.i = phi i64 [ %next_len_0.i.i.i, %if.true.mirror.i.i.i ], [ %exp_0.i.i, %for.end.i.i ]
  %in.tr6.i.i.i = phi i64 [ %next_in_0.i.i.i, %if.true.mirror.i.i.i ], [ %loop_counter_11.i, %for.end.i.i ]
  %min.iters.check = icmp ult i64 %len.tr7.i.i.i, 2
  br i1 %min.iters.check, label %if.false.pow.i.i.i.i.preheader, label %vector.ph

vector.ph:                                        ; preds = %for.cond.pow.outer.i.preheader.i.i.i
  %n.vec = and i64 %len.tr7.i.i.i, -2
  %ind.end = and i64 %len.tr7.i.i.i, 1
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %vec.phi = phi i64 [ 1, %vector.ph ], [ %5, %vector.body ]
  %vec.phi2 = phi i64 [ 1, %vector.ph ], [ %6, %vector.body ]
  %5 = mul i64 %vec.phi, 10
  %6 = mul i64 %vec.phi2, 10
  %index.next = add nuw i64 %index, 2
  %7 = icmp eq i64 %index.next, %n.vec
  br i1 %7, label %middle.block, label %vector.body, !llvm.loop !15

middle.block:                                     ; preds = %vector.body
  %bin.rdx = mul i64 %6, %5
  %cmp.n = icmp eq i64 %len.tr7.i.i.i, %n.vec
  br i1 %cmp.n, label %for.cond.pow.outer.i.__pow.exit_crit_edge.i.i.i, label %if.false.pow.i.i.i.i.preheader

if.false.pow.i.i.i.i.preheader:                   ; preds = %for.cond.pow.outer.i.preheader.i.i.i, %middle.block
  %exp_1.ph.i4.i.i.i.ph = phi i64 [ 1, %for.cond.pow.outer.i.preheader.i.i.i ], [ %ind.end, %middle.block ]
  %res_1.ph.i3.i.i.i.ph = phi i64 [ 1, %for.cond.pow.outer.i.preheader.i.i.i ], [ %bin.rdx, %middle.block ]
  br label %if.false.pow.i.i.i.i

if.false.pow.i.i.i.i:                             ; preds = %if.false.pow.i.i.i.i.preheader, %if.false.pow.i.i.i.i
  %exp_1.ph.i4.i.i.i = phi i64 [ %exp_2.i.i.i.i, %if.false.pow.i.i.i.i ], [ %exp_1.ph.i4.i.i.i.ph, %if.false.pow.i.i.i.i.preheader ]
  %res_1.ph.i3.i.i.i = phi i64 [ %res_2.i.i.i.i, %if.false.pow.i.i.i.i ], [ %res_1.ph.i3.i.i.i.ph, %if.false.pow.i.i.i.i.preheader ]
  %res_2.i.i.i.i = mul i64 %res_1.ph.i3.i.i.i, 10
  %exp_2.i.i.i.i = add i64 %exp_1.ph.i4.i.i.i, -1
  %finished_0.i.i.i.i = icmp eq i64 %exp_2.i.i.i.i, 0
  br i1 %finished_0.i.i.i.i, label %for.cond.pow.outer.i.__pow.exit_crit_edge.i.i.i, label %if.false.pow.i.i.i.i, !llvm.loop !16

for.cond.pow.outer.i.__pow.exit_crit_edge.i.i.i:  ; preds = %if.false.pow.i.i.i.i, %middle.block
  %res_2.i.i.i.i.lcssa = phi i64 [ %bin.rdx, %middle.block ], [ %res_2.i.i.i.i, %if.false.pow.i.i.i.i ]
  %left_0.i.i.i = sdiv i64 %in.tr6.i.i.i, %res_2.i.i.i.i.lcssa
  %8 = srem i64 %in.tr6.i.i.i, 10
  %is_equal_0.i.i.i = icmp eq i64 %left_0.i.i.i, %8
  br i1 %is_equal_0.i.i.i, label %if.true.mirror.i.i.i, label %__orig_main.exit.i

if.true.mirror.i.i.i:                             ; preds = %for.cond.pow.outer.i.__pow.exit_crit_edge.i.i.i
  %temp_0.i.i.i = mul i64 %left_0.i.i.i, %res_2.i.i.i.i.lcssa
  %9 = add i64 %left_0.i.i.i, %temp_0.i.i.i
  %temp_2.i.i.i = sub i64 %in.tr6.i.i.i, %9
  %next_in_0.i.i.i = sdiv i64 %temp_2.i.i.i, 10
  %next_len_0.i.i.i = add nsw i64 %len.tr7.i.i.i, -2
  %check_0.i.i.i = icmp slt i64 %len.tr7.i.i.i, 3
  br i1 %check_0.i.i.i, label %__orig_main.exit.i, label %for.cond.pow.outer.i.preheader.i.i.i

__orig_main.exit.i:                               ; preds = %if.true.mirror.i.i.i, %for.cond.pow.outer.i.__pow.exit_crit_edge.i.i.i, %for.end.i.i
  %10 = phi i8* [ getelementptr inbounds ([5 x i8], [5 x i8]* @.str, i64 0, i64 0), %for.end.i.i ], [ getelementptr inbounds ([5 x i8], [5 x i8]* @.str, i64 0, i64 0), %if.true.mirror.i.i.i ], [ getelementptr inbounds ([6 x i8], [6 x i8]* @.str.1, i64 0, i64 0), %for.cond.pow.outer.i.__pow.exit_crit_edge.i.i.i ]
  %11 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) %10) #5
  %12 = tail call i32 @putchar(i32 10) #5
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 1000000
  br i1 %exitcond.not.i, label %__main.exit, label %for.cond.pow.outer.i.preheader.i.preheader.i

__main.exit:                                      ; preds = %__orig_main.exit.i
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
attributes #2 = { nofree norecurse nosync nounwind readnone }
attributes #3 = { nofree nosync nounwind readnone }
attributes #4 = { cold minsize noreturn }
attributes #5 = { nounwind }
attributes #6 = { noinline }

!0 = distinct !{!0, !1}
!1 = !{!"llvm.loop.isvectorized", i32 1}
!2 = distinct !{!2, !1}
!3 = distinct !{!3, !1}
!4 = distinct !{!4, !1}
!5 = distinct !{!5, !1}
!6 = distinct !{!6, !1}
!7 = distinct !{!7, !1}
!8 = distinct !{!8, !1}
!9 = distinct !{!9, !1}
!10 = distinct !{!10, !1}
!11 = distinct !{!11, !1}
!12 = distinct !{!12, !1}
!13 = distinct !{!13, !1}
!14 = distinct !{!14, !1}
!15 = distinct !{!15, !1}
!16 = distinct !{!16, !1}
