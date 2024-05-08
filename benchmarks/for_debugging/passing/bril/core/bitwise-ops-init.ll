
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


define dso_local i1 @__mod2(i64 %a) {
pre_entry:
  %tmp_0 = sdiv i64 %a, 2
  %tmp2_0 = mul i64 %tmp_0, 2
  %tmp3_0 = sub i64 %a, %tmp2_0
  %ans_0 = icmp eq i64 1, %tmp3_0
  ret i1 %ans_0

}


define dso_local i64 @__loop_subroutine(i64 %a, i64 %b, i1 %c) {
pre_entry:
  br label %loop
loop:
  %to_add_1 = phi i64 [ %to_add_2, %end_loop ], [ 1, %pre_entry ]
  %ans_1 = phi i64 [ %ans_3, %end_loop ], [ 0, %pre_entry ]
  %i_1 = phi i64 [ %i_2, %end_loop ], [ 0, %pre_entry ]
  %b_1 = phi i64 [ %b_2, %end_loop ], [ %b, %pre_entry ]
  %a_1 = phi i64 [ %a_2, %end_loop ], [ %a, %pre_entry ]
  %cond_0 = icmp sle i64 %i_1, 63
  br i1 %cond_0, label %here, label %end
here:
  %mod2a_0 = call i1 @__mod2(i64 %a_1)
  %mod2b_0 = call i1 @__mod2(i64 %b_1)
  %cond_add_0 = and i1 %mod2a_0, %mod2b_0
  br i1 %c, label %doOr, label %stay
doOr:
  %cond_add_1 = or i1 %mod2a_0, %mod2b_0
  br label %stay
stay:
  %cond_add_2 = phi i1 [ %cond_add_1, %doOr ], [ %cond_add_0, %here ]
  br i1 %cond_add_2, label %add, label %end_loop
add:
  %ans_2 = add i64 %ans_1, %to_add_1
  br label %end_loop
end_loop:
  %ans_3 = phi i64 [ %ans_2, %add ], [ %ans_1, %stay ]
  %a_2 = sdiv i64 %a_1, 2
  %b_2 = sdiv i64 %b_1, 2
  %to_add_2 = mul i64 %to_add_1, 2
  %i_2 = add i64 %i_1, 1
  br label %loop
end:
  ret i64 %ans_3

}


define dso_local i64 @__OR(i64 %a, i64 %b) {
pre_entry:
  %v1_0 = call i64 @__loop_subroutine(i64 %a, i64 %b, i1 1)
  ret i64 %v1_0

}


define dso_local i64 @__AND(i64 %a, i64 %b) {
pre_entry:
  %v1_0 = call i64 @__loop_subroutine(i64 %a, i64 %b, i1 0)
  ret i64 %v1_0

}


define dso_local i64 @__XOR(i64 %a, i64 %b) {
pre_entry:
  %and_val_0 = call i64 @__AND(i64 %a, i64 %b)
  %or_val_0 = call i64 @__OR(i64 %a, i64 %b)
  %ans_0 = sub i64 %or_val_0, %and_val_0
  ret i64 %ans_0

}


define dso_local void @__main() {
b0:
  br label %loop_cond
loop_cond:
  %loop_counter_1 = phi i64 [ %loop_counter_2, %loop2_done ], [ 10, %b0 ]
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 100
  br i1 %loop_cond_0, label %loop_body, label %loop_done
loop_body:
  br label %loop2_cond
loop2_cond:
  %loop2_counter_1 = phi i64 [ %loop2_counter_2, %loop3_done ], [ 10, %loop_body ]
  %loop2_cond_0 = icmp slt i64 %loop2_counter_1, 100
  br i1 %loop2_cond_0, label %loop2_body, label %loop2_done
loop2_body:
  br label %loop3_cond
loop3_cond:
  %loop3_counter_1 = phi i64 [ %loop3_counter_2, %loop3_body ], [ 10, %loop2_body ]
  %loop3_cond_0 = icmp slt i64 %loop3_counter_1, 100
  br i1 %loop3_cond_0, label %loop3_body, label %loop3_done
loop3_body:
  call void @__orig_main(i64 %loop_counter_1, i64 %loop2_counter_1, i64 %loop3_counter_1)
  %loop3_counter_2 = add i64 %loop3_counter_1, 1
  br label %loop3_cond
loop3_done:
  %loop2_counter_2 = add i64 %loop2_counter_1, 1
  br label %loop2_cond
loop2_done:
  %loop_counter_2 = add i64 %loop_counter_1, 1
  br label %loop_cond
loop_done:
  ret void

}


define dso_local void @__orig_main(i64 %a, i64 %b, i64 %c) {
pre_entry:
  %sel_0 = sub i64 %c, 1
  %less_0 = icmp slt i64 %sel_0, 0
  %equal_0 = icmp eq i64 %sel_0, 0
  %greater_0 = icmp sgt i64 %sel_0, 0
  br i1 %less_0, label %and_op, label %useless_lbl
useless_lbl:
  br i1 %equal_0, label %or_op, label %xor_op
and_op:
  %ans_3 = call i64 @__AND(i64 %a, i64 %b)
  br label %end
or_op:
  %ans_1 = call i64 @__OR(i64 %a, i64 %b)
  br label %end
xor_op:
  %ans_2 = call i64 @__XOR(i64 %a, i64 %b)
  br label %end
end:
  %ans_4 = phi i64 [ %ans_3, %and_op ], [ %ans_2, %xor_op ], [ %ans_1, %or_op ]
  call void @print_int(i64 %ans_4)
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

