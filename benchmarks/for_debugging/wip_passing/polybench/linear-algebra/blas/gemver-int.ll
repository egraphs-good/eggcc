
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
  %A_0 = call i64* @__matrix_new(i64 400, i64 400)
  %u1_0 = call i64* @__vector_new(i64 400)
  %v1_0 = call i64* @__vector_new(i64 400)
  %u2_0 = call i64* @__vector_new(i64 400)
  %v2_0 = call i64* @__vector_new(i64 400)
  %w_0 = call i64* @__vector_new(i64 400)
  %x_0 = call i64* @__vector_new(i64 400)
  %y_0 = call i64* @__vector_new(i64 400)
  %z_0 = call i64* @__vector_new(i64 400)
  call void @__init(i64* %A_0, i64* %u1_0, i64* %v1_0, i64* %u2_0, i64* %v2_0, i64* %w_0, i64* %x_0, i64* %y_0, i64* %z_0, i64 400, i64 400)
  br label %part1_i
part1_i:
  %i_1 = phi i64 [ %i_2, %part1_j_done ], [ 0, %b0 ]
  %cond_0 = icmp slt i64 %i_1, 400
  br i1 %cond_0, label %part1_i_body, label %part1_i_done
part1_i_body:
  br label %part1_j
part1_j:
  %j_1 = phi i64 [ %j_2, %part1_j_body ], [ 0, %part1_i_body ]
  %cond_1 = phi i1 [ %cond_2, %part1_j_body ], [ %cond_0, %part1_i_body ]
  %cond_2 = icmp slt i64 %j_1, 400
  br i1 %cond_2, label %part1_j_body, label %part1_j_done
part1_j_body:
  %u1i_0 = call i64 @__vector_get(i64* %u1_0, i64 %i_1)
  %v1j_0 = call i64 @__vector_get(i64* %v1_0, i64 %j_1)
  %u2i_0 = call i64 @__vector_get(i64* %u2_0, i64 %i_1)
  %v2j_0 = call i64 @__vector_get(i64* %v2_0, i64 %j_1)
  %Aij_0 = call i64 @__matrix_get(i64* %A_0, i64 %i_1, i64 %j_1, i64 400)
  %tmp_0 = mul i64 %u2i_0, %v2j_0
  %new_Aij_0 = mul i64 %u1i_0, %v1j_0
  %new_Aij_1 = add i64 %tmp_0, %new_Aij_0
  %new_Aij_2 = add i64 %Aij_0, %new_Aij_1
  call void @__matrix_set(i64* %A_0, i64 %i_1, i64 %j_1, i64 400, i64 %new_Aij_2)
  %j_2 = add i64 %j_1, 1
  br label %part1_j
part1_j_done:
  %i_2 = add i64 %i_1, 1
  br label %part1_i
part1_i_done:
  br label %part2_i
part2_i:
  %j_3 = phi i64 [ %j_5, %part2_j_done ], [ 0, %part1_i_done ]
  %cond_3 = phi i1 [ %cond_6, %part2_j_done ], [ %cond_1, %part1_i_done ]
  %i_4 = phi i64 [ %i_5, %part2_j_done ], [ 0, %part1_i_done ]
  %cond_4 = icmp slt i64 %i_4, 400
  br i1 %cond_4, label %part2_i_body, label %part2_i_done
part2_i_body:
  br label %part2_j
part2_j:
  %j_5 = phi i64 [ %j_6, %part2_j_body ], [ 0, %part2_i_body ]
  %cond_5 = phi i1 [ %cond_6, %part2_j_body ], [ %cond_4, %part2_i_body ]
  %cond_6 = icmp slt i64 %j_5, 400
  br i1 %cond_6, label %part2_j_body, label %part2_j_done
part2_j_body:
  %Aji_0 = call i64 @__matrix_get(i64* %A_0, i64 %j_5, i64 %i_4, i64 400)
  %yj_0 = call i64 @__vector_get(i64* %y_0, i64 %j_5)
  %xi_0 = call i64 @__vector_get(i64* %x_0, i64 %i_4)
  %new_xi_0 = mul i64 %Aji_0, %yj_0
  %new_xi_1 = mul i64 2, %new_xi_0
  %new_xi_2 = add i64 %xi_0, %new_xi_1
  call void @__vector_set(i64* %x_0, i64 %i_4, i64 %new_xi_2)
  %j_6 = add i64 %j_5, 1
  br label %part2_j
part2_j_done:
  %i_5 = add i64 %i_4, 1
  br label %part2_i
part2_i_done:
  br label %part3_i
part3_i:
  %cond_7 = phi i1 [ %cond_8, %part3_i_body ], [ %cond_5, %part2_i_done ]
  %i_7 = phi i64 [ %i_8, %part3_i_body ], [ 0, %part2_i_done ]
  %cond_8 = icmp slt i64 %i_7, 400
  br i1 %cond_8, label %part3_i_body, label %part3_i_done
part3_i_body:
  %xi_1 = call i64 @__vector_get(i64* %x_0, i64 %i_7)
  %zi_0 = call i64 @__vector_get(i64* %z_0, i64 %i_7)
  %new_xi_3 = add i64 %xi_1, %zi_0
  call void @__vector_set(i64* %x_0, i64 %i_7, i64 %new_xi_3)
  %i_8 = add i64 %i_7, 1
  br label %part3_i
part3_i_done:
  br label %part4_i
part4_i:
  %j_7 = phi i64 [ %j_9, %part4_j_done ], [ 0, %part3_i_done ]
  %cond_9 = phi i1 [ %cond_12, %part4_j_done ], [ %cond_8, %part3_i_done ]
  %i_10 = phi i64 [ %i_11, %part4_j_done ], [ 0, %part3_i_done ]
  %cond_10 = icmp slt i64 %i_10, 400
  br i1 %cond_10, label %part4_i_body, label %part4_i_done
part4_i_body:
  br label %part4_j
part4_j:
  %j_9 = phi i64 [ %j_10, %part4_j_body ], [ 0, %part4_i_body ]
  %cond_11 = phi i1 [ %cond_12, %part4_j_body ], [ %cond_10, %part4_i_body ]
  %cond_12 = icmp slt i64 %j_9, 400
  br i1 %cond_12, label %part4_j_body, label %part4_j_done
part4_j_body:
  %Aij_1 = call i64 @__matrix_get(i64* %A_0, i64 %i_10, i64 %j_9, i64 400)
  %xj_0 = call i64 @__vector_get(i64* %x_0, i64 %j_9)
  %wi_0 = call i64 @__vector_get(i64* %w_0, i64 %i_10)
  %new_wi_0 = mul i64 %Aij_1, %xj_0
  %new_wi_1 = mul i64 3, %new_wi_0
  %new_wi_2 = add i64 %wi_0, %new_wi_1
  call void @__vector_set(i64* %w_0, i64 %i_10, i64 %new_wi_2)
  %j_10 = add i64 %j_9, 1
  br label %part4_j
part4_j_done:
  %i_11 = add i64 %i_10, 1
  br label %part4_i
part4_i_done:
  call void @__vector_print(i64* %w_0, i64 400)
  %z0 = bitcast i64* %A_0 to i8*
  call void @free(i8* %z0)
  %z1 = bitcast i64* %u1_0 to i8*
  call void @free(i8* %z1)
  %z2 = bitcast i64* %v1_0 to i8*
  call void @free(i8* %z2)
  %z3 = bitcast i64* %u2_0 to i8*
  call void @free(i8* %z3)
  %z4 = bitcast i64* %v2_0 to i8*
  call void @free(i8* %z4)
  %z5 = bitcast i64* %w_0 to i8*
  call void @free(i8* %z5)
  %z6 = bitcast i64* %x_0 to i8*
  call void @free(i8* %z6)
  %z7 = bitcast i64* %y_0 to i8*
  call void @free(i8* %z7)
  %z8 = bitcast i64* %z_0 to i8*
  call void @free(i8* %z8)
  ret void

}


define dso_local void @__init(i64* %A, i64* %u1, i64* %v1, i64* %u2, i64* %v2, i64* %w, i64* %x, i64* %y, i64* %z, i64 %N, i64 %fN) {
pre_entry:
  br label %init_i
init_i:
  %fi_1 = phi i64 [ %fi_2, %init_j_done ], [ 0, %pre_entry ]
  %i_1 = phi i64 [ %i_2, %init_j_done ], [ 0, %pre_entry ]
  %cond_0 = icmp slt i64 %i_1, %N
  br i1 %cond_0, label %init_i_body, label %init_i_done
init_i_body:
  call void @__vector_set(i64* %u1, i64 %i_1, i64 %fi_1)
  %val_0 = add i64 %fi_1, 1
  %val_1 = sdiv i64 %val_0, %fN
  %val_2 = sdiv i64 %val_1, 2
  call void @__vector_set(i64* %u2, i64 %i_1, i64 %val_2)
  %val_3 = add i64 %fi_1, 1
  %val_4 = sdiv i64 %val_3, %fN
  %val_5 = sdiv i64 %val_4, 4
  call void @__vector_set(i64* %v1, i64 %i_1, i64 %val_5)
  %val_6 = add i64 %fi_1, 1
  %val_7 = sdiv i64 %val_6, %fN
  %val_8 = sdiv i64 %val_7, 6
  call void @__vector_set(i64* %v2, i64 %i_1, i64 %val_8)
  %val_9 = add i64 %fi_1, 1
  %val_10 = sdiv i64 %val_9, %fN
  %val_11 = sdiv i64 %val_10, 8
  call void @__vector_set(i64* %y, i64 %i_1, i64 %val_11)
  %val_12 = add i64 %fi_1, 1
  %val_13 = sdiv i64 %val_12, %fN
  %val_14 = sdiv i64 %val_13, 9
  call void @__vector_set(i64* %z, i64 %i_1, i64 %val_14)
  call void @__vector_set(i64* %x, i64 %i_1, i64 0)
  call void @__vector_set(i64* %w, i64 %i_1, i64 0)
  br label %init_j
init_j:
  %fj_1 = phi i64 [ %fj_2, %init_j_body ], [ 0, %init_i_body ]
  %j_1 = phi i64 [ %j_2, %init_j_body ], [ 0, %init_i_body ]
  %val_15 = phi i64 [ %val_18, %init_j_body ], [ %val_14, %init_i_body ]
  %cond_1 = phi i1 [ %cond_2, %init_j_body ], [ %cond_0, %init_i_body ]
  %cond_2 = icmp slt i64 %j_1, %N
  br i1 %cond_2, label %init_j_body, label %init_j_done
init_j_body:
  %val_16 = mul i64 %fi_1, %fj_1
  %val_17 = call i64 @__fmod(i64 %val_16, i64 %fN)
  %val_18 = sdiv i64 %val_17, %fN
  call void @__matrix_set(i64* %A, i64 %i_1, i64 %j_1, i64 %N, i64 %val_18)
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

