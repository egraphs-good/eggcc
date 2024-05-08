; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmp3UhcT5/euclid-init.ll'
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
  br label %inner_cond.preheader

inner_cond.preheader:                             ; preds = %b0, %inner_done
  %loop_counter_12 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %inner_done ]
  br label %for.body.5.i.i.preheader

for.body.5.i.i.preheader:                         ; preds = %inner_cond.preheader, %__orig_main.exit
  %inner_counter_11 = phi i64 [ 10, %inner_cond.preheader ], [ %inner_counter_2, %__orig_main.exit ]
  br label %for.body.5.i.i

for.body.5.i.i:                                   ; preds = %for.body.5.i.i.preheader, %for.body.5.i.i
  %r.fr.i4.i.i = phi i64 [ %b_13.i.i, %for.body.5.i.i ], [ %loop_counter_12, %for.body.5.i.i.preheader ]
  %b_13.i.i = phi i64 [ %0, %for.body.5.i.i ], [ %inner_counter_11, %for.body.5.i.i.preheader ]
  %0 = srem i64 %r.fr.i4.i.i, %b_13.i.i
  %cond_1.not.i.i = icmp eq i64 %0, 0
  br i1 %cond_1.not.i.i, label %__orig_main.exit, label %for.body.5.i.i

__orig_main.exit:                                 ; preds = %for.body.5.i.i
  %1 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %b_13.i.i)
  %2 = tail call i32 @putchar(i32 10)
  %inner_counter_2 = add nuw nsw i64 %inner_counter_11, 1
  %exitcond.not = icmp eq i64 %inner_counter_2, 1000
  br i1 %exitcond.not, label %inner_done, label %for.body.5.i.i.preheader

inner_done:                                       ; preds = %__orig_main.exit
  %loop_counter_2 = add nuw nsw i64 %loop_counter_12, 1
  %exitcond3.not = icmp eq i64 %loop_counter_2, 1000
  br i1 %exitcond3.not, label %loop_done, label %inner_cond.preheader

loop_done:                                        ; preds = %inner_done
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %v0, i64 %v1) local_unnamed_addr #0 {
pre_entry:
  %b.fr.i = freeze i64 %v1
  %r.fr.i1.i = freeze i64 %v0
  %cond_1.not2.i = icmp eq i64 %b.fr.i, 0
  br i1 %cond_1.not2.i, label %__gcd.exit, label %for.body.5.i

for.body.5.i:                                     ; preds = %pre_entry, %for.body.5.i
  %r.fr.i4.i = phi i64 [ %b_13.i, %for.body.5.i ], [ %r.fr.i1.i, %pre_entry ]
  %b_13.i = phi i64 [ %0, %for.body.5.i ], [ %b.fr.i, %pre_entry ]
  %0 = srem i64 %r.fr.i4.i, %b_13.i
  %cond_1.not.i = icmp eq i64 %0, 0
  br i1 %cond_1.not.i, label %__gcd.exit, label %for.body.5.i

__gcd.exit:                                       ; preds = %for.body.5.i, %pre_entry
  %r.fr.i.lcssa.i = phi i64 [ %r.fr.i1.i, %pre_entry ], [ %b_13.i, %for.body.5.i ]
  %1 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %r.fr.i.lcssa.i)
  %2 = tail call i32 @putchar(i32 10)
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define dso_local i64 @__mod(i64 %r, i64 %s) local_unnamed_addr #2 {
pre_entry:
  %r.fr = freeze i64 %r
  %0 = srem i64 %r.fr, %s
  ret i64 %0
}

; Function Attrs: nofree norecurse nosync nounwind memory(none)
define dso_local i64 @__gcd(i64 %a, i64 %b) local_unnamed_addr #3 {
pre_entry:
  %b.fr = freeze i64 %b
  %r.fr.i1 = freeze i64 %a
  %cond_1.not2 = icmp eq i64 %b.fr, 0
  br i1 %cond_1.not2, label %for.end.5, label %for.body.5

for.body.5:                                       ; preds = %pre_entry, %for.body.5
  %r.fr.i4 = phi i64 [ %b_13, %for.body.5 ], [ %r.fr.i1, %pre_entry ]
  %b_13 = phi i64 [ %0, %for.body.5 ], [ %b.fr, %pre_entry ]
  %0 = srem i64 %r.fr.i4, %b_13
  %cond_1.not = icmp eq i64 %0, 0
  br i1 %cond_1.not, label %for.end.5, label %for.body.5

for.end.5:                                        ; preds = %for.body.5, %pre_entry
  %r.fr.i.lcssa = phi i64 [ %r.fr.i1, %pre_entry ], [ %b_13, %for.body.5 ]
  ret i64 %r.fr.i.lcssa
}

define dso_local noundef i32 @main(i32 %argc, ptr nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %inner_cond.preheader.i, label %2

2:                                                ; preds = %0
  %3 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.4, i32 0, i32 %1)
  tail call void @exit(i32 2)
  unreachable

inner_cond.preheader.i:                           ; preds = %0, %inner_done.i
  %loop_counter_12.i = phi i64 [ %loop_counter_2.i, %inner_done.i ], [ 10, %0 ]
  br label %for.body.5.i.i.preheader.i

for.body.5.i.i.preheader.i:                       ; preds = %__orig_main.exit.i, %inner_cond.preheader.i
  %inner_counter_11.i = phi i64 [ 10, %inner_cond.preheader.i ], [ %inner_counter_2.i, %__orig_main.exit.i ]
  br label %for.body.5.i.i.i

for.body.5.i.i.i:                                 ; preds = %for.body.5.i.i.i, %for.body.5.i.i.preheader.i
  %r.fr.i4.i.i.i = phi i64 [ %b_13.i.i.i, %for.body.5.i.i.i ], [ %loop_counter_12.i, %for.body.5.i.i.preheader.i ]
  %b_13.i.i.i = phi i64 [ %4, %for.body.5.i.i.i ], [ %inner_counter_11.i, %for.body.5.i.i.preheader.i ]
  %4 = srem i64 %r.fr.i4.i.i.i, %b_13.i.i.i
  %cond_1.not.i.i.i = icmp eq i64 %4, 0
  br i1 %cond_1.not.i.i.i, label %__orig_main.exit.i, label %for.body.5.i.i.i

__orig_main.exit.i:                               ; preds = %for.body.5.i.i.i
  %5 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %b_13.i.i.i)
  %6 = tail call i32 @putchar(i32 10)
  %inner_counter_2.i = add nuw nsw i64 %inner_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %inner_counter_2.i, 1000
  br i1 %exitcond.not.i, label %inner_done.i, label %for.body.5.i.i.preheader.i

inner_done.i:                                     ; preds = %__orig_main.exit.i
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_12.i, 1
  %exitcond3.not.i = icmp eq i64 %loop_counter_2.i, 1000
  br i1 %exitcond3.not.i, label %__main.exit, label %inner_cond.preheader.i

__main.exit:                                      ; preds = %inner_done.i
  ret i32 0
}

attributes #0 = { nofree nounwind }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read) }
attributes #2 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) }
attributes #3 = { nofree norecurse nosync nounwind memory(none) }
