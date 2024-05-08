; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmp4kQt8B/reverse-init.ll'
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
  br label %for.body.3.i.preheader

for.body.3.i.preheader:                           ; preds = %b0, %__orig_main.exit
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %__orig_main.exit ]
  br label %for.body.3.i

for.body.3.i:                                     ; preds = %for.body.3.i.preheader, %for.body.3.i
  %n_12.i = phi i64 [ %a_0.i, %for.body.3.i ], [ %loop_counter_11, %for.body.3.i.preheader ]
  %result_11.i = phi i64 [ %result_3.i, %for.body.3.i ], [ 0, %for.body.3.i.preheader ]
  %a_0.i = sdiv i64 %n_12.i, 10
  %reass.add.i = sub i64 %result_11.i, %a_0.i
  %reass.mul.i = mul i64 %reass.add.i, 10
  %result_3.i = add i64 %reass.mul.i, %n_12.i
  %0 = add nsw i64 %n_12.i, -10
  %comp1_0.i = icmp ult i64 %0, -19
  br i1 %comp1_0.i, label %for.body.3.i, label %__orig_main.exit

__orig_main.exit:                                 ; preds = %for.body.3.i
  %1 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %result_3.i)
  %2 = tail call i32 @putchar(i32 10)
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 1000000
  br i1 %exitcond.not, label %loop_done, label %for.body.3.i.preheader

loop_done:                                        ; preds = %__orig_main.exit
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %input) local_unnamed_addr #0 {
pre_entry:
  br label %for.body.3

for.body.3:                                       ; preds = %pre_entry, %for.body.3
  %n_12 = phi i64 [ %input, %pre_entry ], [ %a_0, %for.body.3 ]
  %result_11 = phi i64 [ 0, %pre_entry ], [ %result_3, %for.body.3 ]
  %a_0 = sdiv i64 %n_12, 10
  %reass.add = sub i64 %result_11, %a_0
  %reass.mul = mul i64 %reass.add, 10
  %result_3 = add i64 %reass.mul, %n_12
  %0 = add i64 %n_12, -10
  %comp1_0 = icmp ult i64 %0, -19
  br i1 %comp1_0, label %for.body.3, label %for.end.3

for.end.3:                                        ; preds = %for.body.3
  %1 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %result_3)
  %2 = tail call i32 @putchar(i32 10)
  ret void
}

define dso_local noundef i32 @main(i32 %argc, ptr nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %for.body.3.i.preheader.i, label %2

2:                                                ; preds = %0
  %3 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.4, i32 0, i32 %1)
  tail call void @exit(i32 2)
  unreachable

for.body.3.i.preheader.i:                         ; preds = %0, %__orig_main.exit.i
  %loop_counter_11.i = phi i64 [ %loop_counter_2.i, %__orig_main.exit.i ], [ 10, %0 ]
  br label %for.body.3.i.i

for.body.3.i.i:                                   ; preds = %for.body.3.i.i, %for.body.3.i.preheader.i
  %n_12.i.i = phi i64 [ %a_0.i.i, %for.body.3.i.i ], [ %loop_counter_11.i, %for.body.3.i.preheader.i ]
  %result_11.i.i = phi i64 [ %result_3.i.i, %for.body.3.i.i ], [ 0, %for.body.3.i.preheader.i ]
  %a_0.i.i = sdiv i64 %n_12.i.i, 10
  %reass.add.i.i = sub i64 %result_11.i.i, %a_0.i.i
  %reass.mul.i.i = mul i64 %reass.add.i.i, 10
  %result_3.i.i = add i64 %reass.mul.i.i, %n_12.i.i
  %4 = add nsw i64 %n_12.i.i, -10
  %comp1_0.i.i = icmp ult i64 %4, -19
  br i1 %comp1_0.i.i, label %for.body.3.i.i, label %__orig_main.exit.i

__orig_main.exit.i:                               ; preds = %for.body.3.i.i
  %5 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %result_3.i.i)
  %6 = tail call i32 @putchar(i32 10)
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 1000000
  br i1 %exitcond.not.i, label %__main.exit, label %for.body.3.i.preheader.i

__main.exit:                                      ; preds = %__orig_main.exit.i
  ret i32 0
}

attributes #0 = { nofree nounwind }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read) }
