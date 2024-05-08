
; ModuleID = 'stdin'
source_filename = "stdin"
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-pc-linux-gnu"

@.str = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.str.1 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@.str.2 = private unnamed_addr constant [4 x i8] c"%ld\00", align 1
@.str.3 = private unnamed_addr constant [9 x i8] c"[object]\00", align 1
@.str.4 = private unnamed_addr constant [33 x i8] c"error: expected %d args, got %d\0A\00", align 1

; DECLARE LIBRARY CALLS
declare dso_local i32 @putchar(i32)
declare dso_local i32 @printf(i8*, ...)
declare dso_local void @exit(i32)
declare dso_local i64 @atol(i8*)
declare dso_local noalias i8* @malloc(i64)
declare dso_local void @free(i8*)

define dso_local i32 @btoi(i8* %0) #0 {
  %2 = alloca i8*, align 8
  store i8* %0, i8** %2, align 8
  %3 = load i8*, i8** %2, align 8
  %4 = load i8, i8* %3, align 1
  %5 = sext i8 %4 to i32
  %6 = icmp eq i32 %5, 116
  %7 = zext i1 %6 to i32
  ret i32 %7
}

define dso_local void @print_bool(i1 %0) {
  %2 = icmp ne i1 %0, 0
  br i1 %2, label %3, label %5

3:
  %4 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str, i64 0, i64 0))
  br label %7

5:
  %6 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([6 x i8], [6 x i8]* @.str.1, i64 0, i64 0))
  br label %7

7:
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
  %2 = alloca i64, align 8
  store i64 %0, i64* %2, align 8
  %3 = load i64, i64* %2, align 8
  %4 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %3)
  ret void
}

define dso_local void @print_ptr(i8* %0) {
  %2 = alloca i8*, align 8
  store i8* %0, i8** %2, align 8
  %3 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([9 x i8], [9 x i8]* @.str.3, i64 0, i64 0))
  ret void
}


define dso_local void @__main() {
b0:
  br label %loop_cond
loop_cond:
  %loop_counter_1 = phi i64 [ %loop_counter_2, %loop_body ], [ 10, %b0 ]
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 1000000
  br i1 %loop_cond_0, label %loop_body, label %loop_done
loop_body:
  call void @__orig_main(i64 %loop_counter_1)
  %loop_counter_2 = add i64 %loop_counter_1, 1
  br label %loop_cond
loop_done:
  ret void

}


define dso_local void @__orig_main(i64 %in) {
pre_entry:
  br label %for.cond
for.cond:
  %not_finished_1 = phi i1 [ %not_finished_1, %if.false ], [ 0, %if.true ], [ 1, %pre_entry ]
  %index_1 = phi i64 [ %index_2, %if.false ], [ %index_1, %if.true ], [ 1, %pre_entry ]
  br i1 %not_finished_1, label %for.body, label %for.end
for.body:
  %power_0 = call i64 @__pow(i64 10, i64 %index_1)
  %d_0 = sdiv i64 %in, %power_0
  %check_0 = icmp eq i64 %d_0, 0
  br i1 %check_0, label %if.true, label %if.false
if.true:
  br label %for.cond
if.false:
  %index_2 = add i64 %index_1, 1
  br label %for.cond
for.end:
  %exp_0 = sub i64 %index_1, 1
  %is_palindrome_0 = call i1 @__palindrome(i64 %in, i64 %exp_0)
  call void @print_bool(i1 %is_palindrome_0)
  call void @print_newline()
  ret void

}


define dso_local i64 @__pow(i64 %base, i64 %exp) {
pre_entry:
  br label %for.cond.pow
for.cond.pow:
  %not_finished_1 = phi i1 [ %not_finished_1, %if.false.pow ], [ 0, %if.true.pow ], [ 1, %pre_entry ]
  %res_1 = phi i64 [ %res_2, %if.false.pow ], [ %res_1, %if.true.pow ], [ 1, %pre_entry ]
  %exp_1 = phi i64 [ %exp_2, %if.false.pow ], [ %exp_1, %if.true.pow ], [ %exp, %pre_entry ]
  br i1 %not_finished_1, label %for.body.pow, label %for.end.pow
for.body.pow:
  %finished_0 = icmp eq i64 %exp_1, 0
  br i1 %finished_0, label %if.true.pow, label %if.false.pow
if.true.pow:
  br label %for.cond.pow
if.false.pow:
  %res_2 = mul i64 %res_1, %base
  %exp_2 = sub i64 %exp_1, 1
  br label %for.cond.pow
for.end.pow:
  ret i64 %res_1

}


define dso_local i1 @__palindrome(i64 %in, i64 %len) {
pre_entry:
  %check_0 = icmp sle i64 %len, 0
  br i1 %check_0, label %if.true.palindrome, label %if.false.palindrome
if.true.palindrome:
  br label %if.end.palindrome
if.false.palindrome:
  %power_0 = call i64 @__pow(i64 10, i64 %len)
  %left_0 = sdiv i64 %in, %power_0
  %v1_0 = sdiv i64 %in, 10
  %v2_0 = mul i64 %v1_0, 10
  %right_0 = sub i64 %in, %v2_0
  %is_equal_0 = icmp eq i64 %left_0, %right_0
  br i1 %is_equal_0, label %if.true.mirror, label %if.false.mirror
if.true.mirror:
  %temp_0 = mul i64 %power_0, %left_0
  %temp_1 = sub i64 %in, %temp_0
  %temp_2 = sub i64 %temp_1, %right_0
  %next_in_0 = sdiv i64 %temp_2, 10
  %next_len_0 = sub i64 %len, 2
  %is_palindrome_2 = call i1 @__palindrome(i64 %next_in_0, i64 %next_len_0)
  br label %if.end.palindrome
if.false.mirror:
  br label %if.end.palindrome
if.end.palindrome:
  %is_palindrome_4 = phi i1 [ 0, %if.false.mirror ], [ %is_palindrome_2, %if.true.mirror ], [ 1, %if.true.palindrome ]
  ret i1 %is_palindrome_4

}


define dso_local i32 @main(i32 %argc, i8** %argv) {
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca i8**, align 8
  store i32 0, i32* %1, align 4
  store i32 %argc, i32* %2, align 4
  store i8** %argv, i8*** %3, align 8
  %4 = load i32, i32* %2, align 4
  %5 = sub nsw i32 %4, 1
  %6 = icmp ne i32 %5, 0  ; NUM ARGS
  br i1 %6, label %7, label %11

7:
  %8 = load i32, i32* %2, align 4
  %9 = sub nsw i32 %8, 1
  %10 = call i32 (i8*, ...) @printf(i8* getelementptr inbounds ([33 x i8], [33 x i8]* @.str.4, i64 0, i64 0), i32 0, i32 %9)
  call void @exit(i32 2) #3
  unreachable

11:
  %12 = load i8**, i8*** %3, align 8

  call void @__main()
  ret i32 0
}

