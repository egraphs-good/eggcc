; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmp74uZmT/lcm-init.ll'
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
  br label %inner_body

inner_body:                                       ; preds = %inner_cond.preheader, %__orig_main.exit
  %inner_counter_11 = phi i64 [ 10, %inner_cond.preheader ], [ %inner_counter_2, %__orig_main.exit ]
  %spec.select.i = tail call i64 @llvm.smax.i64(i64 %loop_counter_12, i64 %inner_counter_11)
  %spec.select.fr.i = freeze i64 %spec.select.i
  br label %else.1.i

else.1.i:                                         ; preds = %else.1.i, %inner_body
  %greater_2.i = phi i64 [ %greater_3.i, %else.1.i ], [ %spec.select.fr.i, %inner_body ]
  %0 = srem i64 %greater_2.i, %loop_counter_12
  %1 = srem i64 %greater_2.i, %inner_counter_11
  %2 = or i64 %1, %0
  %bothZero_0.i = icmp eq i64 %2, 0
  %greater_3.i = add i64 %greater_2.i, 1
  br i1 %bothZero_0.i, label %__orig_main.exit, label %else.1.i

__orig_main.exit:                                 ; preds = %else.1.i
  %3 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %greater_2.i)
  %4 = tail call i32 @putchar(i32 10)
  %inner_counter_2 = add nuw nsw i64 %inner_counter_11, 1
  %exitcond.not = icmp eq i64 %inner_counter_2, 250
  br i1 %exitcond.not, label %inner_done, label %inner_body

inner_done:                                       ; preds = %__orig_main.exit
  %loop_counter_2 = add nuw nsw i64 %loop_counter_12, 1
  %exitcond3.not = icmp eq i64 %loop_counter_2, 250
  br i1 %exitcond3.not, label %loop_done, label %inner_cond.preheader

loop_done:                                        ; preds = %inner_done
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %x, i64 %y) local_unnamed_addr #0 {
pre_entry:
  %spec.select = tail call i64 @llvm.smax.i64(i64 %x, i64 %y)
  %spec.select.fr = freeze i64 %spec.select
  br label %else.1

else.1:                                           ; preds = %else.1, %pre_entry
  %greater_2 = phi i64 [ %greater_3, %else.1 ], [ %spec.select.fr, %pre_entry ]
  %0 = srem i64 %greater_2, %x
  %1 = srem i64 %greater_2, %y
  %2 = or i64 %1, %0
  %bothZero_0 = icmp eq i64 %2, 0
  %greater_3 = add i64 %greater_2, 1
  br i1 %bothZero_0, label %then.2, label %else.1

then.2:                                           ; preds = %else.1
  %3 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %greater_2)
  %4 = tail call i32 @putchar(i32 10)
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define dso_local i64 @__getMod(i64 %val, i64 %mod) local_unnamed_addr #2 {
pre_entry:
  %val.fr = freeze i64 %val
  %0 = srem i64 %val.fr, %mod
  ret i64 %0
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
  br label %inner_body.i

inner_body.i:                                     ; preds = %__orig_main.exit.i, %inner_cond.preheader.i
  %inner_counter_11.i = phi i64 [ 10, %inner_cond.preheader.i ], [ %inner_counter_2.i, %__orig_main.exit.i ]
  %spec.select.i.i = tail call i64 @llvm.smax.i64(i64 %loop_counter_12.i, i64 %inner_counter_11.i)
  %spec.select.fr.i.i = freeze i64 %spec.select.i.i
  br label %else.1.i.i

else.1.i.i:                                       ; preds = %else.1.i.i, %inner_body.i
  %greater_2.i.i = phi i64 [ %greater_3.i.i, %else.1.i.i ], [ %spec.select.fr.i.i, %inner_body.i ]
  %4 = srem i64 %greater_2.i.i, %loop_counter_12.i
  %5 = srem i64 %greater_2.i.i, %inner_counter_11.i
  %6 = or i64 %5, %4
  %bothZero_0.i.i = icmp eq i64 %6, 0
  %greater_3.i.i = add i64 %greater_2.i.i, 1
  br i1 %bothZero_0.i.i, label %__orig_main.exit.i, label %else.1.i.i

__orig_main.exit.i:                               ; preds = %else.1.i.i
  %7 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %greater_2.i.i)
  %8 = tail call i32 @putchar(i32 10)
  %inner_counter_2.i = add nuw nsw i64 %inner_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %inner_counter_2.i, 250
  br i1 %exitcond.not.i, label %inner_done.i, label %inner_body.i

inner_done.i:                                     ; preds = %__orig_main.exit.i
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_12.i, 1
  %exitcond3.not.i = icmp eq i64 %loop_counter_2.i, 250
  br i1 %exitcond3.not.i, label %__main.exit, label %inner_cond.preheader.i

__main.exit:                                      ; preds = %inner_done.i
  ret i32 0
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.smax.i64(i64, i64) #3

attributes #0 = { nofree nounwind }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read) }
attributes #2 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) }
attributes #3 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
