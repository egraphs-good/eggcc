; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpgHRt8i/postprocessed.ll'
source_filename = "stdin"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"
target triple = "arm64-apple-macosx13.0.0"

@.str = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.str.1 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@.str.2 = private unnamed_addr constant [4 x i8] c"%ld\00", align 1
@.str.3 = private unnamed_addr constant [9 x i8] c"[object]\00", align 1
@.str.4 = private unnamed_addr constant [33 x i8] c"error: expected %d args, got %d\0A\00", align 1

declare dso_local i32 @putchar(i32)

declare dso_local i32 @printf(ptr, ...)

declare dso_local void @exit(i32)

declare dso_local i64 @atol(ptr)

declare dso_local noalias ptr @malloc(i64)

declare dso_local void @free(ptr)

define dso_local i32 @btoi(ptr %0) {
  %2 = load i8, ptr %0, align 1
  %3 = icmp eq i8 %2, 116
  %4 = zext i1 %3 to i32
  ret i32 %4
}

define dso_local void @print_bool(i1 %0) {
  br i1 %0, label %2, label %4

2:                                                ; preds = %1
  %3 = call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str)
  br label %6

4:                                                ; preds = %1
  %5 = call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str.1)
  br label %6

6:                                                ; preds = %4, %2
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
  %2 = call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str.2, i64 %0)
  ret void
}

define dso_local void @print_ptr(ptr %0) {
  %2 = call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str.3)
  ret void
}

define dso_local i1 @__xor(i1 %x, i1 %y) {
pre_entry:
  %res_0 = xor i1 %x, %y
  ret i1 %res_0
}

define dso_local i1 @__getbit(i64 %x, i64 %position) {
pre_entry:
  br label %loop_cond

loop_cond:                                        ; preds = %loop_body, %pre_entry
  %i_1 = phi i64 [ %i_2, %loop_body ], [ 0, %pre_entry ]
  %x_1 = phi i64 [ %x_2, %loop_body ], [ %x, %pre_entry ]
  %cond_0 = icmp slt i64 %i_1, %position
  br i1 %cond_0, label %loop_body, label %loop_exit

loop_body:                                        ; preds = %loop_cond
  %x_2 = sdiv i64 %x_1, 2
  %i_2 = add i64 %i_1, 1
  br label %loop_cond

loop_exit:                                        ; preds = %loop_cond
  %x_1.lcssa = phi i64 [ %x_1, %loop_cond ]
  %halfx_0 = sdiv i64 %x_1.lcssa, 2
  %twohalfx_0 = shl nsw i64 %halfx_0, 1
  %iszero_0 = icmp ne i64 %twohalfx_0, %x_1.lcssa
  ret i1 %iszero_0
}

define dso_local void @__rand(ptr %state) {
pre_entry:
  %s_0 = load i64, ptr %state, align 8
  %head0_0 = call i1 @__getbit(i64 %s_0, i64 11)
  %head1_0 = call i1 @__getbit(i64 %s_0, i64 13)
  %head2_0 = call i1 @__getbit(i64 %s_0, i64 14)
  %head3_0 = call i1 @__getbit(i64 %s_0, i64 16)
  %fb_0 = call i1 @__xor(i1 %head0_0, i1 %head1_0)
  %fb_1 = call i1 @__xor(i1 %fb_0, i1 %head2_0)
  %fb_2 = call i1 @__xor(i1 %fb_1, i1 %head3_0)
  %s_1 = shl i64 %s_0, 1
  br i1 %fb_2, label %add_one, label %end

add_one:                                          ; preds = %pre_entry
  %s_2 = or disjoint i64 %s_1, 1
  br label %end

end:                                              ; preds = %add_one, %pre_entry
  %s_3 = phi i64 [ %s_2, %add_one ], [ %s_1, %pre_entry ]
  store i64 %s_3, ptr %state, align 8
  ret void
}

define dso_local i64 @__mod(i64 %x, i64 %m) {
pre_entry:
  %x.fr = freeze i64 %x
  %0 = srem i64 %x.fr, %m
  ret i64 %0
}

define dso_local void @__gen_uniform_csr(i64 %rows, i64 %cols, i64 %degree, ptr %csr_rowptr, ptr %csr_colidx, ptr %csr_values) {
pre_entry:
  %nnz_0 = mul i64 %degree, %rows
  store i64 0, ptr %csr_rowptr, align 8
  br label %loop_gen_rptr_cond

loop_gen_rptr_cond:                               ; preds = %loop_gen_rptr_body, %pre_entry
  %i_1 = phi i64 [ %i_2, %loop_gen_rptr_body ], [ 1, %pre_entry ]
  %cond_0.not = icmp sgt i64 %i_1, %rows
  br i1 %cond_0.not, label %loop_gen_rptr_exit, label %loop_gen_rptr_body

loop_gen_rptr_body:                               ; preds = %loop_gen_rptr_cond
  %p_0 = getelementptr inbounds i64, ptr %csr_rowptr, i64 %i_1
  %v_0 = mul i64 %i_1, %degree
  store i64 %v_0, ptr %p_0, align 8
  %i_2 = add i64 %i_1, 1
  br label %loop_gen_rptr_cond

loop_gen_rptr_exit:                               ; preds = %loop_gen_rptr_cond
  %colidx_incr_0 = sdiv i64 %cols, %degree
  br label %loop_gen_cidx_cond

loop_gen_cidx_cond:                               ; preds = %loop_gen_cidx_body, %loop_gen_rptr_exit
  %i_4 = phi i64 [ %i_5, %loop_gen_cidx_body ], [ 0, %loop_gen_rptr_exit ]
  %cond_2 = icmp slt i64 %i_4, %nnz_0
  br i1 %cond_2, label %loop_gen_cidx_body, label %loop_gen_cidx_exit

loop_gen_cidx_body:                               ; preds = %loop_gen_cidx_cond
  %rid_0 = sdiv i64 %i_4, %degree
  %v_1 = mul i64 %i_4, %colidx_incr_0
  %v_2 = add i64 %v_1, %rid_0
  %cid_0 = call i64 @__mod(i64 %v_2, i64 %cols)
  %p_1 = getelementptr inbounds i64, ptr %csr_colidx, i64 %i_4
  store i64 %cid_0, ptr %p_1, align 8
  %i_5 = add i64 %i_4, 1
  br label %loop_gen_cidx_cond

loop_gen_cidx_exit:                               ; preds = %loop_gen_cidx_cond
  %z1 = call ptr @malloc(i64 8)
  store i64 72160722, ptr %z1, align 8
  br label %loop_gen_vals_cond

loop_gen_vals_cond:                               ; preds = %loop_gen_vals_body, %loop_gen_cidx_exit
  %i_7 = phi i64 [ %i_8, %loop_gen_vals_body ], [ 0, %loop_gen_cidx_exit ]
  %cond_4 = icmp slt i64 %i_7, %nnz_0
  br i1 %cond_4, label %loop_gen_vals_body, label %loop_gen_vals_exit

loop_gen_vals_body:                               ; preds = %loop_gen_vals_cond
  call void @__rand(ptr nonnull %z1)
  %v_3 = load i64, ptr %z1, align 8
  %v_4 = call i64 @__mod(i64 %v_3, i64 10)
  %p_2 = getelementptr inbounds i64, ptr %csr_values, i64 %i_7
  store i64 %v_4, ptr %p_2, align 8
  %i_8 = add i64 %i_7, 1
  br label %loop_gen_vals_cond

loop_gen_vals_exit:                               ; preds = %loop_gen_vals_cond
  call void @free(ptr nonnull %z1)
  ret void
}

define dso_local void @__gen_vec(i64 %len, ptr %data) {
pre_entry:
  %z1 = call ptr @malloc(i64 8)
  store i64 85817256, ptr %z1, align 8
  br label %loop_cond

loop_cond:                                        ; preds = %loop_body, %pre_entry
  %i_1 = phi i64 [ %i_2, %loop_body ], [ 0, %pre_entry ]
  %cond_0 = icmp slt i64 %i_1, %len
  br i1 %cond_0, label %loop_body, label %loop_exit

loop_body:                                        ; preds = %loop_cond
  call void @__rand(ptr nonnull %z1)
  %v_0 = load i64, ptr %z1, align 8
  %v_1 = call i64 @__mod(i64 %v_0, i64 10)
  %p_0 = getelementptr inbounds i64, ptr %data, i64 %i_1
  store i64 %v_1, ptr %p_0, align 8
  %i_2 = add i64 %i_1, 1
  br label %loop_cond

loop_exit:                                        ; preds = %loop_cond
  call void @free(ptr nonnull %z1)
  ret void
}

define dso_local void @__csr_spmv(i64 %rows, i64 %cols, ptr %csr_rowptr, ptr %csr_colidx, ptr %csr_values, ptr %vec, ptr %res) {
pre_entry:
  br label %loop_init_cond

loop_init_cond:                                   ; preds = %loop_init_body, %pre_entry
  %i_1 = phi i64 [ %i_2, %loop_init_body ], [ 0, %pre_entry ]
  %cond_0 = icmp slt i64 %i_1, %rows
  br i1 %cond_0, label %loop_init_body, label %loop_init_exit

loop_init_body:                                   ; preds = %loop_init_cond
  %p_0 = getelementptr inbounds i64, ptr %res, i64 %i_1
  store i64 0, ptr %p_0, align 8
  %i_2 = add i64 %i_1, 1
  br label %loop_init_cond

loop_init_exit:                                   ; preds = %loop_init_cond
  br label %loop_rows_cond

loop_rows_cond:                                   ; preds = %loop_nnzs_exit, %loop_init_exit
  %rid_1 = phi i64 [ %rid_2, %loop_nnzs_exit ], [ 0, %loop_init_exit ]
  %cond_2 = icmp slt i64 %rid_1, %rows
  br i1 %cond_2, label %loop_rows_body, label %loop_rows_exit

loop_rows_body:                                   ; preds = %loop_rows_cond
  %p_1 = getelementptr inbounds i64, ptr %csr_rowptr, i64 %rid_1
  %start_0 = load i64, ptr %p_1, align 8
  %p_2 = getelementptr inbounds i64, ptr %p_1, i64 1
  %end_0 = load i64, ptr %p_2, align 8
  br label %loop_nnzs_cond

loop_nnzs_cond:                                   ; preds = %loop_nnzs_body, %loop_rows_body
  %j_1 = phi i64 [ %j_2, %loop_nnzs_body ], [ %start_0, %loop_rows_body ]
  %cond_4 = icmp slt i64 %j_1, %end_0
  br i1 %cond_4, label %loop_nnzs_body, label %loop_nnzs_exit

loop_nnzs_body:                                   ; preds = %loop_nnzs_cond
  %p_4 = getelementptr inbounds i64, ptr %csr_colidx, i64 %j_1
  %cid_0 = load i64, ptr %p_4, align 8
  %p_5 = getelementptr inbounds i64, ptr %csr_values, i64 %j_1
  %mat_val_0 = load i64, ptr %p_5, align 8
  %p_6 = getelementptr inbounds i64, ptr %vec, i64 %cid_0
  %vec_val_0 = load i64, ptr %p_6, align 8
  %p_7 = getelementptr inbounds i64, ptr %res, i64 %rid_1
  %acc_0 = load i64, ptr %p_7, align 8
  %incr_0 = mul i64 %mat_val_0, %vec_val_0
  %acc_1 = add i64 %acc_0, %incr_0
  store i64 %acc_1, ptr %p_7, align 8
  %j_2 = add i64 %j_1, 1
  br label %loop_nnzs_cond

loop_nnzs_exit:                                   ; preds = %loop_nnzs_cond
  %rid_2 = add i64 %rid_1, 1
  br label %loop_rows_cond

loop_rows_exit:                                   ; preds = %loop_rows_cond
  ret void
}

define dso_local void @__print_arr(ptr %arr, i64 %size) {
pre_entry:
  br label %loop_cond

loop_cond:                                        ; preds = %loop_body, %pre_entry
  %i_1 = phi i64 [ %i_2, %loop_body ], [ 0, %pre_entry ]
  %cond_0 = icmp slt i64 %i_1, %size
  br i1 %cond_0, label %loop_body, label %loop_exit

loop_body:                                        ; preds = %loop_cond
  %p_0 = getelementptr inbounds i64, ptr %arr, i64 %i_1
  %v_0 = load i64, ptr %p_0, align 8
  call void @print_int(i64 %v_0)
  call void @print_newline()
  %i_2 = add i64 %i_1, 1
  br label %loop_cond

loop_exit:                                        ; preds = %loop_cond
  ret void
}

define dso_local void @__main() {
b0:
  br label %loop_cond

loop_cond:                                        ; preds = %loop_body, %b0
  %loop_counter_1 = phi i64 [ %loop_counter_2, %loop_body ], [ 10, %b0 ]
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 500
  br i1 %loop_cond_0, label %loop_body, label %loop_done

loop_body:                                        ; preds = %loop_cond
  call void @__orig_main(i64 %loop_counter_1)
  %loop_counter_2 = add i64 %loop_counter_1, 1
  br label %loop_cond

loop_done:                                        ; preds = %loop_cond
  ret void
}

define dso_local void @__orig_main(i64 %n) {
pre_entry:
  %rptr_len_0 = add i64 %n, 1
  %nnz_0 = mul i64 %n, 5
  %z0 = shl i64 %rptr_len_0, 3
  %z1 = call ptr @malloc(i64 %z0)
  %z2 = mul i64 %n, 40
  %z3 = call ptr @malloc(i64 %z2)
  %z4 = mul i64 %n, 40
  %z5 = call ptr @malloc(i64 %z4)
  call void @__gen_uniform_csr(i64 %n, i64 %n, i64 5, ptr %z1, ptr %z3, ptr %z5)
  call void @__print_arr(ptr %z1, i64 %rptr_len_0)
  call void @__print_arr(ptr %z3, i64 %nnz_0)
  call void @__print_arr(ptr %z5, i64 %nnz_0)
  %z6 = shl i64 %n, 3
  %z7 = call ptr @malloc(i64 %z6)
  call void @__gen_vec(i64 %n, ptr %z7)
  call void @__print_arr(ptr %z7, i64 %n)
  %z8 = shl i64 %n, 3
  %z9 = call ptr @malloc(i64 %z8)
  call void @__csr_spmv(i64 %n, i64 %n, ptr %z1, ptr %z3, ptr %z5, ptr %z7, ptr %z9)
  call void @__print_arr(ptr %z9, i64 %n)
  call void @free(ptr %z1)
  call void @free(ptr %z3)
  call void @free(ptr %z5)
  call void @free(ptr %z7)
  call void @free(ptr %z9)
  ret void
}

define dso_local i32 @main(i32 %argc, ptr %argv) {
  %.not = icmp eq i32 %argc, 1
  br i1 %.not, label %4, label %1

1:                                                ; preds = %0
  %2 = add nsw i32 %argc, -1
  %3 = call i32 (ptr, ...) @printf(ptr noundef nonnull dereferenceable(1) @.str.4, i32 0, i32 %2)
  call void @exit(i32 2)
  unreachable

4:                                                ; preds = %0
  call void @__main()
  ret i32 0
}
