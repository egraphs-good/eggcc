
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
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 50000
  br i1 %loop_cond_0, label %loop_body, label %loop_done
loop_body:
  call void @__orig_main(i64 %loop_counter_1)
  %loop_counter_2 = add i64 %loop_counter_1, 1
  br label %loop_cond
loop_done:
  ret void

}


define dso_local void @__orig_main(i64 %size) {
pre_entry:
  %z0 = mul i64 %size, 8
  %z1 = call i8* @malloc(i64 %z0)
  %arr_0 = bitcast i8* %z1 to i64*
  call void @__fill_array(i64* %arr_0, i64 %size)
  %checksum_0 = call i64 @__adler32(i64* %arr_0, i64 %size)
  call void @print_int(i64 %checksum_0)
  call void @print_newline()
  %z2 = bitcast i64* %arr_0 to i8*
  call void @free(i8* %z2)
  ret void

}


define dso_local i64 @__mod(i64 %r, i64 %s) {
pre_entry:
  %x_0 = sdiv i64 %r, %s
  %y_0 = mul i64 %x_0, %s
  %result_0 = sub i64 %r, %y_0
  ret i64 %result_0

}


define dso_local void @__fill_array(i64* %arr, i64 %size) {
pre_entry:
  br label %loop
loop:
  %loc_1 = phi i64* [ %loc_2, %loop ], [ %arr, %pre_entry ]
  %curr_1 = phi i64 [ %curr_2, %loop ], [ 0, %pre_entry ]
  store i64 %curr_1, i64* %loc_1
  %loc_2 = getelementptr inbounds i64, i64* %loc_1, i64 1
  %curr_2 = add i64 %curr_1, 1
  %continue_0 = icmp slt i64 %curr_2, %size
  br i1 %continue_0, label %loop, label %exit
exit:
  ret void

}


define dso_local i64 @__bitwise_or(i64 %x, i64 %y) {
pre_entry:
  br label %loop
loop:
  %result_1 = phi i64 [ %result_3, %false ], [ 0, %pre_entry ]
  %val_1 = phi i64 [ %val_2, %false ], [ 1, %pre_entry ]
  %y_1 = phi i64 [ %y_2, %false ], [ %y, %pre_entry ]
  %x_1 = phi i64 [ %x_2, %false ], [ %x, %pre_entry ]
  %xmod2_0 = call i64 @__mod(i64 %x_1, i64 2)
  %ymod2_0 = call i64 @__mod(i64 %y_1, i64 2)
  %xodd_0 = icmp eq i64 %xmod2_0, 1
  %yodd_0 = icmp eq i64 %ymod2_0, 1
  %cond_0 = or i1 %xodd_0, %yodd_0
  br i1 %cond_0, label %true, label %false
true:
  %result_2 = add i64 %result_1, %val_1
  br label %false
false:
  %result_3 = phi i64 [ %result_2, %true ], [ %result_1, %loop ]
  %x_2 = sdiv i64 %x_1, 2
  %y_2 = sdiv i64 %y_1, 2
  %xpos_0 = icmp sgt i64 %x_2, 0
  %ypos_0 = icmp sgt i64 %y_2, 0
  %val_2 = mul i64 %val_1, 2
  %continue_0 = or i1 %xpos_0, %ypos_0
  br i1 %continue_0, label %loop, label %exit
exit:
  ret i64 %result_3

}


define dso_local i64 @__adler32(i64* %arr, i64 %size) {
pre_entry:
  br label %loop
loop:
  %loc_1 = phi i64* [ %loc_2, %loop ], [ %arr, %pre_entry ]
  %curr_1 = phi i64 [ %curr_2, %loop ], [ 0, %pre_entry ]
  %b_1 = phi i64 [ %b_2, %loop ], [ 0, %pre_entry ]
  %a_1 = phi i64 [ %a_2, %loop ], [ 1, %pre_entry ]
  %val_0 = load i64, i64* %loc_1
  %a_2 = add i64 %a_1, %val_0
  %b_2 = add i64 %b_1, %a_2
  %loc_2 = getelementptr inbounds i64, i64* %loc_1, i64 1
  %curr_2 = add i64 %curr_1, 1
  %continue_0 = icmp slt i64 %curr_2, %size
  br i1 %continue_0, label %loop, label %exit
exit:
  %a_3 = call i64 @__mod(i64 %a_2, i64 65521)
  %b_3 = call i64 @__mod(i64 %b_2, i64 65521)
  %b_4 = mul i64 %b_3, 65536
  %result_0 = call i64 @__bitwise_or(i64 %b_4, i64 %a_3)
  ret i64 %result_0

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

