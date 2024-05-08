
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
  %A_0 = call i64* @__matrix_new(i64 250, i64 250)
  %B_0 = call i64* @__matrix_new(i64 250, i64 250)
  %x_0 = call i64* @__vector_new(i64 250)
  call void @__init(i64* %A_0, i64* %B_0, i64* %x_0, i64 250, i64 250)
  %tmp_0 = call i64* @__vector_new(i64 250)
  %y_0 = call i64* @__vector_new(i64 250)
  br label %main_i
main_i:
  %i_1 = phi i64 [ %i_2, %main_j_done ], [ 0, %b0 ]
  %cond_0 = icmp slt i64 %i_1, 250
  br i1 %cond_0, label %main_i_body, label %main_i_done
main_i_body:
  call void @__vector_set(i64* %tmp_0, i64 %i_1, i64 0)
  call void @__vector_set(i64* %y_0, i64 %i_1, i64 0)
  br label %main_j
main_j:
  %j_1 = phi i64 [ %j_2, %main_j_body ], [ 0, %main_i_body ]
  %cond_1 = phi i1 [ %cond_2, %main_j_body ], [ %cond_0, %main_i_body ]
  %cond_2 = icmp slt i64 %j_1, 250
  br i1 %cond_2, label %main_j_body, label %main_j_done
main_j_body:
  %Aij_0 = call i64 @__matrix_get(i64* %A_0, i64 %i_1, i64 %j_1, i64 250)
  %xj_0 = call i64 @__vector_get(i64* %x_0, i64 %j_1)
  %tmpi_0 = call i64 @__vector_get(i64* %tmp_0, i64 %i_1)
  %val_0 = mul i64 %Aij_0, %xj_0
  %val_1 = add i64 %val_0, %tmpi_0
  call void @__vector_set(i64* %tmp_0, i64 %i_1, i64 %val_1)
  %Bij_0 = call i64 @__matrix_get(i64* %B_0, i64 %i_1, i64 %j_1, i64 250)
  %xj_1 = call i64 @__vector_get(i64* %x_0, i64 %j_1)
  %yi_0 = call i64 @__vector_get(i64* %y_0, i64 %i_1)
  %val_2 = mul i64 %Bij_0, %xj_1
  %val_3 = add i64 %val_2, %yi_0
  call void @__vector_set(i64* %y_0, i64 %i_1, i64 %val_3)
  %j_2 = add i64 %j_1, 1
  br label %main_j
main_j_done:
  %tmpi_1 = call i64 @__vector_get(i64* %tmp_0, i64 %i_1)
  %yi_1 = call i64 @__vector_get(i64* %y_0, i64 %i_1)
  %val1_0 = mul i64 3, %tmpi_1
  %val2_0 = mul i64 2, %yi_1
  %new_yi_0 = add i64 %val1_0, %val2_0
  call void @__vector_set(i64* %y_0, i64 %i_1, i64 %new_yi_0)
  %i_2 = add i64 %i_1, 1
  br label %main_i
main_i_done:
  call void @__vector_print(i64* %y_0, i64 250)
  %z0 = bitcast i64* %A_0 to i8*
  call void @free(i8* %z0)
  %z1 = bitcast i64* %B_0 to i8*
  call void @free(i8* %z1)
  %z2 = bitcast i64* %tmp_0 to i8*
  call void @free(i8* %z2)
  %z3 = bitcast i64* %x_0 to i8*
  call void @free(i8* %z3)
  %z4 = bitcast i64* %y_0 to i8*
  call void @free(i8* %z4)
  ret void

}


define dso_local void @__init(i64* %A, i64* %B, i64* %x, i64 %N, i64 %fN) {
pre_entry:
  br label %init_i
init_i:
  %fi_1 = phi i64 [ %fi_2, %init_j_done ], [ 0, %pre_entry ]
  %i_1 = phi i64 [ %i_2, %init_j_done ], [ 0, %pre_entry ]
  %cond_0 = icmp slt i64 %i_1, %N
  br i1 %cond_0, label %init_i_body, label %init_i_done
init_i_body:
  %val_0 = call i64 @__fmod(i64 %fi_1, i64 %fN)
  %val_1 = sdiv i64 %val_0, %fN
  call void @__vector_set(i64* %x, i64 %i_1, i64 %val_1)
  br label %init_j
init_j:
  %fj_1 = phi i64 [ %fj_2, %init_j_body ], [ 0, %init_i_body ]
  %j_1 = phi i64 [ %j_2, %init_j_body ], [ 0, %init_i_body ]
  %val_2 = phi i64 [ %val_10, %init_j_body ], [ %val_1, %init_i_body ]
  %cond_1 = phi i1 [ %cond_2, %init_j_body ], [ %cond_0, %init_i_body ]
  %cond_2 = icmp slt i64 %j_1, %N
  br i1 %cond_2, label %init_j_body, label %init_j_done
init_j_body:
  %val_3 = mul i64 %fi_1, %fj_1
  %val_4 = add i64 %val_3, 1
  %val_5 = call i64 @__fmod(i64 %val_4, i64 %fN)
  %val_6 = sdiv i64 %val_5, %fN
  call void @__matrix_set(i64* %A, i64 %i_1, i64 %j_1, i64 %N, i64 %val_6)
  %val_7 = mul i64 %fi_1, %fj_1
  %val_8 = add i64 %val_7, 2
  %val_9 = call i64 @__fmod(i64 %val_8, i64 %fN)
  %val_10 = sdiv i64 %val_9, %fN
  call void @__matrix_set(i64* %B, i64 %i_1, i64 %j_1, i64 %N, i64 %val_10)
  %j_2 = add i64 %j_1, 1
  %fj_2 = add i64 %fj_1, 1
  br label %init_j
init_j_done:
  %i_2 = add i64 %i_1, 1
  %fi_2 = add i64 %fi_1, 1
  br label %init_i
init_i_done:
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


define dso_local i64* @__vector_new(i64 %N) {
pre_entry:
  %z0 = mul i64 %N, 8
  %z1 = call i8* @malloc(i64 %z0)
  %ptr_0 = bitcast i8* %z1 to i64*
  ret i64* %ptr_0

}


define dso_local i64 @__vector_get(i64* %vec, i64 %i) {
pre_entry:
  %ptr_0 = getelementptr inbounds i64, i64* %vec, i64 %i
  %val_0 = load i64, i64* %ptr_0
  ret i64 %val_0

}


define dso_local void @__vector_set(i64* %vec, i64 %i, i64 %val) {
pre_entry:
  %ptr_0 = getelementptr inbounds i64, i64* %vec, i64 %i
  store i64 %val, i64* %ptr_0
  ret void

}


define dso_local void @__vector_print(i64* %vec, i64 %N) {
pre_entry:
  br label %while
while:
  %i_1 = phi i64 [ %i_2, %body ], [ 0, %pre_entry ]
  %cond_0 = icmp slt i64 %i_1, %N
  br i1 %cond_0, label %body, label %done
body:
  %val_0 = call i64 @__vector_get(i64* %vec, i64 %i_1)
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

