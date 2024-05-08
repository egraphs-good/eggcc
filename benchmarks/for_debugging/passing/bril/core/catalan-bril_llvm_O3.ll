; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpIaImf2/catalan-init.ll'
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
  %catn_0.i = tail call i64 @__catalan(i64 10)
  %0 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %catn_0.i)
  %1 = tail call i32 @putchar(i32 10)
  %catn_0.i.1 = tail call i64 @__catalan(i64 11)
  %2 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %catn_0.i.1)
  %3 = tail call i32 @putchar(i32 10)
  %catn_0.i.2 = tail call i64 @__catalan(i64 12)
  %4 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %catn_0.i.2)
  %5 = tail call i32 @putchar(i32 10)
  %catn_0.i.3 = tail call i64 @__catalan(i64 13)
  %6 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %catn_0.i.3)
  %7 = tail call i32 @putchar(i32 10)
  %catn_0.i.4 = tail call i64 @__catalan(i64 14)
  %8 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %catn_0.i.4)
  %9 = tail call i32 @putchar(i32 10)
  %catn_0.i.5 = tail call i64 @__catalan(i64 15)
  %10 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %catn_0.i.5)
  %11 = tail call i32 @putchar(i32 10)
  %catn_0.i.6 = tail call i64 @__catalan(i64 16)
  %12 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %catn_0.i.6)
  %13 = tail call i32 @putchar(i32 10)
  %catn_0.i.7 = tail call i64 @__catalan(i64 17)
  %14 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %catn_0.i.7)
  %15 = tail call i32 @putchar(i32 10)
  %catn_0.i.8 = tail call i64 @__catalan(i64 18)
  %16 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %catn_0.i.8)
  %17 = tail call i32 @putchar(i32 10)
  %catn_0.i.9 = tail call i64 @__catalan(i64 19)
  %18 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %catn_0.i.9)
  %19 = tail call i32 @putchar(i32 10)
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %input) local_unnamed_addr #0 {
pre_entry:
  %catn_0 = tail call i64 @__catalan(i64 %input)
  %0 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %catn_0)
  %1 = tail call i32 @putchar(i32 10)
  ret void
}

; Function Attrs: nofree nosync nounwind memory(none)
define dso_local i64 @__catalan(i64 %n) local_unnamed_addr #2 {
pre_entry:
  %guard0_0 = icmp eq i64 %n, 0
  br i1 %guard0_0, label %common.ret, label %else

common.ret:                                       ; preds = %while.body, %else, %pre_entry
  %common.ret.op = phi i64 [ 1, %pre_entry ], [ 0, %else ], [ %sum_2, %while.body ]
  ret i64 %common.ret.op

else:                                             ; preds = %pre_entry
  %n_1 = add i64 %n, -1
  %guard1_0.not1 = icmp slt i64 %n_1, 0
  br i1 %guard1_0.not1, label %common.ret, label %while.body

while.body:                                       ; preds = %else, %while.body
  %sum_13 = phi i64 [ %sum_2, %while.body ], [ 0, %else ]
  %idx_12 = phi i64 [ %idx_2, %while.body ], [ 0, %else ]
  %n2_0 = sub i64 %n_1, %idx_12
  %v1_0 = tail call i64 @__catalan(i64 %idx_12)
  %v2_0 = tail call i64 @__catalan(i64 %n2_0)
  %elti_0 = mul i64 %v2_0, %v1_0
  %sum_2 = add i64 %elti_0, %sum_13
  %idx_2 = add i64 %idx_12, 1
  %guard1_0.not = icmp sgt i64 %idx_2, %n_1
  br i1 %guard1_0.not, label %common.ret, label %while.body
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
