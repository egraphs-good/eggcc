
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


define dso_local i1 @__xor(i1 %x, i1 %y) {
pre_entry:
  %xn_0 = xor i1 1, %x
  %yn_0 = xor i1 1, %y
  %xyn_0 = and i1 %x, %yn_0
  %xny_0 = and i1 %xn_0, %y
  %res_0 = or i1 %xyn_0, %xny_0
  ret i1 %res_0

}


define dso_local i1 @__getbit(i64 %x, i64 %position) {
pre_entry:
  br label %loop_cond
loop_cond:
  %i_1 = phi i64 [ %i_2, %loop_body ], [ 0, %pre_entry ]
  %x_1 = phi i64 [ %x_2, %loop_body ], [ %x, %pre_entry ]
  %cond_0 = icmp slt i64 %i_1, %position
  br i1 %cond_0, label %loop_body, label %loop_exit
loop_body:
  %x_2 = sdiv i64 %x_1, 2
  %i_2 = add i64 %i_1, 1
  br label %loop_cond
loop_exit:
  %halfx_0 = sdiv i64 %x_1, 2
  %twohalfx_0 = mul i64 %halfx_0, 2
  %iszero_0 = icmp eq i64 %twohalfx_0, %x_1
  %res_0 = xor i1 1, %iszero_0
  ret i1 %res_0

}


define dso_local void @__rand(i64* %state) {
pre_entry:
  %s_0 = load i64, i64* %state
  %head0_0 = call i1 @__getbit(i64 %s_0, i64 11)
  %head1_0 = call i1 @__getbit(i64 %s_0, i64 13)
  %head2_0 = call i1 @__getbit(i64 %s_0, i64 14)
  %head3_0 = call i1 @__getbit(i64 %s_0, i64 16)
  %fb_0 = call i1 @__xor(i1 %head0_0, i1 %head1_0)
  %fb_1 = call i1 @__xor(i1 %fb_0, i1 %head2_0)
  %fb_2 = call i1 @__xor(i1 %fb_1, i1 %head3_0)
  %s_1 = mul i64 %s_0, 2
  br i1 %fb_2, label %add_one, label %end
add_one:
  %s_2 = add i64 %s_1, 1
  br label %end
end:
  %s_3 = phi i64 [ %s_2, %add_one ], [ %s_1, %pre_entry ]
  store i64 %s_3, i64* %state
  ret void

}


define dso_local i64 @__mod(i64 %x, i64 %m) {
pre_entry:
  %q_0 = sdiv i64 %x, %m
  %multiple_0 = mul i64 %q_0, %m
  %r_0 = sub i64 %x, %multiple_0
  ret i64 %r_0

}


define dso_local void @__gen_uniform_csr(i64 %rows, i64 %cols, i64 %degree, i64* %csr_rowptr, i64* %csr_colidx, i64* %csr_values) {
pre_entry:
  %nnz_0 = mul i64 %degree, %rows
  store i64 0, i64* %csr_rowptr
  br label %loop_gen_rptr_cond
loop_gen_rptr_cond:
  %i_1 = phi i64 [ %i_2, %loop_gen_rptr_body ], [ 1, %pre_entry ]
  %cond_0 = icmp sle i64 %i_1, %rows
  br i1 %cond_0, label %loop_gen_rptr_body, label %loop_gen_rptr_exit
loop_gen_rptr_body:
  %p_0 = getelementptr inbounds i64, i64* %csr_rowptr, i64 %i_1
  %v_0 = mul i64 %degree, %i_1
  store i64 %v_0, i64* %p_0
  %i_2 = add i64 %i_1, 1
  br label %loop_gen_rptr_cond
loop_gen_rptr_exit:
  %colidx_incr_0 = sdiv i64 %cols, %degree
  br label %loop_gen_cidx_cond
loop_gen_cidx_cond:
  %cond_1 = phi i1 [ %cond_2, %loop_gen_cidx_body ], [ %cond_0, %loop_gen_rptr_exit ]
  %i_4 = phi i64 [ %i_5, %loop_gen_cidx_body ], [ 0, %loop_gen_rptr_exit ]
  %cond_2 = icmp slt i64 %i_4, %nnz_0
  br i1 %cond_2, label %loop_gen_cidx_body, label %loop_gen_cidx_exit
loop_gen_cidx_body:
  %rid_0 = sdiv i64 %i_4, %degree
  %v_1 = mul i64 %i_4, %colidx_incr_0
  %v_2 = add i64 %v_1, %rid_0
  %cid_0 = call i64 @__mod(i64 %v_2, i64 %cols)
  %p_1 = getelementptr inbounds i64, i64* %csr_colidx, i64 %i_4
  store i64 %cid_0, i64* %p_1
  %i_5 = add i64 %i_4, 1
  br label %loop_gen_cidx_cond
loop_gen_cidx_exit:
  %z0 = mul i64 1, 8
  %z1 = call i8* @malloc(i64 %z0)
  %rng_0 = bitcast i8* %z1 to i64*
  store i64 72160722, i64* %rng_0
  br label %loop_gen_vals_cond
loop_gen_vals_cond:
  %cond_3 = phi i1 [ %cond_4, %loop_gen_vals_body ], [ %cond_2, %loop_gen_cidx_exit ]
  %i_7 = phi i64 [ %i_8, %loop_gen_vals_body ], [ 0, %loop_gen_cidx_exit ]
  %cond_4 = icmp slt i64 %i_7, %nnz_0
  br i1 %cond_4, label %loop_gen_vals_body, label %loop_gen_vals_exit
loop_gen_vals_body:
  call void @__rand(i64* %rng_0)
  %v_3 = load i64, i64* %rng_0
  %v_4 = call i64 @__mod(i64 %v_3, i64 10)
  %p_2 = getelementptr inbounds i64, i64* %csr_values, i64 %i_7
  store i64 %v_4, i64* %p_2
  %i_8 = add i64 %i_7, 1
  br label %loop_gen_vals_cond
loop_gen_vals_exit:
  %z2 = bitcast i64* %rng_0 to i8*
  call void @free(i8* %z2)
  ret void

}


define dso_local void @__gen_vec(i64 %len, i64* %data) {
pre_entry:
  %z0 = mul i64 1, 8
  %z1 = call i8* @malloc(i64 %z0)
  %rng_0 = bitcast i8* %z1 to i64*
  store i64 85817256, i64* %rng_0
  br label %loop_cond
loop_cond:
  %i_1 = phi i64 [ %i_2, %loop_body ], [ 0, %pre_entry ]
  %cond_0 = icmp slt i64 %i_1, %len
  br i1 %cond_0, label %loop_body, label %loop_exit
loop_body:
  call void @__rand(i64* %rng_0)
  %v_0 = load i64, i64* %rng_0
  %v_1 = call i64 @__mod(i64 %v_0, i64 10)
  %p_0 = getelementptr inbounds i64, i64* %data, i64 %i_1
  store i64 %v_1, i64* %p_0
  %i_2 = add i64 %i_1, 1
  br label %loop_cond
loop_exit:
  %z2 = bitcast i64* %rng_0 to i8*
  call void @free(i8* %z2)
  ret void

}


define dso_local void @__csr_spmv(i64 %rows, i64 %cols, i64* %csr_rowptr, i64* %csr_colidx, i64* %csr_values, i64* %vec, i64* %res) {
pre_entry:
  br label %loop_init_cond
loop_init_cond:
  %i_1 = phi i64 [ %i_2, %loop_init_body ], [ 0, %pre_entry ]
  %cond_0 = icmp slt i64 %i_1, %rows
  br i1 %cond_0, label %loop_init_body, label %loop_init_exit
loop_init_body:
  %p_0 = getelementptr inbounds i64, i64* %res, i64 %i_1
  store i64 0, i64* %p_0
  %i_2 = add i64 %i_1, 1
  br label %loop_init_cond
loop_init_exit:
  br label %loop_rows_cond
loop_rows_cond:
  %rid_1 = phi i64 [ %rid_2, %loop_nnzs_exit ], [ 0, %loop_init_exit ]
  %cond_1 = phi i1 [ %cond_4, %loop_nnzs_exit ], [ %cond_0, %loop_init_exit ]
  %cond_2 = icmp slt i64 %rid_1, %rows
  br i1 %cond_2, label %loop_rows_body, label %loop_rows_exit
loop_rows_body:
  %p_1 = getelementptr inbounds i64, i64* %csr_rowptr, i64 %rid_1
  %start_0 = load i64, i64* %p_1
  %p_2 = getelementptr inbounds i64, i64* %p_1, i64 1
  %end_0 = load i64, i64* %p_2
  %j_0 = add i64 %start_0, 0
  br label %loop_nnzs_cond
loop_nnzs_cond:
  %j_1 = phi i64 [ %j_2, %loop_nnzs_body ], [ %j_0, %loop_rows_body ]
  %p_3 = phi i64* [ %p_7, %loop_nnzs_body ], [ %p_2, %loop_rows_body ]
  %cond_3 = phi i1 [ %cond_4, %loop_nnzs_body ], [ %cond_2, %loop_rows_body ]
  %cond_4 = icmp slt i64 %j_1, %end_0
  br i1 %cond_4, label %loop_nnzs_body, label %loop_nnzs_exit
loop_nnzs_body:
  %p_4 = getelementptr inbounds i64, i64* %csr_colidx, i64 %j_1
  %cid_0 = load i64, i64* %p_4
  %p_5 = getelementptr inbounds i64, i64* %csr_values, i64 %j_1
  %mat_val_0 = load i64, i64* %p_5
  %p_6 = getelementptr inbounds i64, i64* %vec, i64 %cid_0
  %vec_val_0 = load i64, i64* %p_6
  %p_7 = getelementptr inbounds i64, i64* %res, i64 %rid_1
  %acc_0 = load i64, i64* %p_7
  %incr_0 = mul i64 %mat_val_0, %vec_val_0
  %acc_1 = add i64 %acc_0, %incr_0
  store i64 %acc_1, i64* %p_7
  %j_2 = add i64 %j_1, 1
  br label %loop_nnzs_cond
loop_nnzs_exit:
  %rid_2 = add i64 %rid_1, 1
  br label %loop_rows_cond
loop_rows_exit:
  ret void

}


define dso_local void @__print_arr(i64* %arr, i64 %size) {
pre_entry:
  br label %loop_cond
loop_cond:
  %i_1 = phi i64 [ %i_2, %loop_body ], [ 0, %pre_entry ]
  %cond_0 = icmp slt i64 %i_1, %size
  br i1 %cond_0, label %loop_body, label %loop_exit
loop_body:
  %p_0 = getelementptr inbounds i64, i64* %arr, i64 %i_1
  %v_0 = load i64, i64* %p_0
  call void @print_int(i64 %v_0)
  call void @print_newline()
  %i_2 = add i64 %i_1, 1
  br label %loop_cond
loop_exit:
  ret void

}


define dso_local void @__main() {
b0:
  br label %loop_cond
loop_cond:
  %loop_counter_1 = phi i64 [ %loop_counter_2, %loop_body ], [ 10, %b0 ]
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 500
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
  %rptr_len_0 = add i64 %n, 1
  %nnz_0 = mul i64 %n, 5
  %z0 = mul i64 %rptr_len_0, 8
  %z1 = call i8* @malloc(i64 %z0)
  %csr_rowptr_0 = bitcast i8* %z1 to i64*
  %z2 = mul i64 %nnz_0, 8
  %z3 = call i8* @malloc(i64 %z2)
  %csr_colidx_0 = bitcast i8* %z3 to i64*
  %z4 = mul i64 %nnz_0, 8
  %z5 = call i8* @malloc(i64 %z4)
  %csr_values_0 = bitcast i8* %z5 to i64*
  call void @__gen_uniform_csr(i64 %n, i64 %n, i64 5, i64* %csr_rowptr_0, i64* %csr_colidx_0, i64* %csr_values_0)
  call void @__print_arr(i64* %csr_rowptr_0, i64 %rptr_len_0)
  call void @__print_arr(i64* %csr_colidx_0, i64 %nnz_0)
  call void @__print_arr(i64* %csr_values_0, i64 %nnz_0)
  %z6 = mul i64 %n, 8
  %z7 = call i8* @malloc(i64 %z6)
  %vec_0 = bitcast i8* %z7 to i64*
  call void @__gen_vec(i64 %n, i64* %vec_0)
  call void @__print_arr(i64* %vec_0, i64 %n)
  %z8 = mul i64 %n, 8
  %z9 = call i8* @malloc(i64 %z8)
  %res_0 = bitcast i8* %z9 to i64*
  call void @__csr_spmv(i64 %n, i64 %n, i64* %csr_rowptr_0, i64* %csr_colidx_0, i64* %csr_values_0, i64* %vec_0, i64* %res_0)
  call void @__print_arr(i64* %res_0, i64 %n)
  %z10 = bitcast i64* %csr_rowptr_0 to i8*
  call void @free(i8* %z10)
  %z11 = bitcast i64* %csr_colidx_0 to i8*
  call void @free(i8* %z11)
  %z12 = bitcast i64* %csr_values_0 to i8*
  call void @free(i8* %z12)
  %z13 = bitcast i64* %vec_0 to i8*
  call void @free(i8* %z13)
  %z14 = bitcast i64* %res_0 to i8*
  call void @free(i8* %z14)
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

