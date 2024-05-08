; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpV41FR4/postprocessed.ll'
source_filename = "stdin"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"
target triple = "arm64-apple-macosx13.0.0"

@.str = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.str.1 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@.str.2 = private unnamed_addr constant [4 x i8] c"%ld\00", align 1
@.str.3 = private unnamed_addr constant [9 x i8] c"[object]\00", align 1
@.str.4 = private unnamed_addr constant [33 x i8] c"error: expected %d args, got %d\0A\00", align 1

declare dso_local i32 @putchar(i32)

declare dso_local i32 @printf(ptr, ...)

declare dso_local void @exit(i32)

declare dso_local i64 @atol(ptr)

declare dso_local noalias ptr @malloc(i64)

declare dso_local void @free(ptr)

define dso_local i32 @btoi(ptr %0) {
  %2 = load i8, ptr %0, align 1
  %3 = icmp eq i8 %2, 116
  %4 = zext i1 %3 to i32
  ret i32 %4
}

define dso_local void @print_bool(i1 %0) {
  br i1 %0, label %2, label %4

2:                                                ; preds = %1
  %3 = call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str)
  br label %6

4:                                                ; preds = %1
  %5 = call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str.1)
  br label %6

6:                                                ; preds = %4, %2
  ret void
}

define dso_local void @print_space() {
  %1 = call i32 @putchar(i32 32)
  ret void
}

define dso_local void @print_newline() {
  %1 = call i32 @putchar(i32 10)
  ret void
}

define dso_local void @print_int(i64 %0) {
  %2 = call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str.2, i64 %0)
  ret void
}

define dso_local void @print_ptr(ptr %0) {
  %2 = call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str.3)
  ret void
}

define dso_local void @__main() {
b0:
  br label %loop_cond

loop_cond:                                        ; preds = %loop_body, %b0
  %loop_counter_1 = phi i64 [ %loop_counter_2, %loop_body ], [ 10, %b0 ]
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 100000
  br i1 %loop_cond_0, label %loop_body, label %loop_done

loop_body:                                        ; preds = %loop_cond
  call void @__orig_main(i64 %loop_counter_1)
  %loop_counter_2 = add i64 %loop_counter_1, 1
  br label %loop_cond

loop_done:                                        ; preds = %loop_cond
  ret void
}

define dso_local void @__orig_main(i64 %n) {
pre_entry:
  %nlz_0 = icmp slt i64 %n, 0
  br i1 %nlz_0, label %neg, label %begin.preheader

neg:                                              ; preds = %pre_entry
  %n_1 = sub i64 0, %n
  br label %begin.preheader

begin.preheader:                                  ; preds = %pre_entry, %neg
  %n_2.ph = phi i64 [ %n, %pre_entry ], [ %n_1, %neg ]
  br label %begin.outer

begin.outer:                                      ; preds = %begin.outer.backedge, %begin.preheader
  %res_1.ph = phi i64 [ 0, %begin.preheader ], [ %res_1.ph.be, %begin.outer.backedge ]
  %i_1.ph = phi i64 [ 0, %begin.preheader ], [ %i_2.lcssa2, %begin.outer.backedge ]
  br label %begin

begin:                                            ; preds = %begin.outer, %check
  %i_1 = phi i64 [ %i_2, %check ], [ %i_1.ph, %begin.outer ]
  %i_2 = add i64 %i_1, 1
  %isq_0 = mul i64 %i_2, %i_2
  %sqgt_0 = icmp sgt i64 %isq_0, %n_2.ph
  br i1 %sqgt_0, label %end, label %check

check:                                            ; preds = %begin
  %m_0 = call i64 @__mod(i64 %n_2.ph, i64 %i_2)
  %eqz_0 = icmp eq i64 %m_0, 0
  br i1 %eqz_0, label %body, label %begin

body:                                             ; preds = %check
  %i_2.lcssa2 = phi i64 [ %i_2, %check ]
  call void @print_int(i64 %i_2.lcssa2)
  call void @print_newline()
  %res_2 = add i64 %res_1.ph, %i_2.lcssa2
  %d_0 = sdiv i64 %n_2.ph, %i_2.lcssa2
  %deqi_0 = icmp eq i64 %d_0, %i_2.lcssa2
  br i1 %deqi_0, label %begin.outer.backedge, label %then

then:                                             ; preds = %body
  call void @print_int(i64 %d_0)
  call void @print_newline()
  %res_3 = add i64 %res_2, %d_0
  br label %begin.outer.backedge

begin.outer.backedge:                             ; preds = %then, %body
  %res_1.ph.be = phi i64 [ %res_2, %body ], [ %res_3, %then ]
  br label %begin.outer

end:                                              ; preds = %begin
  %res_1.ph.lcssa = phi i64 [ %res_1.ph, %begin ]
  call void @print_int(i64 %res_1.ph.lcssa)
  call void @print_newline()
  ret void
}

define dso_local i64 @__mod(i64 %dividend, i64 %divisor) {
pre_entry:
  %dividend.fr = freeze i64 %dividend
  %0 = srem i64 %dividend.fr, %divisor
  ret i64 %0
}

define dso_local i32 @main(i32 %argc, ptr %argv) {
  %.not = icmp eq i32 %argc, 1
  br i1 %.not, label %4, label %1

1:                                                ; preds = %0
  %2 = add nsw i32 %argc, -1
  %3 = call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str.4, i32 0, i32 %2)
  call void @exit(i32 2)
  unreachable

4:                                                ; preds = %0
  call void @__main()
  ret i32 0
}
