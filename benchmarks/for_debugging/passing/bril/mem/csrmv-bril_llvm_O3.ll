; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmp9H3jNy/csrmv-init.ll'
source_filename = "stdin"
target datalayout = "e-m:o-i64:64-i128:128-n32:64-S128"
target triple = "arm64-apple-macosx13.0.0"

@.str = private unnamed_addr constant [5 x i8] c"true\00", align 1
@.str.1 = private unnamed_addr constant [6 x i8] c"false\00", align 1
@.str.2 = private unnamed_addr constant [4 x i8] c"%ld\00", align 1
@.str.3 = private unnamed_addr constant [9 x i8] c"[object]\00", align 1
@.str.4 = private unnamed_addr constant [33 x i8] c"error: expected %d args, got %d\0A\00", align 1

; Function Attrs: nofree nounwind
declare dso_local noundef i32 @putchar(i32 noundef) local_unnamed_addr #0

; Function Attrs: nofree nounwind
declare dso_local noundef i32 @printf(ptr nocapture noundef readonly, ...) local_unnamed_addr #0

declare dso_local void @exit(i32) local_unnamed_addr

; Function Attrs: mustprogress nofree nounwind willreturn allockind("alloc,uninitialized") allocsize(0) memory(inaccessiblemem: readwrite)
declare dso_local noalias noundef ptr @malloc(i64 noundef) local_unnamed_addr #1

; Function Attrs: mustprogress nounwind willreturn allockind("free") memory(argmem: readwrite, inaccessiblemem: readwrite)
declare dso_local void @free(ptr allocptr nocapture noundef) local_unnamed_addr #2

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read)
define dso_local i32 @btoi(ptr nocapture readonly %0) local_unnamed_addr #3 {
  %2 = load i8, ptr %0, align 1
  %3 = icmp eq i8 %2, 116
  %4 = zext i1 %3 to i32
  ret i32 %4
}

; Function Attrs: nofree nounwind
define dso_local void @print_bool(i1 %0) local_unnamed_addr #0 {
  %.str..str.1 = select i1 %0, ptr @.str, ptr @.str.1
  %2 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) %.str..str.1)
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @print_space() local_unnamed_addr #0 {
  %1 = tail call i32 @putchar(i32 32)
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @print_newline() local_unnamed_addr #0 {
  %1 = tail call i32 @putchar(i32 10)
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @print_int(i64 %0) local_unnamed_addr #0 {
  %2 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %0)
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @print_ptr(ptr nocapture readnone %0) local_unnamed_addr #0 {
  %2 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.3)
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define dso_local i1 @__xor(i1 %x, i1 %y) local_unnamed_addr #4 {
pre_entry:
  %res_0 = xor i1 %x, %y
  ret i1 %res_0
}

; Function Attrs: nofree norecurse nosync nounwind memory(none)
define dso_local i1 @__getbit(i64 %x, i64 %position) local_unnamed_addr #5 {
pre_entry:
  %cond_01 = icmp sgt i64 %position, 0
  br i1 %cond_01, label %loop_body, label %loop_exit

loop_body:                                        ; preds = %pre_entry, %loop_body
  %x_13 = phi i64 [ %x_2, %loop_body ], [ %x, %pre_entry ]
  %i_12 = phi i64 [ %i_2, %loop_body ], [ 0, %pre_entry ]
  %x_2 = sdiv i64 %x_13, 2
  %i_2 = add nuw nsw i64 %i_12, 1
  %exitcond.not = icmp eq i64 %i_2, %position
  br i1 %exitcond.not, label %loop_exit, label %loop_body

loop_exit:                                        ; preds = %loop_body, %pre_entry
  %x_1.lcssa = phi i64 [ %x, %pre_entry ], [ %x_2, %loop_body ]
  %halfx_0 = sdiv i64 %x_1.lcssa, 2
  %twohalfx_0 = shl nsw i64 %halfx_0, 1
  %iszero_0 = icmp ne i64 %twohalfx_0, %x_1.lcssa
  ret i1 %iszero_0
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: readwrite)
define dso_local void @__rand(ptr nocapture %state) local_unnamed_addr #6 {
pre_entry:
  %s_0 = load i64, ptr %state, align 8
  %x_2.i4.10 = sdiv i64 %s_0, 2048
  %x_2.i4.12 = sdiv i64 %s_0, 8192
  %x_2.i14.13 = sdiv i64 %s_0, 16384
  %x_2.i24.15 = sdiv i64 %s_0, 65536
  %halfx_0.i17 = sdiv i64 %s_0, 32768
  %twohalfx_0.i18 = shl nsw i64 %halfx_0.i17, 1
  %iszero_0.i19 = icmp ne i64 %twohalfx_0.i18, %x_2.i14.13
  %halfx_0.i7 = sdiv i64 %s_0, 16384
  %twohalfx_0.i8 = shl nsw i64 %halfx_0.i7, 1
  %iszero_0.i9 = icmp ne i64 %twohalfx_0.i8, %x_2.i4.12
  %halfx_0.i = sdiv i64 %s_0, 4096
  %twohalfx_0.i = shl nsw i64 %halfx_0.i, 1
  %iszero_0.i = icmp ne i64 %twohalfx_0.i, %x_2.i4.10
  %halfx_0.i27 = sdiv i64 %s_0, 131072
  %twohalfx_0.i28 = shl nsw i64 %halfx_0.i27, 1
  %iszero_0.i29 = icmp ne i64 %twohalfx_0.i28, %x_2.i24.15
  %res_0.i = xor i1 %iszero_0.i, %iszero_0.i9
  %res_0.i31 = xor i1 %res_0.i, %iszero_0.i19
  %res_0.i32 = xor i1 %res_0.i31, %iszero_0.i29
  %s_1 = shl i64 %s_0, 1
  %s_2 = zext i1 %res_0.i32 to i64
  %spec.select = or disjoint i64 %s_1, %s_2
  store i64 %spec.select, ptr %state, align 8
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(none)
define dso_local i64 @__mod(i64 %x, i64 %m) local_unnamed_addr #4 {
pre_entry:
  %x.fr = freeze i64 %x
  %0 = srem i64 %x.fr, %m
  ret i64 %0
}

; Function Attrs: nofree norecurse nosync nounwind memory(argmem: write)
define dso_local void @__gen_uniform_csr(i64 %rows, i64 %cols, i64 %degree, ptr nocapture writeonly %csr_rowptr, ptr nocapture writeonly %csr_colidx, ptr nocapture writeonly %csr_values) local_unnamed_addr #7 {
pre_entry:
  %nnz_0 = mul i64 %degree, %rows
  store i64 0, ptr %csr_rowptr, align 8
  %cond_0.not2 = icmp slt i64 %rows, 1
  br i1 %cond_0.not2, label %loop_gen_rptr_exit, label %loop_gen_rptr_body

loop_gen_rptr_body:                               ; preds = %pre_entry, %loop_gen_rptr_body
  %i_13 = phi i64 [ %i_2, %loop_gen_rptr_body ], [ 1, %pre_entry ]
  %p_0 = getelementptr inbounds i64, ptr %csr_rowptr, i64 %i_13
  %v_0 = mul i64 %i_13, %degree
  store i64 %v_0, ptr %p_0, align 8
  %i_2 = add i64 %i_13, 1
  %cond_0.not = icmp sgt i64 %i_2, %rows
  br i1 %cond_0.not, label %loop_gen_rptr_exit, label %loop_gen_rptr_body

loop_gen_rptr_exit:                               ; preds = %loop_gen_rptr_body, %pre_entry
  %colidx_incr_0 = sdiv i64 %cols, %degree
  %cond_24 = icmp sgt i64 %nnz_0, 0
  br i1 %cond_24, label %loop_gen_cidx_body.preheader, label %loop_gen_vals_exit

loop_gen_cidx_body.preheader:                     ; preds = %loop_gen_rptr_exit
  %min.iters.check = icmp ult i64 %nnz_0, 4
  br i1 %min.iters.check, label %loop_gen_cidx_body.preheader10, label %vector.ph

vector.ph:                                        ; preds = %loop_gen_cidx_body.preheader
  %n.vec = and i64 %nnz_0, 9223372036854775804
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %0 = or disjoint i64 %index, 1
  %1 = or disjoint i64 %index, 2
  %2 = or disjoint i64 %index, 3
  %3 = sdiv i64 %index, %degree
  %4 = sdiv i64 %0, %degree
  %5 = sdiv i64 %1, %degree
  %6 = sdiv i64 %2, %degree
  %7 = mul i64 %index, %colidx_incr_0
  %8 = mul i64 %0, %colidx_incr_0
  %9 = mul i64 %1, %colidx_incr_0
  %10 = mul i64 %2, %colidx_incr_0
  %11 = add i64 %3, %7
  %12 = add i64 %4, %8
  %13 = add i64 %5, %9
  %14 = add i64 %6, %10
  %15 = freeze i64 %11
  %16 = freeze i64 %12
  %17 = freeze i64 %13
  %18 = freeze i64 %14
  %19 = srem i64 %15, %cols
  %20 = srem i64 %16, %cols
  %21 = srem i64 %17, %cols
  %22 = srem i64 %18, %cols
  %23 = getelementptr inbounds i64, ptr %csr_colidx, i64 %index
  %24 = getelementptr inbounds i64, ptr %csr_colidx, i64 %0
  %25 = getelementptr inbounds i64, ptr %csr_colidx, i64 %1
  %26 = getelementptr inbounds i64, ptr %csr_colidx, i64 %2
  store i64 %19, ptr %23, align 8
  store i64 %20, ptr %24, align 8
  store i64 %21, ptr %25, align 8
  store i64 %22, ptr %26, align 8
  %index.next = add nuw i64 %index, 4
  %27 = icmp eq i64 %index.next, %n.vec
  br i1 %27, label %middle.block, label %vector.body, !llvm.loop !0

middle.block:                                     ; preds = %vector.body
  %cmp.n = icmp eq i64 %nnz_0, %n.vec
  br i1 %cmp.n, label %loop_gen_cidx_exit, label %loop_gen_cidx_body.preheader10

loop_gen_cidx_body.preheader10:                   ; preds = %loop_gen_cidx_body.preheader, %middle.block
  %i_45.ph = phi i64 [ 0, %loop_gen_cidx_body.preheader ], [ %n.vec, %middle.block ]
  br label %loop_gen_cidx_body

loop_gen_cidx_body:                               ; preds = %loop_gen_cidx_body.preheader10, %loop_gen_cidx_body
  %i_45 = phi i64 [ %i_5, %loop_gen_cidx_body ], [ %i_45.ph, %loop_gen_cidx_body.preheader10 ]
  %rid_0 = sdiv i64 %i_45, %degree
  %v_1 = mul i64 %i_45, %colidx_incr_0
  %v_2 = add i64 %rid_0, %v_1
  %x.fr.i = freeze i64 %v_2
  %28 = srem i64 %x.fr.i, %cols
  %p_1 = getelementptr inbounds i64, ptr %csr_colidx, i64 %i_45
  store i64 %28, ptr %p_1, align 8
  %i_5 = add nuw nsw i64 %i_45, 1
  %exitcond.not = icmp eq i64 %i_5, %nnz_0
  br i1 %exitcond.not, label %loop_gen_cidx_exit, label %loop_gen_cidx_body, !llvm.loop !3

loop_gen_cidx_exit:                               ; preds = %loop_gen_cidx_body, %middle.block
  br i1 %cond_24, label %loop_gen_vals_body, label %loop_gen_vals_exit

loop_gen_vals_body:                               ; preds = %loop_gen_cidx_exit, %loop_gen_vals_body
  %x.fr.i18 = phi i64 [ %spec.select.i, %loop_gen_vals_body ], [ 72160722, %loop_gen_cidx_exit ]
  %i_77 = phi i64 [ %i_8, %loop_gen_vals_body ], [ 0, %loop_gen_cidx_exit ]
  %x_2.i4.10.i = sdiv i64 %x.fr.i18, 2048
  %x_2.i4.12.i = sdiv i64 %x.fr.i18, 8192
  %x_2.i14.13.i = sdiv i64 %x.fr.i18, 16384
  %x_2.i24.15.i = sdiv i64 %x.fr.i18, 65536
  %halfx_0.i17.i = sdiv i64 %x.fr.i18, 32768
  %twohalfx_0.i18.i = shl nsw i64 %halfx_0.i17.i, 1
  %iszero_0.i19.i = icmp ne i64 %twohalfx_0.i18.i, %x_2.i14.13.i
  %twohalfx_0.i8.i = shl nsw i64 %x_2.i14.13.i, 1
  %iszero_0.i9.i = icmp ne i64 %twohalfx_0.i8.i, %x_2.i4.12.i
  %halfx_0.i.i = sdiv i64 %x.fr.i18, 4096
  %twohalfx_0.i.i = shl nsw i64 %halfx_0.i.i, 1
  %iszero_0.i.i = icmp ne i64 %twohalfx_0.i.i, %x_2.i4.10.i
  %halfx_0.i27.i = sdiv i64 %x.fr.i18, 131072
  %twohalfx_0.i28.i = shl nsw i64 %halfx_0.i27.i, 1
  %iszero_0.i29.i = icmp ne i64 %twohalfx_0.i28.i, %x_2.i24.15.i
  %res_0.i.i = xor i1 %iszero_0.i.i, %iszero_0.i9.i
  %res_0.i31.i = xor i1 %iszero_0.i19.i, %res_0.i.i
  %res_0.i32.i = xor i1 %iszero_0.i29.i, %res_0.i31.i
  %s_1.i = shl i64 %x.fr.i18, 1
  %res_0.i32.i.fr = freeze i1 %res_0.i32.i
  %s_2.i = zext i1 %res_0.i32.i.fr to i64
  %spec.select.i = or disjoint i64 %s_1.i, %s_2.i
  %29 = srem i64 %spec.select.i, 10
  %p_2 = getelementptr inbounds i64, ptr %csr_values, i64 %i_77
  store i64 %29, ptr %p_2, align 8
  %i_8 = add nuw nsw i64 %i_77, 1
  %exitcond9.not = icmp eq i64 %i_8, %nnz_0
  br i1 %exitcond9.not, label %loop_gen_vals_exit, label %loop_gen_vals_body

loop_gen_vals_exit:                               ; preds = %loop_gen_vals_body, %loop_gen_rptr_exit, %loop_gen_cidx_exit
  ret void
}

; Function Attrs: nofree norecurse nosync nounwind memory(argmem: write)
define dso_local void @__gen_vec(i64 %len, ptr nocapture writeonly %data) local_unnamed_addr #7 {
pre_entry:
  %cond_01 = icmp sgt i64 %len, 0
  br i1 %cond_01, label %loop_body, label %loop_exit

loop_body:                                        ; preds = %pre_entry, %loop_body
  %x.fr.i3 = phi i64 [ %spec.select.i, %loop_body ], [ 85817256, %pre_entry ]
  %i_12 = phi i64 [ %i_2, %loop_body ], [ 0, %pre_entry ]
  %x_2.i4.10.i = sdiv i64 %x.fr.i3, 2048
  %x_2.i4.12.i = sdiv i64 %x.fr.i3, 8192
  %x_2.i14.13.i = sdiv i64 %x.fr.i3, 16384
  %x_2.i24.15.i = sdiv i64 %x.fr.i3, 65536
  %halfx_0.i17.i = sdiv i64 %x.fr.i3, 32768
  %twohalfx_0.i18.i = shl nsw i64 %halfx_0.i17.i, 1
  %iszero_0.i19.i = icmp ne i64 %twohalfx_0.i18.i, %x_2.i14.13.i
  %twohalfx_0.i8.i = shl nsw i64 %x_2.i14.13.i, 1
  %iszero_0.i9.i = icmp ne i64 %twohalfx_0.i8.i, %x_2.i4.12.i
  %halfx_0.i.i = sdiv i64 %x.fr.i3, 4096
  %twohalfx_0.i.i = shl nsw i64 %halfx_0.i.i, 1
  %iszero_0.i.i = icmp ne i64 %twohalfx_0.i.i, %x_2.i4.10.i
  %halfx_0.i27.i = sdiv i64 %x.fr.i3, 131072
  %twohalfx_0.i28.i = shl nsw i64 %halfx_0.i27.i, 1
  %iszero_0.i29.i = icmp ne i64 %twohalfx_0.i28.i, %x_2.i24.15.i
  %res_0.i.i = xor i1 %iszero_0.i.i, %iszero_0.i9.i
  %res_0.i31.i = xor i1 %iszero_0.i19.i, %res_0.i.i
  %res_0.i32.i = xor i1 %iszero_0.i29.i, %res_0.i31.i
  %s_1.i = shl i64 %x.fr.i3, 1
  %res_0.i32.i.fr = freeze i1 %res_0.i32.i
  %s_2.i = zext i1 %res_0.i32.i.fr to i64
  %spec.select.i = or disjoint i64 %s_1.i, %s_2.i
  %0 = srem i64 %spec.select.i, 10
  %p_0 = getelementptr inbounds i64, ptr %data, i64 %i_12
  store i64 %0, ptr %p_0, align 8
  %i_2 = add nuw nsw i64 %i_12, 1
  %exitcond.not = icmp eq i64 %i_2, %len
  br i1 %exitcond.not, label %loop_exit, label %loop_body

loop_exit:                                        ; preds = %loop_body, %pre_entry
  ret void
}

; Function Attrs: nofree norecurse nosync nounwind memory(argmem: readwrite)
define dso_local void @__csr_spmv(i64 %rows, i64 %cols, ptr nocapture readonly %csr_rowptr, ptr nocapture readonly %csr_colidx, ptr nocapture readonly %csr_values, ptr nocapture readonly %vec, ptr nocapture %res) local_unnamed_addr #8 {
pre_entry:
  %cond_01 = icmp sgt i64 %rows, 0
  br i1 %cond_01, label %loop_rows_body.preheader, label %loop_rows_exit

loop_rows_body.preheader:                         ; preds = %pre_entry
  %0 = shl nuw i64 %rows, 3
  tail call void @llvm.memset.p0.i64(ptr align 8 %res, i8 0, i64 %0, i1 false)
  br label %loop_rows_body

loop_rows_body:                                   ; preds = %loop_rows_body.preheader, %loop_nnzs_exit
  %rid_17 = phi i64 [ %rid_2, %loop_nnzs_exit ], [ 0, %loop_rows_body.preheader ]
  %p_1 = getelementptr inbounds i64, ptr %csr_rowptr, i64 %rid_17
  %start_0 = load i64, ptr %p_1, align 8
  %p_2 = getelementptr inbounds i64, ptr %p_1, i64 1
  %end_0 = load i64, ptr %p_2, align 8
  %cond_43 = icmp slt i64 %start_0, %end_0
  br i1 %cond_43, label %loop_nnzs_body.lr.ph, label %loop_nnzs_exit

loop_nnzs_body.lr.ph:                             ; preds = %loop_rows_body
  %p_7 = getelementptr inbounds i64, ptr %res, i64 %rid_17
  %p_7.promoted = load i64, ptr %p_7, align 8
  br label %loop_nnzs_body

loop_nnzs_body:                                   ; preds = %loop_nnzs_body.lr.ph, %loop_nnzs_body
  %acc_15 = phi i64 [ %p_7.promoted, %loop_nnzs_body.lr.ph ], [ %acc_1, %loop_nnzs_body ]
  %j_14 = phi i64 [ %start_0, %loop_nnzs_body.lr.ph ], [ %j_2, %loop_nnzs_body ]
  %p_4 = getelementptr inbounds i64, ptr %csr_colidx, i64 %j_14
  %cid_0 = load i64, ptr %p_4, align 8
  %p_5 = getelementptr inbounds i64, ptr %csr_values, i64 %j_14
  %mat_val_0 = load i64, ptr %p_5, align 8
  %p_6 = getelementptr inbounds i64, ptr %vec, i64 %cid_0
  %vec_val_0 = load i64, ptr %p_6, align 8
  %incr_0 = mul i64 %vec_val_0, %mat_val_0
  %acc_1 = add i64 %acc_15, %incr_0
  store i64 %acc_1, ptr %p_7, align 8
  %j_2 = add nsw i64 %j_14, 1
  %exitcond.not = icmp eq i64 %j_2, %end_0
  br i1 %exitcond.not, label %loop_nnzs_exit, label %loop_nnzs_body

loop_nnzs_exit:                                   ; preds = %loop_nnzs_body, %loop_rows_body
  %rid_2 = add nuw nsw i64 %rid_17, 1
  %exitcond8.not = icmp eq i64 %rid_2, %rows
  br i1 %exitcond8.not, label %loop_rows_exit, label %loop_rows_body

loop_rows_exit:                                   ; preds = %loop_nnzs_exit, %pre_entry
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__print_arr(ptr nocapture readonly %arr, i64 %size) local_unnamed_addr #0 {
pre_entry:
  %cond_01 = icmp sgt i64 %size, 0
  br i1 %cond_01, label %loop_body, label %loop_exit

loop_body:                                        ; preds = %pre_entry, %loop_body
  %i_12 = phi i64 [ %i_2, %loop_body ], [ 0, %pre_entry ]
  %p_0 = getelementptr inbounds i64, ptr %arr, i64 %i_12
  %v_0 = load i64, ptr %p_0, align 8
  %0 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %v_0)
  %1 = tail call i32 @putchar(i32 10)
  %i_2 = add nuw nsw i64 %i_12, 1
  %exitcond.not = icmp eq i64 %i_2, %size
  br i1 %exitcond.not, label %loop_exit, label %loop_body

loop_exit:                                        ; preds = %loop_body, %pre_entry
  ret void
}

; Function Attrs: nounwind
define dso_local void @__main() local_unnamed_addr #9 {
b0:
  br label %loop_body

loop_body:                                        ; preds = %b0, %loop_body
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %loop_body ]
  tail call void @__orig_main(i64 %loop_counter_11)
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 500
  br i1 %exitcond.not, label %loop_done, label %loop_body

loop_done:                                        ; preds = %loop_body
  ret void
}

; Function Attrs: nounwind
define dso_local void @__orig_main(i64 %n) local_unnamed_addr #9 {
pre_entry:
  %nnz_0 = mul i64 %n, 5
  %rptr_len_0 = shl i64 %n, 3
  %z0 = add i64 %rptr_len_0, 8
  %z1 = tail call ptr @malloc(i64 %z0)
  %z2 = mul i64 %n, 40
  %z3 = tail call ptr @malloc(i64 %z2)
  %z5 = tail call ptr @malloc(i64 %z2)
  store i64 0, ptr %z1, align 8
  %cond_0.not2.i = icmp slt i64 %n, 1
  br i1 %cond_0.not2.i, label %loop_gen_rptr_exit.i, label %loop_gen_rptr_body.i

loop_gen_rptr_body.i:                             ; preds = %pre_entry, %loop_gen_rptr_body.i
  %i_13.i = phi i64 [ %i_2.i, %loop_gen_rptr_body.i ], [ 1, %pre_entry ]
  %p_0.i = getelementptr inbounds i64, ptr %z1, i64 %i_13.i
  %v_0.i = mul i64 %i_13.i, 5
  store i64 %v_0.i, ptr %p_0.i, align 8
  %i_2.i = add i64 %i_13.i, 1
  %cond_0.not.i = icmp sgt i64 %i_2.i, %n
  br i1 %cond_0.not.i, label %loop_gen_rptr_exit.i, label %loop_gen_rptr_body.i

loop_gen_rptr_exit.i:                             ; preds = %loop_gen_rptr_body.i, %pre_entry
  %colidx_incr_0.i = sdiv i64 %n, 5
  %cond_24.i = icmp sgt i64 %nnz_0, 0
  br i1 %cond_24.i, label %loop_gen_cidx_body.i.preheader, label %__gen_uniform_csr.exit

loop_gen_cidx_body.i.preheader:                   ; preds = %loop_gen_rptr_exit.i
  %min.iters.check = icmp ult i64 %nnz_0, 4
  br i1 %min.iters.check, label %loop_gen_cidx_body.i.preheader87, label %vector.ph

vector.ph:                                        ; preds = %loop_gen_cidx_body.i.preheader
  %n.vec = and i64 %nnz_0, 9223372036854775804
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %0 = or disjoint i64 %index, 1
  %1 = or disjoint i64 %index, 2
  %2 = or disjoint i64 %index, 3
  %3 = udiv i64 %index, 5
  %4 = udiv i64 %0, 5
  %5 = udiv i64 %1, 5
  %6 = udiv i64 %2, 5
  %7 = mul i64 %index, %colidx_incr_0.i
  %8 = mul i64 %0, %colidx_incr_0.i
  %9 = mul i64 %1, %colidx_incr_0.i
  %10 = mul i64 %2, %colidx_incr_0.i
  %11 = add i64 %3, %7
  %12 = add i64 %4, %8
  %13 = add i64 %5, %9
  %14 = add i64 %6, %10
  %15 = freeze i64 %11
  %16 = freeze i64 %12
  %17 = freeze i64 %13
  %18 = freeze i64 %14
  %19 = srem i64 %15, %n
  %20 = srem i64 %16, %n
  %21 = srem i64 %17, %n
  %22 = srem i64 %18, %n
  %23 = getelementptr inbounds i64, ptr %z3, i64 %index
  %24 = getelementptr inbounds i64, ptr %z3, i64 %0
  %25 = getelementptr inbounds i64, ptr %z3, i64 %1
  %26 = getelementptr inbounds i64, ptr %z3, i64 %2
  store i64 %19, ptr %23, align 8
  store i64 %20, ptr %24, align 8
  store i64 %21, ptr %25, align 8
  store i64 %22, ptr %26, align 8
  %index.next = add nuw i64 %index, 4
  %27 = icmp eq i64 %index.next, %n.vec
  br i1 %27, label %middle.block, label %vector.body, !llvm.loop !4

middle.block:                                     ; preds = %vector.body
  %cmp.n = icmp eq i64 %nnz_0, %n.vec
  br i1 %cmp.n, label %loop_gen_vals_body.i.preheader, label %loop_gen_cidx_body.i.preheader87

loop_gen_cidx_body.i.preheader87:                 ; preds = %loop_gen_cidx_body.i.preheader, %middle.block
  %i_45.i.ph = phi i64 [ 0, %loop_gen_cidx_body.i.preheader ], [ %n.vec, %middle.block ]
  br label %loop_gen_cidx_body.i

loop_gen_cidx_body.i:                             ; preds = %loop_gen_cidx_body.i.preheader87, %loop_gen_cidx_body.i
  %i_45.i = phi i64 [ %i_5.i, %loop_gen_cidx_body.i ], [ %i_45.i.ph, %loop_gen_cidx_body.i.preheader87 ]
  %rid_0.i = udiv i64 %i_45.i, 5
  %v_1.i = mul i64 %i_45.i, %colidx_incr_0.i
  %v_2.i = add i64 %rid_0.i, %v_1.i
  %x.fr.i.i = freeze i64 %v_2.i
  %28 = srem i64 %x.fr.i.i, %n
  %p_1.i = getelementptr inbounds i64, ptr %z3, i64 %i_45.i
  store i64 %28, ptr %p_1.i, align 8
  %i_5.i = add nuw nsw i64 %i_45.i, 1
  %exitcond.not.i = icmp eq i64 %i_5.i, %nnz_0
  br i1 %exitcond.not.i, label %loop_gen_vals_body.i.preheader, label %loop_gen_cidx_body.i, !llvm.loop !5

loop_gen_vals_body.i.preheader:                   ; preds = %loop_gen_cidx_body.i, %middle.block
  br label %loop_gen_vals_body.i

loop_gen_vals_body.i:                             ; preds = %loop_gen_vals_body.i.preheader, %loop_gen_vals_body.i
  %x.fr.i18.i = phi i64 [ %spec.select.i.i, %loop_gen_vals_body.i ], [ 72160722, %loop_gen_vals_body.i.preheader ]
  %i_77.i = phi i64 [ %i_8.i, %loop_gen_vals_body.i ], [ 0, %loop_gen_vals_body.i.preheader ]
  %x_2.i4.10.i.i = sdiv i64 %x.fr.i18.i, 2048
  %x_2.i4.12.i.i = sdiv i64 %x.fr.i18.i, 8192
  %x_2.i14.13.i.i = sdiv i64 %x.fr.i18.i, 16384
  %x_2.i24.15.i.i = sdiv i64 %x.fr.i18.i, 65536
  %halfx_0.i17.i.i = sdiv i64 %x.fr.i18.i, 32768
  %twohalfx_0.i18.i.i = shl nsw i64 %halfx_0.i17.i.i, 1
  %iszero_0.i19.i.i = icmp ne i64 %twohalfx_0.i18.i.i, %x_2.i14.13.i.i
  %twohalfx_0.i8.i.i = shl nsw i64 %x_2.i14.13.i.i, 1
  %iszero_0.i9.i.i = icmp ne i64 %twohalfx_0.i8.i.i, %x_2.i4.12.i.i
  %halfx_0.i.i.i = sdiv i64 %x.fr.i18.i, 4096
  %twohalfx_0.i.i.i = shl nsw i64 %halfx_0.i.i.i, 1
  %iszero_0.i.i.i = icmp ne i64 %twohalfx_0.i.i.i, %x_2.i4.10.i.i
  %halfx_0.i27.i.i = sdiv i64 %x.fr.i18.i, 131072
  %twohalfx_0.i28.i.i = shl nsw i64 %halfx_0.i27.i.i, 1
  %iszero_0.i29.i.i = icmp ne i64 %twohalfx_0.i28.i.i, %x_2.i24.15.i.i
  %res_0.i.i.i = xor i1 %iszero_0.i.i.i, %iszero_0.i9.i.i
  %res_0.i31.i.i = xor i1 %iszero_0.i19.i.i, %res_0.i.i.i
  %res_0.i32.i.i = xor i1 %iszero_0.i29.i.i, %res_0.i31.i.i
  %s_1.i.i = shl i64 %x.fr.i18.i, 1
  %res_0.i32.i.fr.i = freeze i1 %res_0.i32.i.i
  %s_2.i.i = zext i1 %res_0.i32.i.fr.i to i64
  %spec.select.i.i = or disjoint i64 %s_1.i.i, %s_2.i.i
  %29 = srem i64 %spec.select.i.i, 10
  %p_2.i = getelementptr inbounds i64, ptr %z5, i64 %i_77.i
  store i64 %29, ptr %p_2.i, align 8
  %i_8.i = add nuw nsw i64 %i_77.i, 1
  %exitcond9.not.i = icmp eq i64 %i_8.i, %nnz_0
  br i1 %exitcond9.not.i, label %__gen_uniform_csr.exit, label %loop_gen_vals_body.i

__gen_uniform_csr.exit:                           ; preds = %loop_gen_vals_body.i, %loop_gen_rptr_exit.i
  %cond_01.i = icmp ult i64 %n, 9223372036854775807
  br i1 %cond_01.i, label %loop_body.i, label %__print_arr.exit

loop_body.i:                                      ; preds = %__gen_uniform_csr.exit, %loop_body.i
  %i_12.i = phi i64 [ %i_2.i3, %loop_body.i ], [ 0, %__gen_uniform_csr.exit ]
  %p_0.i1 = getelementptr inbounds i64, ptr %z1, i64 %i_12.i
  %v_0.i2 = load i64, ptr %p_0.i1, align 8
  %30 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %v_0.i2)
  %31 = tail call i32 @putchar(i32 10)
  %i_2.i3 = add nuw nsw i64 %i_12.i, 1
  %exitcond.not.i4 = icmp eq i64 %i_12.i, %n
  br i1 %exitcond.not.i4, label %__print_arr.exit, label %loop_body.i

__print_arr.exit:                                 ; preds = %loop_body.i, %__gen_uniform_csr.exit
  br i1 %cond_24.i, label %loop_body.i6, label %__print_arr.exit20

loop_body.i6:                                     ; preds = %__print_arr.exit, %loop_body.i6
  %i_12.i7 = phi i64 [ %i_2.i10, %loop_body.i6 ], [ 0, %__print_arr.exit ]
  %p_0.i8 = getelementptr inbounds i64, ptr %z3, i64 %i_12.i7
  %v_0.i9 = load i64, ptr %p_0.i8, align 8
  %32 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %v_0.i9)
  %33 = tail call i32 @putchar(i32 10)
  %i_2.i10 = add nuw nsw i64 %i_12.i7, 1
  %exitcond.not.i11 = icmp eq i64 %i_2.i10, %nnz_0
  br i1 %exitcond.not.i11, label %loop_body.i14, label %loop_body.i6

loop_body.i14:                                    ; preds = %loop_body.i6, %loop_body.i14
  %i_12.i15 = phi i64 [ %i_2.i18, %loop_body.i14 ], [ 0, %loop_body.i6 ]
  %p_0.i16 = getelementptr inbounds i64, ptr %z5, i64 %i_12.i15
  %v_0.i17 = load i64, ptr %p_0.i16, align 8
  %34 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %v_0.i17)
  %35 = tail call i32 @putchar(i32 10)
  %i_2.i18 = add nuw nsw i64 %i_12.i15, 1
  %exitcond.not.i19 = icmp eq i64 %i_2.i18, %nnz_0
  br i1 %exitcond.not.i19, label %__print_arr.exit20, label %loop_body.i14

__print_arr.exit20:                               ; preds = %loop_body.i14, %__print_arr.exit
  %z7 = tail call ptr @malloc(i64 %rptr_len_0)
  %cond_01.i21 = icmp sgt i64 %n, 0
  br i1 %cond_01.i21, label %loop_body.i22, label %__print_arr.exit56.thread

loop_body.i22:                                    ; preds = %__print_arr.exit20, %loop_body.i22
  %x.fr.i3.i = phi i64 [ %spec.select.i.i45, %loop_body.i22 ], [ 85817256, %__print_arr.exit20 ]
  %i_12.i23 = phi i64 [ %i_2.i47, %loop_body.i22 ], [ 0, %__print_arr.exit20 ]
  %x_2.i4.10.i.i24 = sdiv i64 %x.fr.i3.i, 2048
  %x_2.i4.12.i.i25 = sdiv i64 %x.fr.i3.i, 8192
  %x_2.i14.13.i.i26 = sdiv i64 %x.fr.i3.i, 16384
  %x_2.i24.15.i.i27 = sdiv i64 %x.fr.i3.i, 65536
  %halfx_0.i17.i.i28 = sdiv i64 %x.fr.i3.i, 32768
  %twohalfx_0.i18.i.i29 = shl nsw i64 %halfx_0.i17.i.i28, 1
  %iszero_0.i19.i.i30 = icmp ne i64 %twohalfx_0.i18.i.i29, %x_2.i14.13.i.i26
  %twohalfx_0.i8.i.i31 = shl nsw i64 %x_2.i14.13.i.i26, 1
  %iszero_0.i9.i.i32 = icmp ne i64 %twohalfx_0.i8.i.i31, %x_2.i4.12.i.i25
  %halfx_0.i.i.i33 = sdiv i64 %x.fr.i3.i, 4096
  %twohalfx_0.i.i.i34 = shl nsw i64 %halfx_0.i.i.i33, 1
  %iszero_0.i.i.i35 = icmp ne i64 %twohalfx_0.i.i.i34, %x_2.i4.10.i.i24
  %halfx_0.i27.i.i36 = sdiv i64 %x.fr.i3.i, 131072
  %twohalfx_0.i28.i.i37 = shl nsw i64 %halfx_0.i27.i.i36, 1
  %iszero_0.i29.i.i38 = icmp ne i64 %twohalfx_0.i28.i.i37, %x_2.i24.15.i.i27
  %res_0.i.i.i39 = xor i1 %iszero_0.i.i.i35, %iszero_0.i9.i.i32
  %res_0.i31.i.i40 = xor i1 %iszero_0.i19.i.i30, %res_0.i.i.i39
  %res_0.i32.i.i41 = xor i1 %iszero_0.i29.i.i38, %res_0.i31.i.i40
  %s_1.i.i42 = shl i64 %x.fr.i3.i, 1
  %res_0.i32.i.fr.i43 = freeze i1 %res_0.i32.i.i41
  %s_2.i.i44 = zext i1 %res_0.i32.i.fr.i43 to i64
  %spec.select.i.i45 = or disjoint i64 %s_1.i.i42, %s_2.i.i44
  %36 = srem i64 %spec.select.i.i45, 10
  %p_0.i46 = getelementptr inbounds i64, ptr %z7, i64 %i_12.i23
  store i64 %36, ptr %p_0.i46, align 8
  %i_2.i47 = add nuw nsw i64 %i_12.i23, 1
  %exitcond.not.i48 = icmp eq i64 %i_2.i47, %n
  br i1 %exitcond.not.i48, label %loop_body.i50, label %loop_body.i22

loop_body.i50:                                    ; preds = %loop_body.i22, %loop_body.i50
  %i_12.i51 = phi i64 [ %i_2.i54, %loop_body.i50 ], [ 0, %loop_body.i22 ]
  %p_0.i52 = getelementptr inbounds i64, ptr %z7, i64 %i_12.i51
  %v_0.i53 = load i64, ptr %p_0.i52, align 8
  %37 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %v_0.i53)
  %38 = tail call i32 @putchar(i32 10)
  %i_2.i54 = add nuw nsw i64 %i_12.i51, 1
  %exitcond.not.i55 = icmp eq i64 %i_2.i54, %n
  br i1 %exitcond.not.i55, label %__print_arr.exit56, label %loop_body.i50

__print_arr.exit56.thread:                        ; preds = %__print_arr.exit20
  %z969 = tail call ptr @malloc(i64 %rptr_len_0)
  br label %__print_arr.exit68

__print_arr.exit56:                               ; preds = %loop_body.i50
  %calloc = tail call ptr @calloc(i64 1, i64 %rptr_len_0)
  br label %loop_rows_body.i

loop_rows_body.i:                                 ; preds = %loop_nnzs_exit.i, %__print_arr.exit56
  %rid_17.i = phi i64 [ %rid_2.i, %loop_nnzs_exit.i ], [ 0, %__print_arr.exit56 ]
  %p_1.i58 = getelementptr inbounds i64, ptr %z1, i64 %rid_17.i
  %start_0.i = load i64, ptr %p_1.i58, align 8
  %p_2.i59 = getelementptr inbounds i64, ptr %p_1.i58, i64 1
  %end_0.i = load i64, ptr %p_2.i59, align 8
  %cond_43.i = icmp slt i64 %start_0.i, %end_0.i
  br i1 %cond_43.i, label %loop_nnzs_body.lr.ph.i, label %loop_nnzs_exit.i

loop_nnzs_body.lr.ph.i:                           ; preds = %loop_rows_body.i
  %p_7.i = getelementptr inbounds i64, ptr %calloc, i64 %rid_17.i
  %p_7.promoted.i = load i64, ptr %p_7.i, align 8
  %39 = sub i64 %end_0.i, %start_0.i
  %min.iters.check75 = icmp ult i64 %39, 2
  br i1 %min.iters.check75, label %loop_nnzs_body.i.preheader, label %vector.ph76

vector.ph76:                                      ; preds = %loop_nnzs_body.lr.ph.i
  %n.vec78 = and i64 %39, -2
  %ind.end = add i64 %start_0.i, %n.vec78
  br label %vector.body81

vector.body81:                                    ; preds = %vector.body81, %vector.ph76
  %index82 = phi i64 [ 0, %vector.ph76 ], [ %index.next84, %vector.body81 ]
  %vec.phi = phi i64 [ %p_7.promoted.i, %vector.ph76 ], [ %55, %vector.body81 ]
  %vec.phi83 = phi i64 [ 0, %vector.ph76 ], [ %56, %vector.body81 ]
  %offset.idx = add i64 %start_0.i, %index82
  %40 = add i64 %offset.idx, 1
  %41 = getelementptr inbounds i64, ptr %z3, i64 %offset.idx
  %42 = getelementptr inbounds i64, ptr %z3, i64 %40
  %43 = load i64, ptr %41, align 8
  %44 = load i64, ptr %42, align 8
  %45 = getelementptr inbounds i64, ptr %z5, i64 %offset.idx
  %46 = getelementptr inbounds i64, ptr %z5, i64 %40
  %47 = load i64, ptr %45, align 8
  %48 = load i64, ptr %46, align 8
  %49 = getelementptr inbounds i64, ptr %z7, i64 %43
  %50 = getelementptr inbounds i64, ptr %z7, i64 %44
  %51 = load i64, ptr %49, align 8
  %52 = load i64, ptr %50, align 8
  %53 = mul i64 %51, %47
  %54 = mul i64 %52, %48
  %55 = add i64 %53, %vec.phi
  %56 = add i64 %54, %vec.phi83
  %index.next84 = add nuw i64 %index82, 2
  %57 = icmp eq i64 %index.next84, %n.vec78
  br i1 %57, label %middle.block73, label %vector.body81, !llvm.loop !6

middle.block73:                                   ; preds = %vector.body81
  %bin.rdx = add i64 %56, %55
  %cmp.n80 = icmp eq i64 %39, %n.vec78
  br i1 %cmp.n80, label %loop_nnzs_exit.i.loopexit, label %loop_nnzs_body.i.preheader

loop_nnzs_body.i.preheader:                       ; preds = %loop_nnzs_body.lr.ph.i, %middle.block73
  %acc_15.i.ph = phi i64 [ %p_7.promoted.i, %loop_nnzs_body.lr.ph.i ], [ %bin.rdx, %middle.block73 ]
  %j_14.i.ph = phi i64 [ %start_0.i, %loop_nnzs_body.lr.ph.i ], [ %ind.end, %middle.block73 ]
  br label %loop_nnzs_body.i

loop_nnzs_body.i:                                 ; preds = %loop_nnzs_body.i.preheader, %loop_nnzs_body.i
  %acc_15.i = phi i64 [ %acc_1.i, %loop_nnzs_body.i ], [ %acc_15.i.ph, %loop_nnzs_body.i.preheader ]
  %j_14.i = phi i64 [ %j_2.i, %loop_nnzs_body.i ], [ %j_14.i.ph, %loop_nnzs_body.i.preheader ]
  %p_4.i = getelementptr inbounds i64, ptr %z3, i64 %j_14.i
  %cid_0.i = load i64, ptr %p_4.i, align 8
  %p_5.i = getelementptr inbounds i64, ptr %z5, i64 %j_14.i
  %mat_val_0.i = load i64, ptr %p_5.i, align 8
  %p_6.i = getelementptr inbounds i64, ptr %z7, i64 %cid_0.i
  %vec_val_0.i = load i64, ptr %p_6.i, align 8
  %incr_0.i = mul i64 %vec_val_0.i, %mat_val_0.i
  %acc_1.i = add i64 %incr_0.i, %acc_15.i
  %j_2.i = add nsw i64 %j_14.i, 1
  %exitcond.not.i60 = icmp eq i64 %j_2.i, %end_0.i
  br i1 %exitcond.not.i60, label %loop_nnzs_exit.i.loopexit, label %loop_nnzs_body.i, !llvm.loop !7

loop_nnzs_exit.i.loopexit:                        ; preds = %loop_nnzs_body.i, %middle.block73
  %acc_1.i.lcssa = phi i64 [ %bin.rdx, %middle.block73 ], [ %acc_1.i, %loop_nnzs_body.i ]
  store i64 %acc_1.i.lcssa, ptr %p_7.i, align 8
  br label %loop_nnzs_exit.i

loop_nnzs_exit.i:                                 ; preds = %loop_nnzs_exit.i.loopexit, %loop_rows_body.i
  %rid_2.i = add nuw nsw i64 %rid_17.i, 1
  %exitcond8.not.i = icmp eq i64 %rid_2.i, %n
  br i1 %exitcond8.not.i, label %loop_body.i62, label %loop_rows_body.i

loop_body.i62:                                    ; preds = %loop_nnzs_exit.i, %loop_body.i62
  %i_12.i63 = phi i64 [ %i_2.i66, %loop_body.i62 ], [ 0, %loop_nnzs_exit.i ]
  %p_0.i64 = getelementptr inbounds i64, ptr %calloc, i64 %i_12.i63
  %v_0.i65 = load i64, ptr %p_0.i64, align 8
  %58 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %v_0.i65)
  %59 = tail call i32 @putchar(i32 10)
  %i_2.i66 = add nuw nsw i64 %i_12.i63, 1
  %exitcond.not.i67 = icmp eq i64 %i_2.i66, %n
  br i1 %exitcond.not.i67, label %__print_arr.exit68, label %loop_body.i62

__print_arr.exit68:                               ; preds = %loop_body.i62, %__print_arr.exit56.thread
  %z97072 = phi ptr [ %z969, %__print_arr.exit56.thread ], [ %calloc, %loop_body.i62 ]
  tail call void @free(ptr %z1)
  tail call void @free(ptr %z3)
  tail call void @free(ptr %z5)
  tail call void @free(ptr %z7)
  tail call void @free(ptr %z97072)
  ret void
}

define dso_local noundef i32 @main(i32 %argc, ptr nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %loop_body.i, label %2

2:                                                ; preds = %0
  %3 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.4, i32 0, i32 %1)
  tail call void @exit(i32 2)
  unreachable

loop_body.i:                                      ; preds = %0, %loop_body.i
  %loop_counter_11.i = phi i64 [ %loop_counter_2.i, %loop_body.i ], [ 10, %0 ]
  tail call void @__orig_main(i64 %loop_counter_11.i)
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 500
  br i1 %exitcond.not.i, label %__main.exit, label %loop_body.i

__main.exit:                                      ; preds = %loop_body.i
  ret i32 0
}

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: write)
declare void @llvm.memset.p0.i64(ptr nocapture writeonly, i8, i64, i1 immarg) #10

; Function Attrs: nofree nounwind willreturn allockind("alloc,zeroed") allocsize(0,1) memory(inaccessiblemem: readwrite)
declare noalias noundef ptr @calloc(i64 noundef, i64 noundef) local_unnamed_addr #11

attributes #0 = { nofree nounwind }
attributes #1 = { mustprogress nofree nounwind willreturn allockind("alloc,uninitialized") allocsize(0) memory(inaccessiblemem: readwrite) "alloc-family"="malloc" }
attributes #2 = { mustprogress nounwind willreturn allockind("free") memory(argmem: readwrite, inaccessiblemem: readwrite) "alloc-family"="malloc" }
attributes #3 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read) }
attributes #4 = { mustprogress nofree norecurse nosync nounwind willreturn memory(none) }
attributes #5 = { nofree norecurse nosync nounwind memory(none) }
attributes #6 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: readwrite) }
attributes #7 = { nofree norecurse nosync nounwind memory(argmem: write) }
attributes #8 = { nofree norecurse nosync nounwind memory(argmem: readwrite) }
attributes #9 = { nounwind }
attributes #10 = { nocallback nofree nounwind willreturn memory(argmem: write) }
attributes #11 = { nofree nounwind willreturn allockind("alloc,zeroed") allocsize(0,1) memory(inaccessiblemem: readwrite) "alloc-family"="malloc" }

!0 = distinct !{!0, !1, !2}
!1 = !{!"llvm.loop.isvectorized", i32 1}
!2 = !{!"llvm.loop.unroll.runtime.disable"}
!3 = distinct !{!3, !1}
!4 = distinct !{!4, !1, !2}
!5 = distinct !{!5, !1}
!6 = distinct !{!6, !1, !2}
!7 = distinct !{!7, !1}
