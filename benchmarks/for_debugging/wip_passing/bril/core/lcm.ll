
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
  %loop_counter_1 = phi i64 [ %loop_counter_2, %inner_done ], [ 10, %b0 ]
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 250
  br i1 %loop_cond_0, label %loop_body, label %loop_done
loop_body:
  br label %inner_cond
inner_cond:
  %inner_counter_1 = phi i64 [ %inner_counter_2, %inner_body ], [ 10, %loop_body ]
  %inner_cond_0 = icmp slt i64 %inner_counter_1, 250
  br i1 %inner_cond_0, label %inner_body, label %inner_done
inner_body:
  call void @__orig_main(i64 %loop_counter_1, i64 %inner_counter_1)
  %inner_counter_2 = add i64 %inner_counter_1, 1
  br label %inner_cond
inner_done:
  %loop_counter_2 = add i64 %loop_counter_1, 1
  br label %loop_cond
loop_done:
  ret void

}


define dso_local void @__orig_main(i64 %x, i64 %y) {
pre_entry:
  %v4_0 = icmp sgt i64 %x, %y
  br i1 %v4_0, label %then.1, label %else.1
then.1:
  br label %else.1
else.1:
  %greater_2 = phi i64 [ %greater_3, %else.2 ], [ %x, %then.1 ], [ %y, %pre_entry ]
  %modX_0 = call i64 @__getMod(i64 %greater_2, i64 %x)
  %modY_0 = call i64 @__getMod(i64 %greater_2, i64 %y)
  %xZero_0 = icmp eq i64 %modX_0, 0
  %yZero_0 = icmp eq i64 %modY_0, 0
  %bothZero_0 = and i1 %xZero_0, %yZero_0
  br i1 %bothZero_0, label %then.2, label %else.2
then.2:
  call void @print_int(i64 %greater_2)
  call void @print_newline()
  br label %loopend
else.2:
  %greater_3 = add i64 %greater_2, 1
  br label %else.1
loopend:
  ret void

}


define dso_local i64 @__getMod(i64 %val, i64 %mod) {
pre_entry:
  %divisor_0 = sdiv i64 %val, %mod
  %multiple_0 = mul i64 %divisor_0, %mod
  %rem_0 = sub i64 %val, %multiple_0
  ret i64 %rem_0

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

