
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


define dso_local i64* @__pack(i64 %size, i64 %n1, i64 %n2, i64 %n3, i64 %n4, i64 %n5) {
pre_entry:
  %z0 = mul i64 %size, 8
  %z1 = call i8* @malloc(i64 %z0)
  %array_0 = bitcast i8* %z1 to i64*
  %loc_0 = getelementptr inbounds i64, i64* %array_0, i64 0
  store i64 %n1, i64* %loc_0
  %i_1 = add i64 0, 1
  %loc_1 = getelementptr inbounds i64, i64* %array_0, i64 %i_1
  store i64 %n2, i64* %loc_1
  %i_2 = add i64 %i_1, 1
  %loc_2 = getelementptr inbounds i64, i64* %array_0, i64 %i_2
  store i64 %n3, i64* %loc_2
  %i_3 = add i64 %i_2, 1
  %loc_3 = getelementptr inbounds i64, i64* %array_0, i64 %i_3
  store i64 %n4, i64* %loc_3
  %i_4 = add i64 %i_3, 1
  %loc_4 = getelementptr inbounds i64, i64* %array_0, i64 %i_4
  store i64 %n5, i64* %loc_4
  ret i64* %array_0

}


define dso_local void @__print_array(i64* %array, i64 %size) {
pre_entry:
  br label %loop
loop:
  %i_1 = phi i64 [ %i_2, %loop_end ], [ 0, %pre_entry ]
  %cond_0 = icmp slt i64 %i_1, %size
  br i1 %cond_0, label %body, label %done
body:
  %loc_0 = getelementptr inbounds i64, i64* %array, i64 %i_1
  %val_0 = load i64, i64* %loc_0
  call void @print_int(i64 %val_0)
  call void @print_newline()
  br label %loop_end
loop_end:
  %i_2 = add i64 %i_1, 1
  br label %loop
done:
  ret void

}


define dso_local void @__swap_cond(i64* %array, i64 %j) {
pre_entry:
  %j_add_1_0 = add i64 %j, 1
  %loc_0 = getelementptr inbounds i64, i64* %array, i64 %j
  %loc_next_0 = getelementptr inbounds i64, i64* %array, i64 %j_add_1_0
  %elem_a_0 = load i64, i64* %loc_0
  %elem_b_0 = load i64, i64* %loc_next_0
  %cond_0 = icmp sgt i64 %elem_a_0, %elem_b_0
  br i1 %cond_0, label %swap, label %done
swap:
  store i64 %elem_b_0, i64* %loc_0
  store i64 %elem_a_0, i64* %loc_next_0
  br label %done
done:
  ret void

}


define dso_local void @__main() {
b0:
  br label %loop_cond
loop_cond:
  %loop_counter_1 = phi i64 [ %loop_counter_2, %loop2_done ], [ 10, %b0 ]
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 25
  br i1 %loop_cond_0, label %loop_body, label %loop_done
loop_body:
  br label %loop2_cond
loop2_cond:
  %loop2_counter_1 = phi i64 [ %loop2_counter_2, %loop3_done ], [ 10, %loop_body ]
  %loop2_cond_0 = icmp slt i64 %loop2_counter_1, 25
  br i1 %loop2_cond_0, label %loop2_body, label %loop2_done
loop2_body:
  br label %loop3_cond
loop3_cond:
  %loop3_counter_1 = phi i64 [ %loop3_counter_2, %loop4_done ], [ 10, %loop2_body ]
  %loop3_cond_0 = icmp slt i64 %loop3_counter_1, 25
  br i1 %loop3_cond_0, label %loop3_body, label %loop3_done
loop3_body:
  br label %loop4_cond
loop4_cond:
  %loop4_counter_1 = phi i64 [ %loop4_counter_2, %loop5_done ], [ 10, %loop3_body ]
  %loop4_cond_0 = icmp slt i64 %loop4_counter_1, 25
  br i1 %loop4_cond_0, label %loop4_body, label %loop4_done
loop4_body:
  br label %loop5_cond
loop5_cond:
  %loop5_counter_1 = phi i64 [ %loop5_counter_2, %loop5_body ], [ 10, %loop4_body ]
  %loop5_cond_0 = icmp slt i64 %loop5_counter_1, 25
  br i1 %loop5_cond_0, label %loop5_body, label %loop5_done
loop5_body:
  call void @__orig_main(i64 %loop_counter_1, i64 %loop2_counter_1, i64 %loop3_counter_1, i64 %loop4_counter_1, i64 %loop5_counter_1)
  %loop5_counter_2 = add i64 %loop5_counter_1, 1
  br label %loop5_cond
loop5_done:
  %loop4_counter_2 = add i64 %loop4_counter_1, 1
  br label %loop4_cond
loop4_done:
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


define dso_local void @__orig_main(i64 %n1, i64 %n2, i64 %n3, i64 %n4, i64 %n5) {
pre_entry:
  %array_0 = call i64* @__pack(i64 5, i64 %n1, i64 %n2, i64 %n3, i64 %n4, i64 %n5)
  %sizei_0 = sub i64 5, 1
  br label %loopi
loopi:
  %j_1 = phi i64 [ 0, %loopi_end ], [ 0, %pre_entry ]
  %i_1 = phi i64 [ %i_2, %loopi_end ], [ 0, %pre_entry ]
  %condi_0 = icmp slt i64 %i_1, %sizei_0
  br i1 %condi_0, label %bodyi, label %donei
bodyi:
  %sizej_0 = sub i64 5, %i_1
  %sizej_1 = sub i64 %sizej_0, 1
  br label %loopj
loopj:
  %j_2 = phi i64 [ %j_3, %loop_endj ], [ %j_1, %bodyi ]
  %condj_0 = icmp slt i64 %j_2, %sizej_1
  br i1 %condj_0, label %bodyj, label %donej
bodyj:
  call void @__swap_cond(i64* %array_0, i64 %j_2)
  br label %loop_endj
loop_endj:
  %j_3 = add i64 %j_2, 1
  br label %loopj
donej:
  br label %loopi_end
loopi_end:
  %i_2 = add i64 %i_1, 1
  br label %loopi
donei:
  call void @__print_array(i64* %array_0, i64 5)
  %z0 = bitcast i64* %array_0 to i8*
  call void @free(i8* %z0)
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

