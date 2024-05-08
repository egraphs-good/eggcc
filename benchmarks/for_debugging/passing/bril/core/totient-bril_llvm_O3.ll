; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmp8ZzNS7/totient-init.ll'
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
  br label %for.set.body.i.i.preheader

for.set.body.i.i.preheader:                       ; preds = %__orig_main.exit, %b0
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %__orig_main.exit ]
  %0 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %loop_counter_11)
  %1 = tail call i32 @putchar(i32 10)
  br label %for.set.body.i.i

for.set.body.i.i:                                 ; preds = %for.set.body.i.i.preheader, %else_lbl.i.i
  %n_18.i.i = phi i64 [ %n_4.i.i, %else_lbl.i.i ], [ %loop_counter_11, %for.set.body.i.i.preheader ]
  %result_17.i.i = phi i64 [ %result_3.i.i, %else_lbl.i.i ], [ %loop_counter_11, %for.set.body.i.i.preheader ]
  %p_15.i.i = phi i64 [ %p_2.i.i, %else_lbl.i.i ], [ 2, %for.set.body.i.i.preheader ]
  %2 = srem i64 %n_18.i.i, %p_15.i.i
  %if_cond_0.i.i = icmp eq i64 %2, 0
  br i1 %if_cond_0.i.i, label %while.body.i.i, label %else_lbl.i.i

while.body.i.i:                                   ; preds = %for.set.body.i.i, %while.body.i.i
  %n_23.i.i = phi i64 [ %npdiv_0.i.i, %while.body.i.i ], [ %n_18.i.i, %for.set.body.i.i ]
  %npdiv_0.i.i = sdiv i64 %n_23.i.i, %p_15.i.i
  %3 = srem i64 %npdiv_0.i.i, %p_15.i.i
  %while_cond_0.i.i = icmp eq i64 %3, 0
  br i1 %while_cond_0.i.i, label %while.body.i.i, label %while.end.i.i

while.end.i.i:                                    ; preds = %while.body.i.i
  %resdiv_0.i.i = sdiv i64 %result_17.i.i, %p_15.i.i
  %result_2.i.i = sub i64 %result_17.i.i, %resdiv_0.i.i
  br label %else_lbl.i.i

else_lbl.i.i:                                     ; preds = %while.end.i.i, %for.set.body.i.i
  %result_3.i.i = phi i64 [ %result_2.i.i, %while.end.i.i ], [ %result_17.i.i, %for.set.body.i.i ]
  %n_4.i.i = phi i64 [ %npdiv_0.i.i, %while.end.i.i ], [ %n_18.i.i, %for.set.body.i.i ]
  %p_2.i.i = add i64 %p_15.i.i, 1
  %pp_0.i.i = mul i64 %p_2.i.i, %p_2.i.i
  %cond_0.not.i.i = icmp sgt i64 %pp_0.i.i, %n_4.i.i
  br i1 %cond_0.not.i.i, label %for.set.end.i.i, label %for.set.body.i.i

for.set.end.i.i:                                  ; preds = %else_lbl.i.i
  %final_if_cond_0.i.i = icmp sgt i64 %n_4.i.i, 1
  br i1 %final_if_cond_0.i.i, label %final_if_label.i.i, label %__orig_main.exit

final_if_label.i.i:                               ; preds = %for.set.end.i.i
  %resdiv_1.i.i = sdiv i64 %result_3.i.i, %n_4.i.i
  %result_4.i.i = sub i64 %result_3.i.i, %resdiv_1.i.i
  br label %__orig_main.exit

__orig_main.exit:                                 ; preds = %for.set.end.i.i, %final_if_label.i.i
  %result_5.i.i = phi i64 [ %result_4.i.i, %final_if_label.i.i ], [ %result_3.i.i, %for.set.end.i.i ]
  %4 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %result_5.i.i)
  %5 = tail call i32 @putchar(i32 10)
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 1000000
  br i1 %exitcond.not, label %loop_done, label %for.set.body.i.i.preheader

loop_done:                                        ; preds = %__orig_main.exit
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %n) local_unnamed_addr #0 {
pre_entry:
  %n.fr.i = freeze i64 %n
  %0 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %n.fr.i)
  %1 = tail call i32 @putchar(i32 10)
  %cond_0.not4.i = icmp slt i64 %n.fr.i, 4
  br i1 %cond_0.not4.i, label %for.set.end.i, label %for.set.body.i

for.set.body.i:                                   ; preds = %pre_entry, %else_lbl.i
  %n_18.i = phi i64 [ %n_4.i, %else_lbl.i ], [ %n.fr.i, %pre_entry ]
  %result_17.i = phi i64 [ %result_3.i, %else_lbl.i ], [ %n.fr.i, %pre_entry ]
  %p_15.i = phi i64 [ %p_2.i, %else_lbl.i ], [ 2, %pre_entry ]
  %2 = srem i64 %n_18.i, %p_15.i
  %if_cond_0.i = icmp eq i64 %2, 0
  br i1 %if_cond_0.i, label %while.body.i, label %else_lbl.i

while.body.i:                                     ; preds = %for.set.body.i, %while.body.i
  %n_23.i = phi i64 [ %npdiv_0.i, %while.body.i ], [ %n_18.i, %for.set.body.i ]
  %npdiv_0.i = sdiv i64 %n_23.i, %p_15.i
  %3 = srem i64 %npdiv_0.i, %p_15.i
  %while_cond_0.i = icmp eq i64 %3, 0
  br i1 %while_cond_0.i, label %while.body.i, label %while.end.i

while.end.i:                                      ; preds = %while.body.i
  %resdiv_0.i = sdiv i64 %result_17.i, %p_15.i
  %result_2.i = sub i64 %result_17.i, %resdiv_0.i
  br label %else_lbl.i

else_lbl.i:                                       ; preds = %while.end.i, %for.set.body.i
  %result_3.i = phi i64 [ %result_2.i, %while.end.i ], [ %result_17.i, %for.set.body.i ]
  %n_4.i = phi i64 [ %npdiv_0.i, %while.end.i ], [ %n_18.i, %for.set.body.i ]
  %p_2.i = add i64 %p_15.i, 1
  %pp_0.i = mul i64 %p_2.i, %p_2.i
  %cond_0.not.i = icmp sgt i64 %pp_0.i, %n_4.i
  br i1 %cond_0.not.i, label %for.set.end.i, label %for.set.body.i

for.set.end.i:                                    ; preds = %else_lbl.i, %pre_entry
  %final_if_cond_0.i = icmp sgt i64 %n_4.i, 1
  br i1 %final_if_cond_0.i, label %final_if_label.i, label %__totient.exit

final_if_label.i:                                 ; preds = %for.set.end.i
  %resdiv_1.i = sdiv i64 %result_3.i, %n_4.i
  %result_4.i = sub i64 %result_3.i, %resdiv_1.i
  br label %__totient.exit

__totient.exit:                                   ; preds = %for.set.end.i, %final_if_label.i
  %result_5.i = phi i64 [ %result_4.i, %final_if_label.i ], [ %result_3.i, %for.set.end.i ]
  %4 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %result_5.i)
  %5 = tail call i32 @putchar(i32 10)
  ret void
}

; Function Attrs: nofree norecurse nosync nounwind memory(none)
define dso_local i64 @__totient(i64 %n) local_unnamed_addr #2 {
pre_entry:
  %n.fr = freeze i64 %n
  %cond_0.not4 = icmp slt i64 %n.fr, 4
  br i1 %cond_0.not4, label %for.set.end, label %for.set.body

for.set.body:                                     ; preds = %pre_entry, %else_lbl
  %n_18 = phi i64 [ %n_4, %else_lbl ], [ %n.fr, %pre_entry ]
  %result_17 = phi i64 [ %result_3, %else_lbl ], [ %n.fr, %pre_entry ]
  %p_15 = phi i64 [ %p_2, %else_lbl ], [ 2, %pre_entry ]
  %0 = srem i64 %n_18, %p_15
  %if_cond_0 = icmp eq i64 %0, 0
  br i1 %if_cond_0, label %while.body, label %else_lbl

while.body:                                       ; preds = %for.set.body, %while.body
  %n_23 = phi i64 [ %npdiv_0, %while.body ], [ %n_18, %for.set.body ]
  %npdiv_0 = sdiv i64 %n_23, %p_15
  %1 = srem i64 %npdiv_0, %p_15
  %while_cond_0 = icmp eq i64 %1, 0
  br i1 %while_cond_0, label %while.body, label %while.end

while.end:                                        ; preds = %while.body
  %resdiv_0 = sdiv i64 %result_17, %p_15
  %result_2 = sub i64 %result_17, %resdiv_0
  br label %else_lbl

else_lbl:                                         ; preds = %while.end, %for.set.body
  %result_3 = phi i64 [ %result_2, %while.end ], [ %result_17, %for.set.body ]
  %n_4 = phi i64 [ %npdiv_0, %while.end ], [ %n_18, %for.set.body ]
  %p_2 = add i64 %p_15, 1
  %pp_0 = mul i64 %p_2, %p_2
  %cond_0.not = icmp sgt i64 %pp_0, %n_4
  br i1 %cond_0.not, label %for.set.end, label %for.set.body

for.set.end:                                      ; preds = %else_lbl, %pre_entry
  %final_if_cond_0 = icmp sgt i64 %n_4, 1
  br i1 %final_if_cond_0, label %final_if_label, label %final_else_label

final_if_label:                                   ; preds = %for.set.end
  %resdiv_1 = sdiv i64 %result_3, %n_4
  %result_4 = sub i64 %result_3, %resdiv_1
  br label %final_else_label

final_else_label:                                 ; preds = %final_if_label, %for.set.end
  %result_5 = phi i64 [ %result_4, %final_if_label ], [ %result_3, %for.set.end ]
  ret i64 %result_5
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define dso_local i64 @__mod(i64 %a, i64 %b) local_unnamed_addr #3 {
pre_entry:
  %a.fr = freeze i64 %a
  %0 = srem i64 %a.fr, %b
  ret i64 %0
}

define dso_local noundef i32 @main(i32 %argc, ptr nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %for.set.body.i.i.preheader.i, label %2

2:                                                ; preds = %0
  %3 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.4, i32 0, i32 %1)
  tail call void @exit(i32 2)
  unreachable

for.set.body.i.i.preheader.i:                     ; preds = %0, %__orig_main.exit.i
  %loop_counter_11.i = phi i64 [ %loop_counter_2.i, %__orig_main.exit.i ], [ 10, %0 ]
  %4 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %loop_counter_11.i)
  %5 = tail call i32 @putchar(i32 10)
  br label %for.set.body.i.i.i

for.set.body.i.i.i:                               ; preds = %else_lbl.i.i.i, %for.set.body.i.i.preheader.i
  %n_18.i.i.i = phi i64 [ %n_4.i.i.i, %else_lbl.i.i.i ], [ %loop_counter_11.i, %for.set.body.i.i.preheader.i ]
  %result_17.i.i.i = phi i64 [ %result_3.i.i.i, %else_lbl.i.i.i ], [ %loop_counter_11.i, %for.set.body.i.i.preheader.i ]
  %p_15.i.i.i = phi i64 [ %p_2.i.i.i, %else_lbl.i.i.i ], [ 2, %for.set.body.i.i.preheader.i ]
  %6 = srem i64 %n_18.i.i.i, %p_15.i.i.i
  %if_cond_0.i.i.i = icmp eq i64 %6, 0
  br i1 %if_cond_0.i.i.i, label %while.body.i.i.i, label %else_lbl.i.i.i

while.body.i.i.i:                                 ; preds = %for.set.body.i.i.i, %while.body.i.i.i
  %n_23.i.i.i = phi i64 [ %npdiv_0.i.i.i, %while.body.i.i.i ], [ %n_18.i.i.i, %for.set.body.i.i.i ]
  %npdiv_0.i.i.i = sdiv i64 %n_23.i.i.i, %p_15.i.i.i
  %7 = srem i64 %npdiv_0.i.i.i, %p_15.i.i.i
  %while_cond_0.i.i.i = icmp eq i64 %7, 0
  br i1 %while_cond_0.i.i.i, label %while.body.i.i.i, label %while.end.i.i.i

while.end.i.i.i:                                  ; preds = %while.body.i.i.i
  %resdiv_0.i.i.i = sdiv i64 %result_17.i.i.i, %p_15.i.i.i
  %result_2.i.i.i = sub i64 %result_17.i.i.i, %resdiv_0.i.i.i
  br label %else_lbl.i.i.i

else_lbl.i.i.i:                                   ; preds = %while.end.i.i.i, %for.set.body.i.i.i
  %result_3.i.i.i = phi i64 [ %result_2.i.i.i, %while.end.i.i.i ], [ %result_17.i.i.i, %for.set.body.i.i.i ]
  %n_4.i.i.i = phi i64 [ %npdiv_0.i.i.i, %while.end.i.i.i ], [ %n_18.i.i.i, %for.set.body.i.i.i ]
  %p_2.i.i.i = add i64 %p_15.i.i.i, 1
  %pp_0.i.i.i = mul i64 %p_2.i.i.i, %p_2.i.i.i
  %cond_0.not.i.i.i = icmp sgt i64 %pp_0.i.i.i, %n_4.i.i.i
  br i1 %cond_0.not.i.i.i, label %for.set.end.i.i.i, label %for.set.body.i.i.i

for.set.end.i.i.i:                                ; preds = %else_lbl.i.i.i
  %final_if_cond_0.i.i.i = icmp sgt i64 %n_4.i.i.i, 1
  br i1 %final_if_cond_0.i.i.i, label %final_if_label.i.i.i, label %__orig_main.exit.i

final_if_label.i.i.i:                             ; preds = %for.set.end.i.i.i
  %resdiv_1.i.i.i = sdiv i64 %result_3.i.i.i, %n_4.i.i.i
  %result_4.i.i.i = sub i64 %result_3.i.i.i, %resdiv_1.i.i.i
  br label %__orig_main.exit.i

__orig_main.exit.i:                               ; preds = %final_if_label.i.i.i, %for.set.end.i.i.i
  %result_5.i.i.i = phi i64 [ %result_4.i.i.i, %final_if_label.i.i.i ], [ %result_3.i.i.i, %for.set.end.i.i.i ]
  %8 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %result_5.i.i.i)
  %9 = tail call i32 @putchar(i32 10)
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 1000000
  br i1 %exitcond.not.i, label %__main.exit, label %for.set.body.i.i.preheader.i

__main.exit:                                      ; preds = %__orig_main.exit.i
  ret i32 0
}

attributes #0 = { nofree nounwind }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read) }
attributes #2 = { nofree norecurse nosync nounwind memory(none) }
attributes #3 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) }
