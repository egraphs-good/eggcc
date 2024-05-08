; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpYMtDpL/bitshift-init.ll'
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
define dso_local i64 @__pow(i64 %x, i64 %n) local_unnamed_addr #2 {
pre_entry:
  %a.fr.i = freeze i64 %n
  %v3_0 = icmp eq i64 %a.fr.i, 1
  br i1 %v3_0, label %common.ret, label %else.0

common.ret:                                       ; preds = %pre_entry, %else.0
  %common.ret.op = phi i64 [ %ans_2, %else.0 ], [ %x, %pre_entry ]
  ret i64 %common.ret.op

else.0:                                           ; preds = %pre_entry
  %v8_0 = sdiv i64 %a.fr.i, 2
  %half_0 = tail call i64 @__pow(i64 %x, i64 %v8_0)
  %0 = and i64 %a.fr.i, -9223372036854775807
  %v17_0 = icmp eq i64 %0, 1
  %v20_0 = select i1 %v17_0, i64 %x, i64 1
  %v11_0 = mul i64 %half_0, %v20_0
  %ans_2 = mul i64 %v11_0, %half_0
  br label %common.ret
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define dso_local i64 @__mod(i64 %a, i64 %b) local_unnamed_addr #3 {
pre_entry:
  %a.fr = freeze i64 %a
  %0 = srem i64 %a.fr, %b
  ret i64 %0
}

; Function Attrs: nofree nosync nounwind memory(none)
define dso_local i64 @__LEFTSHIFT(i64 %x, i64 %step) local_unnamed_addr #2 {
pre_entry:
  %p_0 = tail call i64 @__pow(i64 2, i64 %step)
  %v4_0 = mul i64 %p_0, %x
  ret i64 %v4_0
}

; Function Attrs: nofree nosync nounwind memory(none)
define dso_local i64 @__RIGHTSHIFT(i64 %x, i64 %step) local_unnamed_addr #2 {
pre_entry:
  %p_0 = tail call i64 @__pow(i64 2, i64 %step)
  %v4_0 = sdiv i64 %x, %p_0
  ret i64 %v4_0
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
  %p_0.i.i = tail call i64 @__pow(i64 2, i64 %loop2_counter_13)
  %v4_0.i.i = mul i64 %p_0.i.i, %loop_counter_14
  br label %loop4_cond.preheader

loop4_cond.preheader:                             ; preds = %loop3_cond.preheader, %loop4_done
  %loop3_counter_12 = phi i64 [ 10, %loop3_cond.preheader ], [ %loop3_counter_2, %loop4_done ]
  br label %loop4_body

loop4_body:                                       ; preds = %loop4_cond.preheader, %loop4_body
  %loop4_counter_11 = phi i64 [ 10, %loop4_cond.preheader ], [ %loop4_counter_2, %loop4_body ]
  %0 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %v4_0.i.i)
  %1 = tail call i32 @putchar(i32 10)
  %p_0.i1.i = tail call i64 @__pow(i64 2, i64 %loop4_counter_11)
  %v4_0.i2.i = sdiv i64 %loop3_counter_12, %p_0.i1.i
  %2 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %v4_0.i2.i)
  %3 = tail call i32 @putchar(i32 10)
  %loop4_counter_2 = add nuw nsw i64 %loop4_counter_11, 1
  %exitcond.not = icmp eq i64 %loop4_counter_2, 40
  br i1 %exitcond.not, label %loop4_done, label %loop4_body

loop4_done:                                       ; preds = %loop4_body
  %loop3_counter_2 = add nuw nsw i64 %loop3_counter_12, 1
  %exitcond5.not = icmp eq i64 %loop3_counter_2, 40
  br i1 %exitcond5.not, label %loop3_done, label %loop4_cond.preheader

loop3_done:                                       ; preds = %loop4_done
  %loop2_counter_2 = add nuw nsw i64 %loop2_counter_13, 1
  %exitcond6.not = icmp eq i64 %loop2_counter_2, 40
  br i1 %exitcond6.not, label %loop2_done, label %loop3_cond.preheader

loop2_done:                                       ; preds = %loop3_done
  %loop_counter_2 = add nuw nsw i64 %loop_counter_14, 1
  %exitcond7.not = icmp eq i64 %loop_counter_2, 40
  br i1 %exitcond7.not, label %loop_done, label %loop2_cond.preheader

loop_done:                                        ; preds = %loop2_done
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %a, i64 %b, i64 %c, i64 %d) local_unnamed_addr #0 {
pre_entry:
  %p_0.i = tail call i64 @__pow(i64 2, i64 %b)
  %v4_0.i = mul i64 %p_0.i, %a
  %0 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %v4_0.i)
  %1 = tail call i32 @putchar(i32 10)
  %p_0.i1 = tail call i64 @__pow(i64 2, i64 %d)
  %v4_0.i2 = sdiv i64 %c, %p_0.i1
  %2 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %v4_0.i2)
  %3 = tail call i32 @putchar(i32 10)
  ret void
}

define dso_local noundef i32 @main(i32 %argc, ptr nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %4, label %2

2:                                                ; preds = %0
  %3 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.4, i32 0, i32 %1)
  tail call void @exit(i32 2)
  unreachable

4:                                                ; preds = %0
  tail call void @__main()
  ret i32 0
}

attributes #0 = { nofree nounwind }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read) }
attributes #2 = { nofree nosync nounwind memory(none) }
attributes #3 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) }
