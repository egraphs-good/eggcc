
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
  %C_0 = call i64* @__matrix_new(i64 200, i64 240)
  %A_0 = call i64* @__matrix_new(i64 200, i64 200)
  %B_0 = call i64* @__matrix_new(i64 200, i64 240)
  call void @__init(i64* %C_0, i64* %A_0, i64* %B_0, i64 200, i64 200, i64 240, i64 240)
  br label %main_i
main_i:
  %i_1 = phi i64 [ %i_2, %main_j_done ], [ 0, %b0 ]
  %cond_0 = icmp slt i64 %i_1, 200
  br i1 %cond_0, label %main_i_body, label %main_i_done
main_i_body:
  br label %main_j
main_j:
  %j_1 = phi i64 [ %j_2, %main_k_done ], [ 0, %main_i_body ]
  %cond_1 = phi i1 [ %cond_4, %main_k_done ], [ %cond_0, %main_i_body ]
  %cond_2 = icmp slt i64 %j_1, 240
  br i1 %cond_2, label %main_j_body, label %main_j_done
main_j_body:
  br label %main_k
main_k:
  %k_1 = phi i64 [ %k_2, %main_k_body ], [ 0, %main_j_body ]
  %temp2_1 = phi i64 [ %temp2_2, %main_k_body ], [ 0, %main_j_body ]
  %cond_3 = phi i1 [ %cond_4, %main_k_body ], [ %cond_2, %main_j_body ]
  %cond_4 = icmp slt i64 %k_1, %i_1
  br i1 %cond_4, label %main_k_body, label %main_k_done
main_k_body:
  %Bij_0 = call i64 @__matrix_get(i64* %B_0, i64 %i_1, i64 %j_1, i64 240)
  %Aik_0 = call i64 @__matrix_get(i64* %A_0, i64 %i_1, i64 %k_1, i64 200)
  %incr_0 = mul i64 3, %Bij_0
  %incr_1 = mul i64 %incr_0, %Aik_0
  call void @__matrix_incr(i64* %C_0, i64 %k_1, i64 %j_1, i64 240, i64 %incr_1)
  %Bkj_0 = call i64 @__matrix_get(i64* %B_0, i64 %k_1, i64 %j_1, i64 240)
  %Aik_1 = call i64 @__matrix_get(i64* %A_0, i64 %i_1, i64 %k_1, i64 200)
  %incr_2 = mul i64 %Bkj_0, %Aik_1
  %temp2_2 = add i64 %temp2_1, %incr_2
  %k_2 = add i64 %k_1, 1
  br label %main_k
main_k_done:
  %Cij_0 = call i64 @__matrix_get(i64* %C_0, i64 %i_1, i64 %j_1, i64 240)
  %Bij_1 = call i64 @__matrix_get(i64* %B_0, i64 %i_1, i64 %j_1, i64 240)
  %Aii_0 = call i64 @__matrix_get(i64* %A_0, i64 %i_1, i64 %i_1, i64 200)
  %val1_0 = mul i64 2, %Cij_0
  %val2_0 = mul i64 3, %Bij_1
  %val2_1 = mul i64 %val2_0, %Aii_0
  %val3_0 = mul i64 3, %temp2_1
  %val_0 = add i64 %val1_0, %val2_1
  %val_1 = add i64 %val_0, %val3_0
  call void @__matrix_set(i64* %C_0, i64 %i_1, i64 %j_1, i64 240, i64 %val_1)
  %j_2 = add i64 %j_1, 1
  br label %main_j
main_j_done:
  %i_2 = add i64 %i_1, 1
  br label %main_i
main_i_done:
  call void @__matrix_print(i64* %C_0, i64 200, i64 240)
  %z0 = bitcast i64* %C_0 to i8*
  call void @free(i8* %z0)
  %z1 = bitcast i64* %A_0 to i8*
  call void @free(i8* %z1)
  %z2 = bitcast i64* %B_0 to i8*
  call void @free(i8* %z2)
  ret void

}


define dso_local void @__init(i64* %C, i64* %A, i64* %B, i64 %M, i64 %fM, i64 %N, i64 %fN) {
pre_entry:
  br label %init_CB_i
init_CB_i:
  %fi_1 = phi i64 [ %fi_2, %init_CB_j_done ], [ 0, %pre_entry ]
  %i_1 = phi i64 [ %i_2, %init_CB_j_done ], [ 0, %pre_entry ]
  %cond_0 = icmp slt i64 %i_1, %M
  br i1 %cond_0, label %init_CB_i_body, label %init_CB_i_done
init_CB_i_body:
  br label %init_CB_j
init_CB_j:
  %fj_1 = phi i64 [ %fj_2, %init_CB_j_body ], [ 0, %init_CB_i_body ]
  %j_1 = phi i64 [ %j_2, %init_CB_j_body ], [ 0, %init_CB_i_body ]
  %cond_1 = phi i1 [ %cond_2, %init_CB_j_body ], [ %cond_0, %init_CB_i_body ]
  %cond_2 = icmp slt i64 %j_1, %N
  br i1 %cond_2, label %init_CB_j_body, label %init_CB_j_done
init_CB_j_body:
  %val_0 = add i64 %fi_1, %fj_1
  %val_1 = call i64 @__fmod(i64 %val_0, i64 100)
  %val_2 = sdiv i64 %val_1, %fM
  call void @__matrix_set(i64* %C, i64 %i_1, i64 %j_1, i64 %N, i64 %val_2)
  %val_3 = add i64 %fN, %fi_1
  %val_4 = sub i64 %val_3, %fj_1
  %val_5 = call i64 @__fmod(i64 %val_4, i64 100)
  %val_6 = sdiv i64 %val_5, %fM
  call void @__matrix_set(i64* %B, i64 %i_1, i64 %j_1, i64 %N, i64 %val_6)
  %j_2 = add i64 %j_1, 1
  %fj_2 = add i64 %fj_1, 1
  br label %init_CB_j
init_CB_j_done:
  %i_2 = add i64 %i_1, 1
  %fi_2 = add i64 %fi_1, 1
  br label %init_CB_i
init_CB_i_done:
  br label %init_A_i
init_A_i:
  %fj_3 = phi i64 [ %fj_5, %init_A_j2_done ], [ 0, %init_CB_i_done ]
  %j_3 = phi i64 [ %j_8, %init_A_j2_done ], [ 0, %init_CB_i_done ]
  %cond_3 = phi i1 [ %cond_8, %init_A_j2_done ], [ %cond_1, %init_CB_i_done ]
  %fi_4 = phi i64 [ %fi_5, %init_A_j2_done ], [ 0, %init_CB_i_done ]
  %i_4 = phi i64 [ %i_5, %init_A_j2_done ], [ 0, %init_CB_i_done ]
  %cond_4 = icmp slt i64 %i_4, %M
  br i1 %cond_4, label %init_A_i_body, label %init_A_i_done
init_A_i_body:
  br label %init_A_j1
init_A_j1:
  %fj_5 = phi i64 [ %fj_6, %init_A_j1_body ], [ 0, %init_A_i_body ]
  %j_5 = phi i64 [ %j_6, %init_A_j1_body ], [ 0, %init_A_i_body ]
  %cond_5 = phi i1 [ %cond_6, %init_A_j1_body ], [ %cond_4, %init_A_i_body ]
  %cond_6 = icmp sle i64 %j_5, %i_4
  br i1 %cond_6, label %init_A_j1_body, label %init_A_j1_done
init_A_j1_body:
  %val_7 = add i64 %fi_4, %fj_5
  %val_8 = call i64 @__fmod(i64 %val_7, i64 100)
  %val_9 = sdiv i64 %val_8, %fM
  call void @__matrix_set(i64* %A, i64 %i_4, i64 %j_5, i64 %M, i64 %val_9)
  %j_6 = add i64 %j_5, 1
  %fj_6 = add i64 %fj_5, 1
  br label %init_A_j1
init_A_j1_done:
  %j_7 = add i64 %i_4, 1
  br label %init_A_j2
init_A_j2:
  %j_8 = phi i64 [ %j_9, %init_A_j2_body ], [ %j_7, %init_A_j1_done ]
  %cond_7 = phi i1 [ %cond_8, %init_A_j2_body ], [ %cond_6, %init_A_j1_done ]
  %cond_8 = icmp slt i64 %j_8, %M
  br i1 %cond_8, label %init_A_j2_body, label %init_A_j2_done
init_A_j2_body:
  call void @__matrix_set(i64* %A, i64 %i_4, i64 %j_8, i64 %M, i64 -999)
  %j_9 = add i64 %j_8, 1
  br label %init_A_j2
init_A_j2_done:
  %i_5 = add i64 %i_4, 1
  %fi_5 = add i64 %fi_4, 1
  br label %init_A_i
init_A_i_done:
  ret void

}


define dso_local i64* @__matrix_new(i64 %Nrow, i64 %Ncol) {
pre_entry:
  %total_0 = mul i64 %Nrow, %Ncol
  %z0 = mul i64 %total_0, 8
  %z1 = call i8* @malloc(i64 %z0)
  %ptr_0 = bitcast i8* %z1 to i64*
  ret i64* %ptr_0

}


define dso_local i64* @__matrix_loc(i64* %mtx, i64 %row, i64 %col, i64 %Ncol) {
pre_entry:
  %row_offset_0 = mul i64 %row, %Ncol
  %offset_0 = add i64 %row_offset_0, %col
  %new_ptr_0 = getelementptr inbounds i64, i64* %mtx, i64 %offset_0
  ret i64* %new_ptr_0

}


define dso_local i64 @__matrix_get(i64* %mtx, i64 %row, i64 %col, i64 %Ncol) {
pre_entry:
  %ptr_0 = call i64* @__matrix_loc(i64* %mtx, i64 %row, i64 %col, i64 %Ncol)
  %val_0 = load i64, i64* %ptr_0
  ret i64 %val_0

}


define dso_local void @__matrix_set(i64* %mtx, i64 %row, i64 %col, i64 %Ncol, i64 %val) {
pre_entry:
  %ptr_0 = call i64* @__matrix_loc(i64* %mtx, i64 %row, i64 %col, i64 %Ncol)
  store i64 %val, i64* %ptr_0
  ret void

}


define dso_local void @__matrix_incr(i64* %mtx, i64 %row, i64 %col, i64 %Ncol, i64 %incr) {
pre_entry:
  %ptr_0 = call i64* @__matrix_loc(i64* %mtx, i64 %row, i64 %col, i64 %Ncol)
  %val_0 = load i64, i64* %ptr_0
  %new_val_0 = add i64 %val_0, %incr
  store i64 %new_val_0, i64* %ptr_0
  ret void

}


define dso_local void @__matrix_print(i64* %mtx, i64 %Nrow, i64 %Ncol) {
pre_entry:
  %total_0 = mul i64 %Nrow, %Ncol
  br label %while
while:
  %i_1 = phi i64 [ %i_2, %body ], [ 0, %pre_entry ]
  %cond_0 = icmp slt i64 %i_1, %total_0
  br i1 %cond_0, label %body, label %done
body:
  %mtx_loc_0 = getelementptr inbounds i64, i64* %mtx, i64 %i_1
  %val_0 = load i64, i64* %mtx_loc_0
  call void @print_int(i64 %val_0)
  call void @print_newline()
  %i_2 = add i64 %i_1, 1
  br label %while
done:
  ret void

}


define dso_local i64 @__fmod(i64 %n, i64 %m) {
pre_entry:
  br label %while
while:
  %rem_1 = phi i64 [ %rem_2, %done_inner ], [ %n, %pre_entry ]
  %cond_0 = icmp sge i64 %rem_1, %m
  br i1 %cond_0, label %body, label %done
body:
  br label %while_inner
while_inner:
  %decr_1 = phi i64 [ %decr_2, %body_inner ], [ %m, %body ]
  %cond_1 = phi i1 [ %cond_2, %body_inner ], [ %cond_0, %body ]
  %diff_0 = sub i64 %rem_1, %decr_1
  %cond_2 = icmp sge i64 %diff_0, 0
  br i1 %cond_2, label %body_inner, label %done_inner
body_inner:
  %decr_2 = mul i64 %decr_1, 2
  br label %while_inner
done_inner:
  %decr_3 = sdiv i64 %decr_1, 2
  %rem_2 = sub i64 %rem_1, %decr_3
  br label %while
done:
  ret i64 %rem_1

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

