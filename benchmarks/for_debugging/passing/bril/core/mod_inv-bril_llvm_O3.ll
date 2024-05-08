; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpYvbIKC/mod_inv-init.ll'
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
  br label %loop_body

loop_body:                                        ; preds = %b0, %loop_body
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %loop_body ]
  %0 = urem i64 %loop_counter_11, 10007
  %v26_0.i = mul nuw nsw i64 %loop_counter_11, %loop_counter_11
  %1 = urem i64 %v26_0.i, 10007
  %v26_0.1.i = mul nuw nsw i64 %1, %1
  %.lhs.trunc = trunc i64 %v26_0.1.i to i32
  %2 = urem i32 %.lhs.trunc, 10007
  %3 = trunc i64 %0 to i32
  %.lhs.trunc.i = mul nuw nsw i32 %2, %3
  %4 = urem i32 %.lhs.trunc.i, 10007
  %v26_0.2.i = mul nuw nsw i32 %2, %2
  %5 = urem i32 %v26_0.2.i, 10007
  %.zext.i = zext nneg i32 %5 to i64
  %v26_0.3.i = mul nuw nsw i64 %.zext.i, %.zext.i
  %n.fr.i2.3.i = freeze i64 %v26_0.3.i
  %6 = srem i64 %n.fr.i2.3.i, 10007
  %7 = trunc i64 %6 to i32
  %.lhs.trunc7.i = mul nsw i32 %4, %7
  %8 = srem i32 %.lhs.trunc7.i, 10007
  %.sext8.i = sext i32 %8 to i64
  %v26_0.4.i = mul nsw i64 %6, %6
  %9 = urem i64 %v26_0.4.i, 10007
  %v26_0.5.i = mul nuw nsw i64 %9, %9
  %.lhs.trunc9.i = trunc i64 %v26_0.5.i to i32
  %10 = urem i32 %.lhs.trunc9.i, 10007
  %.zext10.i = zext nneg i32 %10 to i64
  %v26_0.6.i = mul nuw nsw i64 %.zext10.i, %.zext10.i
  %n.fr.i2.6.i = freeze i64 %v26_0.6.i
  %11 = srem i64 %n.fr.i2.6.i, 10007
  %v26_0.7.i = mul nsw i64 %11, %11
  %12 = urem i64 %v26_0.7.i, 10007
  %v21_0.8.i = mul nsw i64 %12, %.sext8.i
  %n.fr.i.8.i = freeze i64 %v21_0.8.i
  %13 = srem i64 %n.fr.i.8.i, 10007
  %v26_0.8.i = mul nuw nsw i64 %12, %12
  %v26_0.8.fr.i = freeze i64 %v26_0.8.i
  %14 = urem i64 %v26_0.8.fr.i, 10007
  %v21_0.9.i = mul nsw i64 %13, %14
  %.lhs.trunc11.i = trunc i64 %v21_0.9.i to i32
  %15 = srem i32 %.lhs.trunc11.i, 10007
  %.sext12.i = sext i32 %15 to i64
  %v26_0.9.i = mul nuw nsw i64 %14, %14
  %.lhs.trunc13.i = trunc i64 %v26_0.9.i to i32
  %16 = urem i32 %.lhs.trunc13.i, 10007
  %.zext14.i = zext nneg i32 %16 to i64
  %v21_0.10.i = mul nsw i64 %.sext12.i, %.zext14.i
  %n.fr.i.10.i = freeze i64 %v21_0.10.i
  %17 = srem i64 %n.fr.i.10.i, 10007
  %v26_0.10.i = mul nuw nsw i32 %16, %16
  %18 = urem i32 %v26_0.10.i, 10007
  %v26_0.11.i = mul nuw nsw i32 %18, %18
  %19 = urem i32 %v26_0.11.i, 10007
  %.zext18.i = zext nneg i32 %19 to i64
  %v26_0.12.i = mul nuw nsw i64 %.zext18.i, %.zext18.i
  %n.fr.i2.12.i = freeze i64 %v26_0.12.i
  %20 = srem i64 %n.fr.i2.12.i, 10007
  %v21_0.13.i = mul nsw i64 %20, %17
  %.lhs.trunc19.i = trunc i64 %v21_0.13.i to i32
  %21 = srem i32 %.lhs.trunc19.i, 10007
  %.sext20.i = sext i32 %21 to i64
  %22 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %.sext20.i)
  %23 = tail call i32 @putchar(i32 10)
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 1000000
  br i1 %exitcond.not, label %loop_done, label %loop_body

loop_done:                                        ; preds = %loop_body
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %n) local_unnamed_addr #0 {
endif.11.13:
  %n.fr.i = freeze i64 %n
  %0 = srem i64 %n.fr.i, 10007
  %v26_0 = mul i64 %n.fr.i, %n.fr.i
  %1 = srem i64 %v26_0, 10007
  %v26_0.1 = mul nsw i64 %1, %1
  %2 = urem i64 %v26_0.1, 10007
  %v21_0.2 = mul nsw i64 %0, %2
  %.lhs.trunc = trunc i64 %v21_0.2 to i32
  %3 = srem i32 %.lhs.trunc, 10007
  %v26_0.2 = mul nuw nsw i64 %2, %2
  %.lhs.trunc6 = trunc i64 %v26_0.2 to i32
  %4 = urem i32 %.lhs.trunc6, 10007
  %.zext = zext nneg i32 %4 to i64
  %v26_0.3 = mul nuw nsw i64 %.zext, %.zext
  %n.fr.i2.3 = freeze i64 %v26_0.3
  %5 = srem i64 %n.fr.i2.3, 10007
  %6 = trunc i64 %5 to i32
  %.lhs.trunc7 = mul nsw i32 %3, %6
  %7 = srem i32 %.lhs.trunc7, 10007
  %.sext8 = sext i32 %7 to i64
  %v26_0.4 = mul nsw i64 %5, %5
  %8 = urem i64 %v26_0.4, 10007
  %v26_0.5 = mul nuw nsw i64 %8, %8
  %.lhs.trunc9 = trunc i64 %v26_0.5 to i32
  %9 = urem i32 %.lhs.trunc9, 10007
  %.zext10 = zext nneg i32 %9 to i64
  %v26_0.6 = mul nuw nsw i64 %.zext10, %.zext10
  %n.fr.i2.6 = freeze i64 %v26_0.6
  %10 = srem i64 %n.fr.i2.6, 10007
  %v26_0.7 = mul nsw i64 %10, %10
  %11 = urem i64 %v26_0.7, 10007
  %v21_0.8 = mul nsw i64 %11, %.sext8
  %n.fr.i.8 = freeze i64 %v21_0.8
  %12 = srem i64 %n.fr.i.8, 10007
  %v26_0.8 = mul nuw nsw i64 %11, %11
  %v26_0.8.fr = freeze i64 %v26_0.8
  %13 = urem i64 %v26_0.8.fr, 10007
  %v21_0.9 = mul nsw i64 %12, %13
  %.lhs.trunc11 = trunc i64 %v21_0.9 to i32
  %14 = srem i32 %.lhs.trunc11, 10007
  %.sext12 = sext i32 %14 to i64
  %v26_0.9 = mul nuw nsw i64 %13, %13
  %.lhs.trunc13 = trunc i64 %v26_0.9 to i32
  %15 = urem i32 %.lhs.trunc13, 10007
  %.zext14 = zext nneg i32 %15 to i64
  %v21_0.10 = mul nsw i64 %.sext12, %.zext14
  %n.fr.i.10 = freeze i64 %v21_0.10
  %16 = srem i64 %n.fr.i.10, 10007
  %v26_0.10 = mul nuw nsw i32 %15, %15
  %17 = urem i32 %v26_0.10, 10007
  %v26_0.11 = mul nuw nsw i32 %17, %17
  %18 = urem i32 %v26_0.11, 10007
  %.zext18 = zext nneg i32 %18 to i64
  %v26_0.12 = mul nuw nsw i64 %.zext18, %.zext18
  %n.fr.i2.12 = freeze i64 %v26_0.12
  %19 = srem i64 %n.fr.i2.12, 10007
  %v21_0.13 = mul nsw i64 %16, %19
  %.lhs.trunc19 = trunc i64 %v21_0.13 to i32
  %20 = srem i32 %.lhs.trunc19, 10007
  %.sext20 = sext i32 %20 to i64
  %21 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %.sext20)
  %22 = tail call i32 @putchar(i32 10)
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define dso_local i64 @__mod(i64 %n, i64 %p) local_unnamed_addr #2 {
pre_entry:
  %n.fr = freeze i64 %n
  %0 = srem i64 %n.fr, %p
  ret i64 %0
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
attributes #2 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) }
