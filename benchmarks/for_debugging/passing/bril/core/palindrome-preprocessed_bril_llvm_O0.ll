; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpCotcax/postprocessed.ll'
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
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 1000000
  br i1 %loop_cond_0, label %loop_body, label %loop_done

loop_body:                                        ; preds = %loop_cond
  call void @__orig_main(i64 %loop_counter_1)
  %loop_counter_2 = add i64 %loop_counter_1, 1
  br label %loop_cond

loop_done:                                        ; preds = %loop_cond
  ret void
}

define dso_local void @__orig_main(i64 %in) {
pre_entry:
  br label %for.cond.outer

for.cond.outer:                                   ; preds = %pre_entry, %if.true
  %not_finished_1.ph = phi i1 [ true, %pre_entry ], [ false, %if.true ]
  %index_1.ph = phi i64 [ 1, %pre_entry ], [ %index_1.lcssa1, %if.true ]
  br label %for.cond

for.cond:                                         ; preds = %for.cond.outer, %if.false
  %index_1 = phi i64 [ %index_2, %if.false ], [ %index_1.ph, %for.cond.outer ]
  br i1 %not_finished_1.ph, label %for.body, label %for.end

for.body:                                         ; preds = %for.cond
  %power_0 = call i64 @__pow(i64 10, i64 %index_1)
  %d_0 = sdiv i64 %in, %power_0
  %check_0 = icmp eq i64 %d_0, 0
  br i1 %check_0, label %if.true, label %if.false

if.true:                                          ; preds = %for.body
  %index_1.lcssa1 = phi i64 [ %index_1, %for.body ]
  br label %for.cond.outer

if.false:                                         ; preds = %for.body
  %index_2 = add i64 %index_1, 1
  br label %for.cond

for.end:                                          ; preds = %for.cond
  %index_1.lcssa = phi i64 [ %index_1, %for.cond ]
  %exp_0 = add i64 %index_1.lcssa, -1
  %is_palindrome_0 = call i1 @__palindrome(i64 %in, i64 %exp_0)
  call void @print_bool(i1 %is_palindrome_0)
  call void @print_newline()
  ret void
}

define dso_local i64 @__pow(i64 %base, i64 %exp) {
pre_entry:
  br label %for.cond.pow.outer

for.cond.pow.outer:                               ; preds = %pre_entry, %if.true.pow
  %not_finished_1.ph = phi i1 [ true, %pre_entry ], [ false, %if.true.pow ]
  %res_1.ph = phi i64 [ 1, %pre_entry ], [ %res_1.lcssa2, %if.true.pow ]
  %exp_1.ph = phi i64 [ %exp, %pre_entry ], [ %exp_1.lcssa1, %if.true.pow ]
  br label %for.cond.pow

for.cond.pow:                                     ; preds = %for.cond.pow.outer, %if.false.pow
  %res_1 = phi i64 [ %res_2, %if.false.pow ], [ %res_1.ph, %for.cond.pow.outer ]
  %exp_1 = phi i64 [ %exp_2, %if.false.pow ], [ %exp_1.ph, %for.cond.pow.outer ]
  br i1 %not_finished_1.ph, label %for.body.pow, label %for.end.pow

for.body.pow:                                     ; preds = %for.cond.pow
  %finished_0 = icmp eq i64 %exp_1, 0
  br i1 %finished_0, label %if.true.pow, label %if.false.pow

if.true.pow:                                      ; preds = %for.body.pow
  %res_1.lcssa2 = phi i64 [ %res_1, %for.body.pow ]
  %exp_1.lcssa1 = phi i64 [ %exp_1, %for.body.pow ]
  br label %for.cond.pow.outer

if.false.pow:                                     ; preds = %for.body.pow
  %res_2 = mul i64 %res_1, %base
  %exp_2 = add i64 %exp_1, -1
  br label %for.cond.pow

for.end.pow:                                      ; preds = %for.cond.pow
  %res_1.lcssa = phi i64 [ %res_1, %for.cond.pow ]
  ret i64 %res_1.lcssa
}

define dso_local i1 @__palindrome(i64 %in, i64 %len) {
pre_entry:
  %in.fr = freeze i64 %in
  %check_0 = icmp slt i64 %len, 1
  br i1 %check_0, label %if.true.palindrome, label %if.false.palindrome

if.true.palindrome:                               ; preds = %pre_entry
  br label %if.end.palindrome

if.false.palindrome:                              ; preds = %pre_entry
  %power_0 = call i64 @__pow(i64 10, i64 %len)
  %left_0 = sdiv i64 %in.fr, %power_0
  %0 = srem i64 %in.fr, 10
  %is_equal_0 = icmp eq i64 %left_0, %0
  br i1 %is_equal_0, label %if.true.mirror, label %if.false.mirror

if.true.mirror:                                   ; preds = %if.false.palindrome
  %temp_0 = mul i64 %power_0, %left_0
  %1 = add i64 %temp_0, %0
  %temp_2 = sub i64 %in.fr, %1
  %next_in_0 = sdiv i64 %temp_2, 10
  %next_len_0 = add nsw i64 %len, -2
  %is_palindrome_2 = call i1 @__palindrome(i64 %next_in_0, i64 %next_len_0)
  br label %if.end.palindrome

if.false.mirror:                                  ; preds = %if.false.palindrome
  br label %if.end.palindrome

if.end.palindrome:                                ; preds = %if.false.mirror, %if.true.mirror, %if.true.palindrome
  %is_palindrome_4 = phi i1 [ false, %if.false.mirror ], [ %is_palindrome_2, %if.true.mirror ], [ true, %if.true.palindrome ]
  ret i1 %is_palindrome_4
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
