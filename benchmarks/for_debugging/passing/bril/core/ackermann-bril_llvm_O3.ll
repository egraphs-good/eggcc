; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpIPkWjd/ackermann-init.ll'
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

; Function Attrs: nofree nosync nounwind memory(none)
define dso_local i64 @__ack(i64 %m, i64 %n) local_unnamed_addr #2 {
pre_entry:
  %cond_m_01 = icmp eq i64 %m, 0
  br i1 %cond_m_01, label %m_zero, label %m_nonzero

m_zero:                                           ; preds = %tailrecurse.backedge, %pre_entry
  %n.tr.lcssa = phi i64 [ %n, %pre_entry ], [ %n.tr.be, %tailrecurse.backedge ]
  %tmp_0 = add i64 %n.tr.lcssa, 1
  ret i64 %tmp_0

m_nonzero:                                        ; preds = %pre_entry, %tailrecurse.backedge
  %n.tr3 = phi i64 [ %n.tr.be, %tailrecurse.backedge ], [ %n, %pre_entry ]
  %m.tr2 = phi i64 [ %m.tr.be, %tailrecurse.backedge ], [ %m, %pre_entry ]
  %cond_n_0 = icmp eq i64 %n.tr3, 0
  br i1 %cond_n_0, label %tailrecurse.backedge, label %n_nonzero

tailrecurse.backedge:                             ; preds = %m_nonzero, %n_nonzero
  %n.tr.be = phi i64 [ %t1_0, %n_nonzero ], [ 1, %m_nonzero ]
  %m.tr.be = add i64 %m.tr2, -1
  %cond_m_0 = icmp eq i64 %m.tr.be, 0
  br i1 %cond_m_0, label %m_zero, label %m_nonzero

n_nonzero:                                        ; preds = %m_nonzero
  %n1_0 = add i64 %n.tr3, -1
  %t1_0 = tail call i64 @__ack(i64 %m.tr2, i64 %n1_0)
  br label %tailrecurse.backedge
}

; Function Attrs: nofree nounwind
define dso_local void @__main() local_unnamed_addr #0 {
b0:
  br label %loop_body

loop_body:                                        ; preds = %b0, %loop_body
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %loop_body ]
  %tmp_0.i = tail call i64 @__ack(i64 2, i64 %loop_counter_11)
  %0 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %tmp_0.i)
  %1 = tail call i32 @putchar(i32 10)
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 1000
  br i1 %exitcond.not, label %loop_done, label %loop_body

loop_done:                                        ; preds = %loop_body
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %n) local_unnamed_addr #0 {
pre_entry:
  %tmp_0 = tail call i64 @__ack(i64 2, i64 %n)
  %0 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %tmp_0)
  %1 = tail call i32 @putchar(i32 10)
  ret void
}

define dso_local noundef i32 @main(i32 %argc, ptr nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %loop_body.i, label %2

2:                                                ; preds = %0
  %3 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.4, i32 0, i32 %1)
  tail call void @exit(i32 2)
  unreachable

loop_body.i:                                      ; preds = %0, %loop_body.i
  %loop_counter_11.i = phi i64 [ %loop_counter_2.i, %loop_body.i ], [ 10, %0 ]
  %tmp_0.i.i = tail call i64 @__ack(i64 2, i64 %loop_counter_11.i)
  %4 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %tmp_0.i.i)
  %5 = tail call i32 @putchar(i32 10)
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 1000
  br i1 %exitcond.not.i, label %__main.exit, label %loop_body.i

__main.exit:                                      ; preds = %loop_body.i
  ret i32 0
}

attributes #0 = { nofree nounwind }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read) }
attributes #2 = { nofree nosync nounwind memory(none) }
