; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpQ5iVKH/digital-root-init.ll'
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
  tail call void @__orig_main(i64 %loop_counter_11)
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 100000
  br i1 %exitcond.not, label %loop_done, label %loop_body

loop_done:                                        ; preds = %loop_body
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %input) local_unnamed_addr #0 {
pre_entry:
  %input.fr = freeze i64 %input
  br label %begin

begin:                                            ; preds = %check_done, %pre_entry
  %result_1 = phi i64 [ %result_3.lcssa, %check_done ], [ 0, %pre_entry ]
  %input_1 = phi i64 [ %input_2, %check_done ], [ %input.fr, %pre_entry ]
  %input_1.frozen = freeze i64 %input_1
  %input_2 = sdiv i64 %input_1.frozen, 10
  %0 = mul i64 %input_2, 10
  %.decomposed = sub i64 %input_1.frozen, %0
  %result_2 = add nsw i64 %.decomposed, %result_1
  %result_2.fr = freeze i64 %result_2
  %1 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %result_2.fr)
  %2 = tail call i32 @putchar(i32 10)
  %3 = srem i64 %result_2.fr, 10
  %result_0.i3 = icmp eq i64 %result_2.fr, %3
  br i1 %result_0.i3, label %check_done, label %process_result

process_result:                                   ; preds = %begin, %process_result
  %4 = phi i64 [ %7, %process_result ], [ %3, %begin ]
  %result_34 = phi i64 [ %result_5, %process_result ], [ %result_2.fr, %begin ]
  %result_4 = sdiv i64 %result_34, 10
  %result_5 = add nsw i64 %result_4, %4
  %5 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %result_5)
  %6 = tail call i32 @putchar(i32 10)
  %7 = srem i64 %result_5, 10
  %result_0.i = icmp eq i64 %result_5, %7
  br i1 %result_0.i, label %check_done, label %process_result

check_done:                                       ; preds = %process_result, %begin
  %result_3.lcssa = phi i64 [ %result_2.fr, %begin ], [ %result_5, %process_result ]
  %input_1.off = add i64 %input_1, 9
  %done_0 = icmp ult i64 %input_1.off, 19
  br i1 %done_0, label %done, label %begin

done:                                             ; preds = %check_done
  %8 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %result_3.lcssa)
  %9 = tail call i32 @putchar(i32 10)
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define dso_local noundef i1 @__is_single_digit(i64 %input) local_unnamed_addr #2 {
pre_entry:
  %input.fr = freeze i64 %input
  %0 = srem i64 %input.fr, 10
  %result_0 = icmp eq i64 %input.fr, %0
  ret i1 %result_0
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define dso_local noundef i64 @__peel_last_digit(i64 %input) local_unnamed_addr #2 {
pre_entry:
  %input.fr = freeze i64 %input
  %0 = srem i64 %input.fr, 10
  ret i64 %0
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
  tail call void @__orig_main(i64 %loop_counter_11.i)
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 100000
  br i1 %exitcond.not.i, label %__main.exit, label %loop_body.i

__main.exit:                                      ; preds = %loop_body.i
  ret i32 0
}

attributes #0 = { nofree nounwind }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read) }
attributes #2 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) }
