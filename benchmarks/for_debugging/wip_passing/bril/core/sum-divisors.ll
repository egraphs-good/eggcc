
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
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 100000
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
  %nlz_0 = icmp slt i64 %n, 0
  br i1 %nlz_0, label %neg, label %begin
neg:
  %n_1 = mul i64 %n, -1
  br label %begin
begin:
  %res_1 = phi i64 [ %res_3, %then ], [ %res_2, %body ], [ %res_1, %check ], [ 0, %neg ], [ 0, %pre_entry ]
  %i_1 = phi i64 [ %i_2, %then ], [ %i_2, %body ], [ %i_2, %check ], [ 0, %neg ], [ 0, %pre_entry ]
  %n_2 = phi i64 [ %n_2, %then ], [ %n_2, %body ], [ %n_2, %check ], [ %n_1, %neg ], [ %n, %pre_entry ]
  %i_2 = add i64 %i_1, 1
  %isq_0 = mul i64 %i_2, %i_2
  %sqgt_0 = icmp sgt i64 %isq_0, %n_2
  br i1 %sqgt_0, label %end, label %check
check:
  %m_0 = call i64 @__mod(i64 %n_2, i64 %i_2)
  %eqz_0 = icmp eq i64 %m_0, 0
  br i1 %eqz_0, label %body, label %begin
body:
  call void @print_int(i64 %i_2)
  call void @print_newline()
  %res_2 = add i64 %res_1, %i_2
  %d_0 = sdiv i64 %n_2, %i_2
  %deqi_0 = icmp eq i64 %d_0, %i_2
  br i1 %deqi_0, label %begin, label %then
then:
  call void @print_int(i64 %d_0)
  call void @print_newline()
  %res_3 = add i64 %res_2, %d_0
  br label %begin
end:
  call void @print_int(i64 %res_1)
  call void @print_newline()
  ret void

}


define dso_local i64 @__mod(i64 %dividend, i64 %divisor) {
pre_entry:
  %quot_0 = sdiv i64 %dividend, %divisor
  %prod_0 = mul i64 %divisor, %quot_0
  %diff_0 = sub i64 %dividend, %prod_0
  ret i64 %diff_0

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

