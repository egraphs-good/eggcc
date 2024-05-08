
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
  %loop_counter_1 = phi i64 [ %loop_counter_2, %loop2_done ], [ 10, %b0 ]
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 1000
  br i1 %loop_cond_0, label %loop_body, label %loop_done
loop_body:
  br label %loop2_cond
loop2_cond:
  %loop2_counter_1 = phi i64 [ %loop2_counter_2, %loop2_body ], [ 10, %loop_body ]
  %loop2_cond_0 = icmp slt i64 %loop2_counter_1, 1000
  br i1 %loop2_cond_0, label %loop2_body, label %loop2_done
loop2_body:
  call void @__orig_main(i64 %loop_counter_1, i64 %loop2_counter_1, i64 %loop_counter_1)
  %loop2_counter_2 = add i64 %loop2_counter_1, 1
  br label %loop2_cond
loop2_done:
  %loop_counter_2 = add i64 %loop_counter_1, 1
  br label %loop_cond
loop_done:
  ret void

}


define dso_local void @__orig_main(i64 %e1, i64 %e2, i64 %e3) {
pre_entry:
  %nums_0 = call i64* @__create_arr(i64 3, i64 %e1, i64 %e2, i64 %e3)
  %first_elm_ptr_0 = getelementptr inbounds i64, i64* %nums_0, i64 0
  %major_elm_0 = load i64, i64* %first_elm_ptr_0
  br label %check_bound
check_bound:
  %i_1 = phi i64 [ %i_4, %eq_zero_else ], [ %i_3, %eq_zero_if ], [ %i_2, %incr_count ], [ 1, %pre_entry ]
  %count_1 = phi i64 [ %count_4, %eq_zero_else ], [ %count_3, %eq_zero_if ], [ %count_2, %incr_count ], [ 1, %pre_entry ]
  %major_elm_1 = phi i64 [ %major_elm_1, %eq_zero_else ], [ %major_elm_2, %eq_zero_if ], [ %major_elm_1, %incr_count ], [ %major_elm_0, %pre_entry ]
  %end_cond_0 = icmp sge i64 %i_1, 3
  br i1 %end_cond_0, label %end, label %body
body:
  %cur_ptr_0 = getelementptr inbounds i64, i64* %nums_0, i64 %i_1
  %cur_val_0 = load i64, i64* %cur_ptr_0
  %cur_major_cond_0 = icmp eq i64 %cur_val_0, %major_elm_1
  br i1 %cur_major_cond_0, label %incr_count, label %body.else
incr_count:
  %count_2 = add i64 %count_1, 1
  %i_2 = add i64 %i_1, 1
  br label %check_bound
body.else:
  %cnt_eq_0_0 = icmp eq i64 %count_1, 0
  br i1 %cnt_eq_0_0, label %eq_zero_if, label %eq_zero_else
eq_zero_if:
  %cur_ptr_1 = getelementptr inbounds i64, i64* %nums_0, i64 %i_1
  %major_elm_2 = load i64, i64* %cur_ptr_1
  %count_3 = add i64 %count_1, 1
  %i_3 = add i64 %i_1, 1
  br label %check_bound
eq_zero_else:
  %count_4 = sub i64 %count_1, 1
  %i_4 = add i64 %i_1, 1
  br label %check_bound
end:
  %z0 = bitcast i64* %nums_0 to i8*
  call void @free(i8* %z0)
  call void @print_int(i64 %major_elm_1)
  call void @print_newline()
  ret void

}


define dso_local i64* @__create_arr(i64 %size, i64 %e1, i64 %e2, i64 %e3) {
pre_entry:
  %z0 = mul i64 %size, 8
  %z1 = call i8* @malloc(i64 %z0)
  %array_0 = bitcast i8* %z1 to i64*
  %loc_0 = getelementptr inbounds i64, i64* %array_0, i64 0
  store i64 %e1, i64* %loc_0
  %i_1 = add i64 0, 1
  %loc_1 = getelementptr inbounds i64, i64* %array_0, i64 %i_1
  store i64 %e2, i64* %loc_1
  %i_2 = add i64 %i_1, 1
  %loc_2 = getelementptr inbounds i64, i64* %array_0, i64 %i_2
  store i64 %e3, i64* %loc_2
  ret i64* %array_0

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

