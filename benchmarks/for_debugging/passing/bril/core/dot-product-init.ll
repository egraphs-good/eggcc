
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


define dso_local i64 @__dot_product(i64* %vectorA, i64* %vectorB, i64 %size) {
pre_entry:
  br label %loop
loop:
  %answer_1 = phi i64 [ %answer_2, %loop ], [ 0, %pre_entry ]
  %index_1 = phi i64 [ %index_2, %loop ], [ 0, %pre_entry ]
  %ptrA_0 = getelementptr inbounds i64, i64* %vectorA, i64 %index_1
  %ptrB_0 = getelementptr inbounds i64, i64* %vectorB, i64 %index_1
  %valA_0 = load i64, i64* %ptrA_0
  %valB_0 = load i64, i64* %ptrB_0
  %tmp_0 = mul i64 %valA_0, %valB_0
  %answer_2 = add i64 %answer_1, %tmp_0
  %index_2 = add i64 %index_1, 1
  %cond_0 = icmp slt i64 %index_2, %size
  br i1 %cond_0, label %loop, label %done
done:
  ret i64 %answer_2

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


define dso_local void @__orig_main(i64 %x) {
pre_entry:
  %z0 = mul i64 5, 8
  %z1 = call i8* @malloc(i64 %z0)
  %vectorA_0 = bitcast i8* %z1 to i64*
  %indexPtr_0 = getelementptr inbounds i64, i64* %vectorA_0, i64 0
  store i64 25, i64* %indexPtr_0
  %indexPtr_1 = getelementptr inbounds i64, i64* %indexPtr_0, i64 1
  store i64 50, i64* %indexPtr_1
  %indexPtr_2 = getelementptr inbounds i64, i64* %indexPtr_1, i64 1
  store i64 100, i64* %indexPtr_2
  %indexPtr_3 = getelementptr inbounds i64, i64* %indexPtr_2, i64 1
  store i64 150, i64* %indexPtr_3
  %indexPtr_4 = getelementptr inbounds i64, i64* %indexPtr_3, i64 1
  store i64 250, i64* %indexPtr_4
  %z2 = mul i64 5, 8
  %z3 = call i8* @malloc(i64 %z2)
  %vectorB_0 = bitcast i8* %z3 to i64*
  %indexPtr_5 = getelementptr inbounds i64, i64* %vectorB_0, i64 0
  store i64 2, i64* %indexPtr_5
  %indexPtr_6 = getelementptr inbounds i64, i64* %indexPtr_5, i64 1
  store i64 10, i64* %indexPtr_6
  %indexPtr_7 = getelementptr inbounds i64, i64* %indexPtr_6, i64 1
  store i64 20, i64* %indexPtr_7
  %indexPtr_8 = getelementptr inbounds i64, i64* %indexPtr_7, i64 1
  store i64 30, i64* %indexPtr_8
  %indexPtr_9 = getelementptr inbounds i64, i64* %indexPtr_8, i64 1
  store i64 40, i64* %indexPtr_9
  %val_0 = call i64 @__dot_product(i64* %vectorA_0, i64* %vectorB_0, i64 5)
  %val_1 = add i64 %val_0, %x
  call void @print_int(i64 %val_1)
  call void @print_newline()
  %z4 = bitcast i64* %vectorA_0 to i8*
  call void @free(i8* %z4)
  %z5 = bitcast i64* %vectorB_0 to i8*
  call void @free(i8* %z5)
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

