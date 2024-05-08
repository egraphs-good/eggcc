; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpV5UBrP/sum-divisors-init.ll'
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
define dso_local void @__orig_main(i64 %n) local_unnamed_addr #0 {
pre_entry:
  %n.fr = freeze i64 %n
  %spec.select = tail call i64 @llvm.abs.i64(i64 %n.fr, i1 false)
  br label %begin.outer

begin.outer:                                      ; preds = %begin.outer.backedge, %pre_entry
  %res_1.ph = phi i64 [ 0, %pre_entry ], [ %res_1.ph.be, %begin.outer.backedge ]
  %i_1.ph = phi i64 [ 0, %pre_entry ], [ %i_2, %begin.outer.backedge ]
  br label %begin

begin:                                            ; preds = %begin.outer, %check
  %i_1 = phi i64 [ %i_2, %check ], [ %i_1.ph, %begin.outer ]
  %i_2 = add i64 %i_1, 1
  %isq_0 = mul i64 %i_2, %i_2
  %sqgt_0 = icmp sgt i64 %isq_0, %spec.select
  br i1 %sqgt_0, label %end, label %check

check:                                            ; preds = %begin
  %i_2.frozen = freeze i64 %i_2
  %d_0 = sdiv i64 %spec.select, %i_2.frozen
  %0 = mul i64 %d_0, %i_2.frozen
  %.decomposed = sub i64 %spec.select, %0
  %eqz_0 = icmp eq i64 %.decomposed, 0
  br i1 %eqz_0, label %body, label %begin

body:                                             ; preds = %check
  %1 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %i_2)
  %2 = tail call i32 @putchar(i32 10)
  %res_2 = add i64 %i_2, %res_1.ph
  %deqi_0 = icmp eq i64 %d_0, %i_2
  br i1 %deqi_0, label %begin.outer.backedge, label %then

then:                                             ; preds = %body
  %3 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %d_0)
  %4 = tail call i32 @putchar(i32 10)
  %res_3 = add i64 %d_0, %res_2
  br label %begin.outer.backedge

begin.outer.backedge:                             ; preds = %then, %body
  %res_1.ph.be = phi i64 [ %res_2, %body ], [ %res_3, %then ]
  br label %begin.outer

end:                                              ; preds = %begin
  %5 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %res_1.ph)
  %6 = tail call i32 @putchar(i32 10)
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define dso_local i64 @__mod(i64 %dividend, i64 %divisor) local_unnamed_addr #2 {
pre_entry:
  %dividend.fr = freeze i64 %dividend
  %0 = srem i64 %dividend.fr, %divisor
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

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.abs.i64(i64, i1 immarg) #3

attributes #0 = { nofree nounwind }
attributes #1 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read) }
attributes #2 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) }
attributes #3 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
