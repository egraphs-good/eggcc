
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


define dso_local i64 @__sumOfSquares(i64 %n) {
pre_entry:
  br label %for.cond.1
for.cond.1:
  %i_1 = phi i64 [ %v14_0, %for.body.1 ], [ 1, %pre_entry ]
  %res_1 = phi i64 [ %v11_0, %for.body.1 ], [ 0, %pre_entry ]
  %v5_0 = icmp sle i64 %i_1, %n
  br i1 %v5_0, label %for.body.1, label %for.end.1
for.body.1:
  %v8_0 = mul i64 %i_1, %i_1
  %v11_0 = add i64 %res_1, %v8_0
  %v14_0 = add i64 %i_1, 1
  br label %for.cond.1
for.end.1:
  ret i64 %res_1

}


define dso_local i64 @__squareOfSum(i64 %n) {
pre_entry:
  br label %for.cond.1
for.cond.1:
  %i_1 = phi i64 [ %v11_0, %for.body.1 ], [ 1, %pre_entry ]
  %res_1 = phi i64 [ %v8_0, %for.body.1 ], [ 0, %pre_entry ]
  %v5_0 = icmp sle i64 %i_1, %n
  br i1 %v5_0, label %for.body.1, label %for.end.1
for.body.1:
  %v8_0 = add i64 %res_1, %i_1
  %v11_0 = add i64 %i_1, 1
  br label %for.cond.1
for.end.1:
  %v14_0 = mul i64 %res_1, %res_1
  ret i64 %v14_0

}


define dso_local void @__main() {
b0:
  br label %loop_cond
loop_cond:
  %loop_counter_1 = phi i64 [ %loop_counter_2, %loop_body ], [ 10, %b0 ]
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 30000
  br i1 %loop_cond_0, label %loop_body, label %loop_done
loop_body:
  call void @__orig_main(i64 %loop_counter_1)
  %loop_counter_2 = add i64 %loop_counter_1, 1
  br label %loop_cond
loop_done:
  ret void

}


define dso_local void @__orig_main(i64 %n) {
pre_entry:
  %sum_0 = call i64 @__sumOfSquares(i64 %n)
  %square_0 = call i64 @__squareOfSum(i64 %n)
  %v4_0 = sub i64 %square_0, %sum_0
  call void @print_int(i64 %v4_0)
  call void @print_newline()
  ret void

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

