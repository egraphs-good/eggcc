; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmp4PK7It/compile.ll'
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
declare dso_local noundef i32 @printf(i8* nocapture noundef readonly, ...) local_unnamed_addr #0

declare dso_local void @exit(i32) local_unnamed_addr

; Function Attrs: inaccessiblememonly mustprogress nofree nounwind willreturn allocsize(0)
declare dso_local noalias noundef i8* @malloc(i64 noundef) local_unnamed_addr #1

; Function Attrs: inaccessiblemem_or_argmemonly mustprogress nounwind willreturn
declare dso_local void @free(i8* nocapture noundef) local_unnamed_addr #2

; Function Attrs: argmemonly mustprogress nofree norecurse nosync nounwind readonly willreturn
define dso_local i32 @btoi(i8* nocapture readonly %0) local_unnamed_addr #3 {
  %2 = load i8, i8* %0, align 1
  %3 = icmp eq i8 %2, 116
  %4 = zext i1 %3 to i32
  ret i32 %4
}

; Function Attrs: nofree nounwind
define dso_local void @print_bool(i1 %0) local_unnamed_addr #0 {
  %. = select i1 %0, i8* getelementptr inbounds ([5 x i8], [5 x i8]* @.str, i64 0, i64 0), i8* getelementptr inbounds ([6 x i8], [6 x i8]* @.str.1, i64 0, i64 0)
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) %.)
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
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %0)
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @print_ptr(i8* nocapture readnone %0) local_unnamed_addr #0 {
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([9 x i8], [9 x i8]* @.str.3, i64 0, i64 0))
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind readnone willreturn
define dso_local i1 @__xor(i1 %x, i1 %y) local_unnamed_addr #4 {
pre_entry:
  %res_0 = xor i1 %x, %y
  ret i1 %res_0
}

; Function Attrs: nofree norecurse nosync nounwind readnone
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

; Function Attrs: argmemonly nofree norecurse nosync nounwind
define dso_local void @__rand(i64* nocapture %state) local_unnamed_addr #6 {
pre_entry:
  %s_0 = load i64, i64* %state, align 8
  %x_2.i3.10 = sdiv i64 %s_0, 2048
  %x_2.i3.12 = sdiv i64 %s_0, 8192
  %x_2.i13.13 = sdiv i64 %s_0, 16384
  %x_2.i23.15 = sdiv i64 %s_0, 65536
  %halfx_0.i17 = sdiv i64 %s_0, 32768
  %twohalfx_0.i18 = shl nsw i64 %halfx_0.i17, 1
  %iszero_0.i19 = icmp ne i64 %twohalfx_0.i18, %x_2.i13.13
  %twohalfx_0.i8 = shl nsw i64 %x_2.i13.13, 1
  %iszero_0.i9 = icmp ne i64 %twohalfx_0.i8, %x_2.i3.12
  %halfx_0.i = sdiv i64 %s_0, 4096
  %twohalfx_0.i = shl nsw i64 %halfx_0.i, 1
  %iszero_0.i = icmp ne i64 %twohalfx_0.i, %x_2.i3.10
  %halfx_0.i27 = sdiv i64 %s_0, 131072
  %twohalfx_0.i28 = shl nsw i64 %halfx_0.i27, 1
  %iszero_0.i29 = icmp ne i64 %twohalfx_0.i28, %x_2.i23.15
  %res_0.i = xor i1 %iszero_0.i, %iszero_0.i9
  %res_0.i31 = xor i1 %res_0.i, %iszero_0.i19
  %res_0.i32 = xor i1 %res_0.i31, %iszero_0.i29
  %s_1 = shl i64 %s_0, 1
  %s_2 = zext i1 %res_0.i32 to i64
  %spec.select = or i64 %s_1, %s_2
  store i64 %spec.select, i64* %state, align 8
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind readnone willreturn
define dso_local i64 @__mod(i64 %x, i64 %m) local_unnamed_addr #4 {
pre_entry:
  %0 = srem i64 %x, %m
  ret i64 %0
}

; Function Attrs: nounwind
define dso_local void @__gen_uniform_csr(i64 %rows, i64 %cols, i64 %degree, i64* nocapture writeonly %csr_rowptr, i64* nocapture writeonly %csr_colidx, i64* nocapture writeonly %csr_values) local_unnamed_addr #7 {
pre_entry:
  %nnz_0 = mul i64 %degree, %rows
  store i64 0, i64* %csr_rowptr, align 8
  %cond_0.not1 = icmp slt i64 %rows, 1
  br i1 %cond_0.not1, label %loop_gen_rptr_exit, label %loop_gen_rptr_body

loop_gen_rptr_body:                               ; preds = %pre_entry, %loop_gen_rptr_body
  %i_12 = phi i64 [ %i_2, %loop_gen_rptr_body ], [ 1, %pre_entry ]
  %p_0 = getelementptr inbounds i64, i64* %csr_rowptr, i64 %i_12
  %v_0 = mul i64 %i_12, %degree
  store i64 %v_0, i64* %p_0, align 8
  %i_2 = add i64 %i_12, 1
  %cond_0.not = icmp sgt i64 %i_2, %rows
  br i1 %cond_0.not, label %loop_gen_rptr_exit, label %loop_gen_rptr_body

loop_gen_rptr_exit:                               ; preds = %loop_gen_rptr_body, %pre_entry
  %colidx_incr_0 = sdiv i64 %cols, %degree
  %cond_23 = icmp sgt i64 %nnz_0, 0
  br i1 %cond_23, label %loop_gen_cidx_body.preheader, label %loop_gen_vals_exit

loop_gen_cidx_body.preheader:                     ; preds = %loop_gen_rptr_exit
  %min.iters.check = icmp ult i64 %nnz_0, 4
  br i1 %min.iters.check, label %loop_gen_cidx_body.preheader12, label %vector.ph

vector.ph:                                        ; preds = %loop_gen_cidx_body.preheader
  %n.vec = and i64 %nnz_0, -4
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %induction9 = or i64 %index, 1
  %induction10 = or i64 %index, 2
  %induction11 = or i64 %index, 3
  %0 = sdiv i64 %index, %degree
  %1 = sdiv i64 %induction9, %degree
  %2 = sdiv i64 %induction10, %degree
  %3 = sdiv i64 %induction11, %degree
  %4 = mul i64 %index, %colidx_incr_0
  %5 = mul i64 %induction9, %colidx_incr_0
  %6 = mul i64 %induction10, %colidx_incr_0
  %7 = mul i64 %induction11, %colidx_incr_0
  %8 = add i64 %0, %4
  %9 = add i64 %1, %5
  %10 = add i64 %2, %6
  %11 = add i64 %3, %7
  %12 = srem i64 %8, %cols
  %13 = srem i64 %9, %cols
  %14 = srem i64 %10, %cols
  %15 = srem i64 %11, %cols
  %16 = getelementptr inbounds i64, i64* %csr_colidx, i64 %index
  %17 = getelementptr inbounds i64, i64* %csr_colidx, i64 %induction9
  %18 = getelementptr inbounds i64, i64* %csr_colidx, i64 %induction10
  %19 = getelementptr inbounds i64, i64* %csr_colidx, i64 %induction11
  store i64 %12, i64* %16, align 8
  store i64 %13, i64* %17, align 8
  store i64 %14, i64* %18, align 8
  store i64 %15, i64* %19, align 8
  %index.next = add nuw i64 %index, 4
  %20 = icmp eq i64 %index.next, %n.vec
  br i1 %20, label %middle.block, label %vector.body, !llvm.loop !0

middle.block:                                     ; preds = %vector.body
  %cmp.n = icmp eq i64 %nnz_0, %n.vec
  br i1 %cmp.n, label %loop_gen_vals_body.preheader, label %loop_gen_cidx_body.preheader12

loop_gen_cidx_body.preheader12:                   ; preds = %loop_gen_cidx_body.preheader, %middle.block
  %i_44.ph = phi i64 [ 0, %loop_gen_cidx_body.preheader ], [ %n.vec, %middle.block ]
  br label %loop_gen_cidx_body

loop_gen_cidx_body:                               ; preds = %loop_gen_cidx_body.preheader12, %loop_gen_cidx_body
  %i_44 = phi i64 [ %i_5, %loop_gen_cidx_body ], [ %i_44.ph, %loop_gen_cidx_body.preheader12 ]
  %rid_0 = sdiv i64 %i_44, %degree
  %v_1 = mul i64 %i_44, %colidx_incr_0
  %v_2 = add i64 %rid_0, %v_1
  %21 = srem i64 %v_2, %cols
  %p_1 = getelementptr inbounds i64, i64* %csr_colidx, i64 %i_44
  store i64 %21, i64* %p_1, align 8
  %i_5 = add nuw nsw i64 %i_44, 1
  %exitcond.not = icmp eq i64 %i_5, %nnz_0
  br i1 %exitcond.not, label %loop_gen_vals_body.preheader, label %loop_gen_cidx_body, !llvm.loop !2

loop_gen_vals_body.preheader:                     ; preds = %loop_gen_cidx_body, %middle.block
  br label %loop_gen_vals_body

loop_gen_vals_body:                               ; preds = %loop_gen_vals_body.preheader, %loop_gen_vals_body
  %spec.select.i7 = phi i64 [ %spec.select.i, %loop_gen_vals_body ], [ 72160722, %loop_gen_vals_body.preheader ]
  %i_76 = phi i64 [ %i_8, %loop_gen_vals_body ], [ 0, %loop_gen_vals_body.preheader ]
  %x_2.i3.10.i = sdiv i64 %spec.select.i7, 2048
  %x_2.i3.12.i = sdiv i64 %spec.select.i7, 8192
  %x_2.i13.13.i = sdiv i64 %spec.select.i7, 16384
  %x_2.i23.15.i = sdiv i64 %spec.select.i7, 65536
  %halfx_0.i17.i = sdiv i64 %spec.select.i7, 32768
  %twohalfx_0.i18.i = shl nsw i64 %halfx_0.i17.i, 1
  %iszero_0.i19.i = icmp ne i64 %twohalfx_0.i18.i, %x_2.i13.13.i
  %twohalfx_0.i8.i = shl nsw i64 %x_2.i13.13.i, 1
  %iszero_0.i9.i = icmp ne i64 %twohalfx_0.i8.i, %x_2.i3.12.i
  %halfx_0.i.i = sdiv i64 %spec.select.i7, 4096
  %twohalfx_0.i.i = shl nsw i64 %halfx_0.i.i, 1
  %iszero_0.i.i = icmp ne i64 %twohalfx_0.i.i, %x_2.i3.10.i
  %halfx_0.i27.i = sdiv i64 %spec.select.i7, 131072
  %twohalfx_0.i28.i = shl nsw i64 %halfx_0.i27.i, 1
  %iszero_0.i29.i = icmp ne i64 %twohalfx_0.i28.i, %x_2.i23.15.i
  %res_0.i.i = xor i1 %iszero_0.i.i, %iszero_0.i9.i
  %res_0.i31.i = xor i1 %iszero_0.i19.i, %res_0.i.i
  %res_0.i32.i = xor i1 %iszero_0.i29.i, %res_0.i31.i
  %s_1.i = shl i64 %spec.select.i7, 1
  %s_2.i = zext i1 %res_0.i32.i to i64
  %spec.select.i = or i64 %s_1.i, %s_2.i
  %22 = srem i64 %spec.select.i, 10
  %p_2 = getelementptr inbounds i64, i64* %csr_values, i64 %i_76
  store i64 %22, i64* %p_2, align 8
  %i_8 = add nuw nsw i64 %i_76, 1
  %exitcond8.not = icmp eq i64 %i_8, %nnz_0
  br i1 %exitcond8.not, label %loop_gen_vals_exit, label %loop_gen_vals_body

loop_gen_vals_exit:                               ; preds = %loop_gen_vals_body, %loop_gen_rptr_exit
  ret void
}

; Function Attrs: nounwind
define dso_local void @__gen_vec(i64 %len, i64* nocapture writeonly %data) local_unnamed_addr #7 {
pre_entry:
  %cond_01 = icmp sgt i64 %len, 0
  br i1 %cond_01, label %loop_body, label %loop_exit

loop_body:                                        ; preds = %pre_entry, %loop_body
  %spec.select.i3 = phi i64 [ %spec.select.i, %loop_body ], [ 85817256, %pre_entry ]
  %i_12 = phi i64 [ %i_2, %loop_body ], [ 0, %pre_entry ]
  %x_2.i3.10.i = sdiv i64 %spec.select.i3, 2048
  %x_2.i3.12.i = sdiv i64 %spec.select.i3, 8192
  %x_2.i13.13.i = sdiv i64 %spec.select.i3, 16384
  %x_2.i23.15.i = sdiv i64 %spec.select.i3, 65536
  %halfx_0.i17.i = sdiv i64 %spec.select.i3, 32768
  %twohalfx_0.i18.i = shl nsw i64 %halfx_0.i17.i, 1
  %iszero_0.i19.i = icmp ne i64 %twohalfx_0.i18.i, %x_2.i13.13.i
  %twohalfx_0.i8.i = shl nsw i64 %x_2.i13.13.i, 1
  %iszero_0.i9.i = icmp ne i64 %twohalfx_0.i8.i, %x_2.i3.12.i
  %halfx_0.i.i = sdiv i64 %spec.select.i3, 4096
  %twohalfx_0.i.i = shl nsw i64 %halfx_0.i.i, 1
  %iszero_0.i.i = icmp ne i64 %twohalfx_0.i.i, %x_2.i3.10.i
  %halfx_0.i27.i = sdiv i64 %spec.select.i3, 131072
  %twohalfx_0.i28.i = shl nsw i64 %halfx_0.i27.i, 1
  %iszero_0.i29.i = icmp ne i64 %twohalfx_0.i28.i, %x_2.i23.15.i
  %res_0.i.i = xor i1 %iszero_0.i.i, %iszero_0.i9.i
  %res_0.i31.i = xor i1 %iszero_0.i19.i, %res_0.i.i
  %res_0.i32.i = xor i1 %iszero_0.i29.i, %res_0.i31.i
  %s_1.i = shl i64 %spec.select.i3, 1
  %s_2.i = zext i1 %res_0.i32.i to i64
  %spec.select.i = or i64 %s_1.i, %s_2.i
  %0 = srem i64 %spec.select.i, 10
  %p_0 = getelementptr inbounds i64, i64* %data, i64 %i_12
  store i64 %0, i64* %p_0, align 8
  %i_2 = add nuw nsw i64 %i_12, 1
  %exitcond.not = icmp eq i64 %i_2, %len
  br i1 %exitcond.not, label %loop_exit, label %loop_body

loop_exit:                                        ; preds = %loop_body, %pre_entry
  ret void
}

; Function Attrs: argmemonly nofree norecurse nosync nounwind
define dso_local void @__csr_spmv(i64 %rows, i64 %cols, i64* nocapture readonly %csr_rowptr, i64* nocapture readonly %csr_colidx, i64* nocapture readonly %csr_values, i64* nocapture readonly %vec, i64* nocapture %res) local_unnamed_addr #6 {
pre_entry:
  %cond_01 = icmp sgt i64 %rows, 0
  br i1 %cond_01, label %loop_rows_body.preheader, label %loop_rows_exit

loop_rows_body.preheader:                         ; preds = %pre_entry
  %res8 = bitcast i64* %res to i8*
  %0 = shl nuw i64 %rows, 3
  call void @llvm.memset.p0i8.i64(i8* align 8 %res8, i8 0, i64 %0, i1 false)
  br label %loop_rows_body

loop_rows_body:                                   ; preds = %loop_rows_body.preheader, %loop_nnzs_exit
  %rid_16 = phi i64 [ %rid_2, %loop_nnzs_exit ], [ 0, %loop_rows_body.preheader ]
  %p_1 = getelementptr inbounds i64, i64* %csr_rowptr, i64 %rid_16
  %start_0 = load i64, i64* %p_1, align 8
  %p_2 = getelementptr inbounds i64, i64* %p_1, i64 1
  %end_0 = load i64, i64* %p_2, align 8
  %cond_43 = icmp slt i64 %start_0, %end_0
  br i1 %cond_43, label %loop_nnzs_body.lr.ph, label %loop_nnzs_exit

loop_nnzs_body.lr.ph:                             ; preds = %loop_rows_body
  %p_7 = getelementptr inbounds i64, i64* %res, i64 %rid_16
  %acc_0.pre = load i64, i64* %p_7, align 8
  br label %loop_nnzs_body

loop_nnzs_body:                                   ; preds = %loop_nnzs_body.lr.ph, %loop_nnzs_body
  %acc_0 = phi i64 [ %acc_0.pre, %loop_nnzs_body.lr.ph ], [ %acc_1, %loop_nnzs_body ]
  %j_14 = phi i64 [ %start_0, %loop_nnzs_body.lr.ph ], [ %j_2, %loop_nnzs_body ]
  %p_4 = getelementptr inbounds i64, i64* %csr_colidx, i64 %j_14
  %cid_0 = load i64, i64* %p_4, align 8
  %p_5 = getelementptr inbounds i64, i64* %csr_values, i64 %j_14
  %mat_val_0 = load i64, i64* %p_5, align 8
  %p_6 = getelementptr inbounds i64, i64* %vec, i64 %cid_0
  %vec_val_0 = load i64, i64* %p_6, align 8
  %incr_0 = mul i64 %vec_val_0, %mat_val_0
  %acc_1 = add i64 %acc_0, %incr_0
  store i64 %acc_1, i64* %p_7, align 8
  %j_2 = add nsw i64 %j_14, 1
  %exitcond.not = icmp eq i64 %j_2, %end_0
  br i1 %exitcond.not, label %loop_nnzs_exit, label %loop_nnzs_body

loop_nnzs_exit:                                   ; preds = %loop_nnzs_body, %loop_rows_body
  %rid_2 = add nuw nsw i64 %rid_16, 1
  %exitcond9.not = icmp eq i64 %rid_2, %rows
  br i1 %exitcond9.not, label %loop_rows_exit, label %loop_rows_body

loop_rows_exit:                                   ; preds = %loop_nnzs_exit, %pre_entry
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__print_arr(i64* nocapture readonly %arr, i64 %size) local_unnamed_addr #0 {
pre_entry:
  %cond_01 = icmp sgt i64 %size, 0
  br i1 %cond_01, label %loop_body, label %loop_exit

loop_body:                                        ; preds = %pre_entry, %loop_body
  %i_12 = phi i64 [ %i_2, %loop_body ], [ 0, %pre_entry ]
  %p_0 = getelementptr inbounds i64, i64* %arr, i64 %i_12
  %v_0 = load i64, i64* %p_0, align 8
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %v_0) #7
  %1 = tail call i32 @putchar(i32 10) #7
  %i_2 = add nuw nsw i64 %i_12, 1
  %exitcond.not = icmp eq i64 %i_2, %size
  br i1 %exitcond.not, label %loop_exit, label %loop_body

loop_exit:                                        ; preds = %loop_body, %pre_entry
  ret void
}

; Function Attrs: nounwind
define dso_local void @__main() local_unnamed_addr #7 {
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
define dso_local void @__orig_main(i64 %n) local_unnamed_addr #7 {
pre_entry:
  %nnz_0 = mul i64 %n, 5
  %rptr_len_0 = shl i64 %n, 3
  %z0 = add i64 %rptr_len_0, 8
  %z1 = tail call i8* @malloc(i64 %z0)
  %csr_rowptr_0 = bitcast i8* %z1 to i64*
  %z2 = mul i64 %n, 40
  %z3 = tail call i8* @malloc(i64 %z2)
  %csr_colidx_0 = bitcast i8* %z3 to i64*
  %z5 = tail call i8* @malloc(i64 %z2)
  %csr_values_0 = bitcast i8* %z5 to i64*
  store i64 0, i64* %csr_rowptr_0, align 8
  %cond_0.not1.i = icmp slt i64 %n, 1
  br i1 %cond_0.not1.i, label %loop_gen_rptr_exit.i, label %loop_gen_rptr_body.i

loop_gen_rptr_body.i:                             ; preds = %pre_entry, %loop_gen_rptr_body.i
  %i_12.i = phi i64 [ %i_2.i, %loop_gen_rptr_body.i ], [ 1, %pre_entry ]
  %p_0.i = getelementptr inbounds i64, i64* %csr_rowptr_0, i64 %i_12.i
  %v_0.i = mul i64 %i_12.i, 5
  store i64 %v_0.i, i64* %p_0.i, align 8
  %i_2.i = add i64 %i_12.i, 1
  %cond_0.not.i = icmp sgt i64 %i_2.i, %n
  br i1 %cond_0.not.i, label %loop_gen_rptr_exit.i, label %loop_gen_rptr_body.i

loop_gen_rptr_exit.i:                             ; preds = %loop_gen_rptr_body.i, %pre_entry
  %colidx_incr_0.i = sdiv i64 %n, 5
  %cond_23.i = icmp sgt i64 %nnz_0, 0
  br i1 %cond_23.i, label %loop_gen_cidx_body.i.preheader, label %__gen_uniform_csr.exit

loop_gen_cidx_body.i.preheader:                   ; preds = %loop_gen_rptr_exit.i
  %min.iters.check = icmp ult i64 %nnz_0, 4
  br i1 %min.iters.check, label %loop_gen_cidx_body.i.preheader97, label %vector.ph

vector.ph:                                        ; preds = %loop_gen_cidx_body.i.preheader
  %n.vec = and i64 %nnz_0, -4
  br label %vector.body

vector.body:                                      ; preds = %vector.body, %vector.ph
  %index = phi i64 [ 0, %vector.ph ], [ %index.next, %vector.body ]
  %induction76 = or i64 %index, 1
  %induction77 = or i64 %index, 2
  %induction78 = or i64 %index, 3
  %0 = udiv i64 %index, 5
  %1 = udiv i64 %induction76, 5
  %2 = udiv i64 %induction77, 5
  %3 = udiv i64 %induction78, 5
  %4 = mul i64 %index, %colidx_incr_0.i
  %5 = mul i64 %induction76, %colidx_incr_0.i
  %6 = mul i64 %induction77, %colidx_incr_0.i
  %7 = mul i64 %induction78, %colidx_incr_0.i
  %8 = add i64 %0, %4
  %9 = add i64 %1, %5
  %10 = add i64 %2, %6
  %11 = add i64 %3, %7
  %12 = srem i64 %8, %n
  %13 = srem i64 %9, %n
  %14 = srem i64 %10, %n
  %15 = srem i64 %11, %n
  %16 = getelementptr inbounds i64, i64* %csr_colidx_0, i64 %index
  %17 = getelementptr inbounds i64, i64* %csr_colidx_0, i64 %induction76
  %18 = getelementptr inbounds i64, i64* %csr_colidx_0, i64 %induction77
  %19 = getelementptr inbounds i64, i64* %csr_colidx_0, i64 %induction78
  store i64 %12, i64* %16, align 8
  store i64 %13, i64* %17, align 8
  store i64 %14, i64* %18, align 8
  store i64 %15, i64* %19, align 8
  %index.next = add nuw i64 %index, 4
  %20 = icmp eq i64 %index.next, %n.vec
  br i1 %20, label %middle.block, label %vector.body, !llvm.loop !3

middle.block:                                     ; preds = %vector.body
  %cmp.n = icmp eq i64 %nnz_0, %n.vec
  br i1 %cmp.n, label %loop_gen_vals_body.i.preheader, label %loop_gen_cidx_body.i.preheader97

loop_gen_cidx_body.i.preheader97:                 ; preds = %loop_gen_cidx_body.i.preheader, %middle.block
  %i_44.i.ph = phi i64 [ 0, %loop_gen_cidx_body.i.preheader ], [ %n.vec, %middle.block ]
  br label %loop_gen_cidx_body.i

loop_gen_cidx_body.i:                             ; preds = %loop_gen_cidx_body.i.preheader97, %loop_gen_cidx_body.i
  %i_44.i = phi i64 [ %i_5.i, %loop_gen_cidx_body.i ], [ %i_44.i.ph, %loop_gen_cidx_body.i.preheader97 ]
  %rid_0.i = udiv i64 %i_44.i, 5
  %v_1.i = mul i64 %i_44.i, %colidx_incr_0.i
  %v_2.i = add i64 %rid_0.i, %v_1.i
  %21 = srem i64 %v_2.i, %n
  %p_1.i = getelementptr inbounds i64, i64* %csr_colidx_0, i64 %i_44.i
  store i64 %21, i64* %p_1.i, align 8
  %i_5.i = add nuw nsw i64 %i_44.i, 1
  %exitcond.not.i = icmp eq i64 %i_5.i, %nnz_0
  br i1 %exitcond.not.i, label %loop_gen_vals_body.i.preheader, label %loop_gen_cidx_body.i, !llvm.loop !4

loop_gen_vals_body.i.preheader:                   ; preds = %loop_gen_cidx_body.i, %middle.block
  br label %loop_gen_vals_body.i

loop_gen_vals_body.i:                             ; preds = %loop_gen_vals_body.i.preheader, %loop_gen_vals_body.i
  %spec.select.i7.i = phi i64 [ %spec.select.i.i, %loop_gen_vals_body.i ], [ 72160722, %loop_gen_vals_body.i.preheader ]
  %i_76.i = phi i64 [ %i_8.i, %loop_gen_vals_body.i ], [ 0, %loop_gen_vals_body.i.preheader ]
  %x_2.i3.10.i.i = sdiv i64 %spec.select.i7.i, 2048
  %x_2.i3.12.i.i = sdiv i64 %spec.select.i7.i, 8192
  %x_2.i13.13.i.i = sdiv i64 %spec.select.i7.i, 16384
  %x_2.i23.15.i.i = sdiv i64 %spec.select.i7.i, 65536
  %halfx_0.i17.i.i = sdiv i64 %spec.select.i7.i, 32768
  %twohalfx_0.i18.i.i = shl nsw i64 %halfx_0.i17.i.i, 1
  %iszero_0.i19.i.i = icmp ne i64 %twohalfx_0.i18.i.i, %x_2.i13.13.i.i
  %twohalfx_0.i8.i.i = shl nsw i64 %x_2.i13.13.i.i, 1
  %iszero_0.i9.i.i = icmp ne i64 %twohalfx_0.i8.i.i, %x_2.i3.12.i.i
  %halfx_0.i.i.i = sdiv i64 %spec.select.i7.i, 4096
  %twohalfx_0.i.i.i = shl nsw i64 %halfx_0.i.i.i, 1
  %iszero_0.i.i.i = icmp ne i64 %twohalfx_0.i.i.i, %x_2.i3.10.i.i
  %halfx_0.i27.i.i = sdiv i64 %spec.select.i7.i, 131072
  %twohalfx_0.i28.i.i = shl nsw i64 %halfx_0.i27.i.i, 1
  %iszero_0.i29.i.i = icmp ne i64 %twohalfx_0.i28.i.i, %x_2.i23.15.i.i
  %res_0.i.i.i = xor i1 %iszero_0.i.i.i, %iszero_0.i9.i.i
  %res_0.i31.i.i = xor i1 %iszero_0.i19.i.i, %res_0.i.i.i
  %res_0.i32.i.i = xor i1 %iszero_0.i29.i.i, %res_0.i31.i.i
  %s_1.i.i = shl i64 %spec.select.i7.i, 1
  %s_2.i.i = zext i1 %res_0.i32.i.i to i64
  %spec.select.i.i = or i64 %s_1.i.i, %s_2.i.i
  %22 = srem i64 %spec.select.i.i, 10
  %p_2.i = getelementptr inbounds i64, i64* %csr_values_0, i64 %i_76.i
  store i64 %22, i64* %p_2.i, align 8
  %i_8.i = add nuw nsw i64 %i_76.i, 1
  %exitcond8.not.i = icmp eq i64 %i_8.i, %nnz_0
  br i1 %exitcond8.not.i, label %__gen_uniform_csr.exit, label %loop_gen_vals_body.i

__gen_uniform_csr.exit:                           ; preds = %loop_gen_vals_body.i, %loop_gen_rptr_exit.i
  %cond_01.i = icmp ult i64 %n, 9223372036854775807
  br i1 %cond_01.i, label %loop_body.i, label %__print_arr.exit

loop_body.i:                                      ; preds = %__gen_uniform_csr.exit, %loop_body.i
  %i_12.i1 = phi i64 [ %i_2.i4, %loop_body.i ], [ 0, %__gen_uniform_csr.exit ]
  %p_0.i2 = getelementptr inbounds i64, i64* %csr_rowptr_0, i64 %i_12.i1
  %v_0.i3 = load i64, i64* %p_0.i2, align 8
  %23 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %v_0.i3) #7
  %24 = tail call i32 @putchar(i32 10) #7
  %i_2.i4 = add nuw nsw i64 %i_12.i1, 1
  %exitcond.not.i5 = icmp eq i64 %i_12.i1, %n
  br i1 %exitcond.not.i5, label %__print_arr.exit, label %loop_body.i

__print_arr.exit:                                 ; preds = %loop_body.i, %__gen_uniform_csr.exit
  br i1 %cond_23.i, label %loop_body.i12, label %__print_arr.exit21

loop_body.i12:                                    ; preds = %__print_arr.exit, %loop_body.i12
  %i_12.i7 = phi i64 [ %i_2.i10, %loop_body.i12 ], [ 0, %__print_arr.exit ]
  %p_0.i8 = getelementptr inbounds i64, i64* %csr_colidx_0, i64 %i_12.i7
  %v_0.i9 = load i64, i64* %p_0.i8, align 8
  %25 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %v_0.i9) #7
  %26 = tail call i32 @putchar(i32 10) #7
  %i_2.i10 = add nuw nsw i64 %i_12.i7, 1
  %exitcond.not.i11 = icmp eq i64 %i_2.i10, %nnz_0
  br i1 %exitcond.not.i11, label %loop_body.i20, label %loop_body.i12

loop_body.i20:                                    ; preds = %loop_body.i12, %loop_body.i20
  %i_12.i15 = phi i64 [ %i_2.i18, %loop_body.i20 ], [ 0, %loop_body.i12 ]
  %p_0.i16 = getelementptr inbounds i64, i64* %csr_values_0, i64 %i_12.i15
  %v_0.i17 = load i64, i64* %p_0.i16, align 8
  %27 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %v_0.i17) #7
  %28 = tail call i32 @putchar(i32 10) #7
  %i_2.i18 = add nuw nsw i64 %i_12.i15, 1
  %exitcond.not.i19 = icmp eq i64 %i_2.i18, %nnz_0
  br i1 %exitcond.not.i19, label %__print_arr.exit21, label %loop_body.i20

__print_arr.exit21:                               ; preds = %loop_body.i20, %__print_arr.exit
  %z7 = tail call i8* @malloc(i64 %rptr_len_0)
  %vec_0 = bitcast i8* %z7 to i64*
  %cond_01.i22 = icmp sgt i64 %n, 0
  br i1 %cond_01.i22, label %loop_body.i48, label %__print_arr.exit56.thread

loop_body.i48:                                    ; preds = %__print_arr.exit21, %loop_body.i48
  %spec.select.i3.i = phi i64 [ %spec.select.i.i44, %loop_body.i48 ], [ 85817256, %__print_arr.exit21 ]
  %i_12.i23 = phi i64 [ %i_2.i46, %loop_body.i48 ], [ 0, %__print_arr.exit21 ]
  %x_2.i3.10.i.i24 = sdiv i64 %spec.select.i3.i, 2048
  %x_2.i3.12.i.i25 = sdiv i64 %spec.select.i3.i, 8192
  %x_2.i13.13.i.i26 = sdiv i64 %spec.select.i3.i, 16384
  %x_2.i23.15.i.i27 = sdiv i64 %spec.select.i3.i, 65536
  %halfx_0.i17.i.i28 = sdiv i64 %spec.select.i3.i, 32768
  %twohalfx_0.i18.i.i29 = shl nsw i64 %halfx_0.i17.i.i28, 1
  %iszero_0.i19.i.i30 = icmp ne i64 %twohalfx_0.i18.i.i29, %x_2.i13.13.i.i26
  %twohalfx_0.i8.i.i31 = shl nsw i64 %x_2.i13.13.i.i26, 1
  %iszero_0.i9.i.i32 = icmp ne i64 %twohalfx_0.i8.i.i31, %x_2.i3.12.i.i25
  %halfx_0.i.i.i33 = sdiv i64 %spec.select.i3.i, 4096
  %twohalfx_0.i.i.i34 = shl nsw i64 %halfx_0.i.i.i33, 1
  %iszero_0.i.i.i35 = icmp ne i64 %twohalfx_0.i.i.i34, %x_2.i3.10.i.i24
  %halfx_0.i27.i.i36 = sdiv i64 %spec.select.i3.i, 131072
  %twohalfx_0.i28.i.i37 = shl nsw i64 %halfx_0.i27.i.i36, 1
  %iszero_0.i29.i.i38 = icmp ne i64 %twohalfx_0.i28.i.i37, %x_2.i23.15.i.i27
  %res_0.i.i.i39 = xor i1 %iszero_0.i.i.i35, %iszero_0.i9.i.i32
  %res_0.i31.i.i40 = xor i1 %iszero_0.i19.i.i30, %res_0.i.i.i39
  %res_0.i32.i.i41 = xor i1 %iszero_0.i29.i.i38, %res_0.i31.i.i40
  %s_1.i.i42 = shl i64 %spec.select.i3.i, 1
  %s_2.i.i43 = zext i1 %res_0.i32.i.i41 to i64
  %spec.select.i.i44 = or i64 %s_1.i.i42, %s_2.i.i43
  %29 = srem i64 %spec.select.i.i44, 10
  %p_0.i45 = getelementptr inbounds i64, i64* %vec_0, i64 %i_12.i23
  store i64 %29, i64* %p_0.i45, align 8
  %i_2.i46 = add nuw nsw i64 %i_12.i23, 1
  %exitcond.not.i47 = icmp eq i64 %i_2.i46, %n
  br i1 %exitcond.not.i47, label %loop_body.i55, label %loop_body.i48

loop_body.i55:                                    ; preds = %loop_body.i48, %loop_body.i55
  %i_12.i50 = phi i64 [ %i_2.i53, %loop_body.i55 ], [ 0, %loop_body.i48 ]
  %p_0.i51 = getelementptr inbounds i64, i64* %vec_0, i64 %i_12.i50
  %v_0.i52 = load i64, i64* %p_0.i51, align 8
  %30 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %v_0.i52) #7
  %31 = tail call i32 @putchar(i32 10) #7
  %i_2.i53 = add nuw nsw i64 %i_12.i50, 1
  %exitcond.not.i54 = icmp eq i64 %i_2.i53, %n
  br i1 %exitcond.not.i54, label %loop_rows_body.preheader.i, label %loop_body.i55

__print_arr.exit56.thread:                        ; preds = %__print_arr.exit21
  %z969 = tail call i8* @malloc(i64 %rptr_len_0)
  br label %__print_arr.exit68

loop_rows_body.preheader.i:                       ; preds = %loop_body.i55
  %calloc = call i8* @calloc(i64 1, i64 %rptr_len_0)
  %res_0 = bitcast i8* %calloc to i64*
  br label %loop_rows_body.i

loop_rows_body.i:                                 ; preds = %loop_nnzs_exit.i, %loop_rows_body.preheader.i
  %rid_16.i = phi i64 [ %rid_2.i, %loop_nnzs_exit.i ], [ 0, %loop_rows_body.preheader.i ]
  %p_1.i58 = getelementptr inbounds i64, i64* %csr_rowptr_0, i64 %rid_16.i
  %start_0.i = load i64, i64* %p_1.i58, align 8
  %p_2.i59 = getelementptr inbounds i64, i64* %p_1.i58, i64 1
  %end_0.i = load i64, i64* %p_2.i59, align 8
  %cond_43.i = icmp sgt i64 %end_0.i, %start_0.i
  br i1 %cond_43.i, label %loop_nnzs_body.lr.ph.i, label %loop_nnzs_exit.i

loop_nnzs_body.lr.ph.i:                           ; preds = %loop_rows_body.i
  %p_7.i = getelementptr inbounds i64, i64* %res_0, i64 %rid_16.i
  %acc_0.pre.i = load i64, i64* %p_7.i, align 8
  %32 = sub i64 %end_0.i, %start_0.i
  %min.iters.check81 = icmp ult i64 %32, 2
  br i1 %min.iters.check81, label %loop_nnzs_body.i.preheader, label %vector.ph82

vector.ph82:                                      ; preds = %loop_nnzs_body.lr.ph.i
  %n.vec84 = and i64 %32, -2
  %ind.end = add i64 %start_0.i, %n.vec84
  br label %vector.body87

vector.body87:                                    ; preds = %vector.body87, %vector.ph82
  %index88 = phi i64 [ 0, %vector.ph82 ], [ %index.next92, %vector.body87 ]
  %vec.phi = phi i64 [ %acc_0.pre.i, %vector.ph82 ], [ %47, %vector.body87 ]
  %vec.phi89 = phi i64 [ 0, %vector.ph82 ], [ %48, %vector.body87 ]
  %offset.idx = add i64 %start_0.i, %index88
  %induction91 = add i64 %offset.idx, 1
  %33 = getelementptr inbounds i64, i64* %csr_colidx_0, i64 %offset.idx
  %34 = getelementptr inbounds i64, i64* %csr_colidx_0, i64 %induction91
  %35 = load i64, i64* %33, align 8
  %36 = load i64, i64* %34, align 8
  %37 = getelementptr inbounds i64, i64* %csr_values_0, i64 %offset.idx
  %38 = getelementptr inbounds i64, i64* %csr_values_0, i64 %induction91
  %39 = load i64, i64* %37, align 8
  %40 = load i64, i64* %38, align 8
  %41 = getelementptr inbounds i64, i64* %vec_0, i64 %35
  %42 = getelementptr inbounds i64, i64* %vec_0, i64 %36
  %43 = load i64, i64* %41, align 8
  %44 = load i64, i64* %42, align 8
  %45 = mul i64 %43, %39
  %46 = mul i64 %44, %40
  %47 = add i64 %45, %vec.phi
  %48 = add i64 %46, %vec.phi89
  %index.next92 = add nuw i64 %index88, 2
  %49 = icmp eq i64 %index.next92, %n.vec84
  br i1 %49, label %middle.block79, label %vector.body87, !llvm.loop !5

middle.block79:                                   ; preds = %vector.body87
  %bin.rdx = add i64 %48, %47
  %cmp.n86 = icmp eq i64 %32, %n.vec84
  br i1 %cmp.n86, label %loop_nnzs_exit.i.loopexit, label %loop_nnzs_body.i.preheader

loop_nnzs_body.i.preheader:                       ; preds = %loop_nnzs_body.lr.ph.i, %middle.block79
  %acc_0.i.ph = phi i64 [ %acc_0.pre.i, %loop_nnzs_body.lr.ph.i ], [ %bin.rdx, %middle.block79 ]
  %j_14.i.ph = phi i64 [ %start_0.i, %loop_nnzs_body.lr.ph.i ], [ %ind.end, %middle.block79 ]
  br label %loop_nnzs_body.i

loop_nnzs_body.i:                                 ; preds = %loop_nnzs_body.i.preheader, %loop_nnzs_body.i
  %acc_0.i = phi i64 [ %acc_1.i, %loop_nnzs_body.i ], [ %acc_0.i.ph, %loop_nnzs_body.i.preheader ]
  %j_14.i = phi i64 [ %j_2.i, %loop_nnzs_body.i ], [ %j_14.i.ph, %loop_nnzs_body.i.preheader ]
  %p_4.i = getelementptr inbounds i64, i64* %csr_colidx_0, i64 %j_14.i
  %cid_0.i = load i64, i64* %p_4.i, align 8
  %p_5.i = getelementptr inbounds i64, i64* %csr_values_0, i64 %j_14.i
  %mat_val_0.i = load i64, i64* %p_5.i, align 8
  %p_6.i = getelementptr inbounds i64, i64* %vec_0, i64 %cid_0.i
  %vec_val_0.i = load i64, i64* %p_6.i, align 8
  %incr_0.i = mul i64 %vec_val_0.i, %mat_val_0.i
  %acc_1.i = add i64 %incr_0.i, %acc_0.i
  %j_2.i = add nsw i64 %j_14.i, 1
  %exitcond.not.i60 = icmp eq i64 %j_2.i, %end_0.i
  br i1 %exitcond.not.i60, label %loop_nnzs_exit.i.loopexit, label %loop_nnzs_body.i, !llvm.loop !6

loop_nnzs_exit.i.loopexit:                        ; preds = %loop_nnzs_body.i, %middle.block79
  %acc_1.i.lcssa = phi i64 [ %bin.rdx, %middle.block79 ], [ %acc_1.i, %loop_nnzs_body.i ]
  store i64 %acc_1.i.lcssa, i64* %p_7.i, align 8
  br label %loop_nnzs_exit.i

loop_nnzs_exit.i:                                 ; preds = %loop_nnzs_exit.i.loopexit, %loop_rows_body.i
  %rid_2.i = add nuw nsw i64 %rid_16.i, 1
  %exitcond9.not.i = icmp eq i64 %rid_2.i, %n
  br i1 %exitcond9.not.i, label %loop_body.i67, label %loop_rows_body.i

loop_body.i67:                                    ; preds = %loop_nnzs_exit.i, %loop_body.i67
  %i_12.i62 = phi i64 [ %i_2.i65, %loop_body.i67 ], [ 0, %loop_nnzs_exit.i ]
  %p_0.i63 = getelementptr inbounds i64, i64* %res_0, i64 %i_12.i62
  %v_0.i64 = load i64, i64* %p_0.i63, align 8
  %50 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %v_0.i64) #7
  %51 = tail call i32 @putchar(i32 10) #7
  %i_2.i65 = add nuw nsw i64 %i_12.i62, 1
  %exitcond.not.i66 = icmp eq i64 %i_2.i65, %n
  br i1 %exitcond.not.i66, label %__print_arr.exit68, label %loop_body.i67

__print_arr.exit68:                               ; preds = %loop_body.i67, %__print_arr.exit56.thread
  %z97175 = phi i8* [ %z969, %__print_arr.exit56.thread ], [ %calloc, %loop_body.i67 ]
  tail call void @free(i8* %z1)
  tail call void @free(i8* %z3)
  tail call void @free(i8* %z5)
  tail call void @free(i8* %z7)
  tail call void @free(i8* %z97175)
  ret void
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %loop_body.i, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #11
  ret i32 0

loop_body.i:                                      ; preds = %0, %loop_body.i
  %loop_counter_11.i = phi i64 [ %loop_counter_2.i, %loop_body.i ], [ 10, %0 ]
  tail call void @__orig_main(i64 %loop_counter_11.i) #7
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 500
  br i1 %exitcond.not.i, label %__main.exit, label %loop_body.i

__main.exit:                                      ; preds = %loop_body.i
  ret i32 0
}

; Function Attrs: argmemonly nofree nounwind willreturn writeonly
declare void @llvm.memset.p0i8.i64(i8* nocapture writeonly, i8, i64, i1 immarg) #8

; Function Attrs: inaccessiblememonly nofree nounwind willreturn allocsize(0,1)
declare noalias noundef i8* @calloc(i64 noundef, i64 noundef) local_unnamed_addr #9

; Function Attrs: cold minsize noreturn
define internal void @main.cold.1(i32 %0) #10 {
newFuncRoot:
  br label %1

1:                                                ; preds = %newFuncRoot
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([33 x i8], [33 x i8]* @.str.4, i64 0, i64 0), i32 0, i32 %0)
  tail call void @exit(i32 2)
  unreachable
}

attributes #0 = { nofree nounwind }
attributes #1 = { inaccessiblememonly mustprogress nofree nounwind willreturn allocsize(0) }
attributes #2 = { inaccessiblemem_or_argmemonly mustprogress nounwind willreturn }
attributes #3 = { argmemonly mustprogress nofree norecurse nosync nounwind readonly willreturn }
attributes #4 = { mustprogress nofree norecurse nosync nounwind readnone willreturn }
attributes #5 = { nofree norecurse nosync nounwind readnone }
attributes #6 = { argmemonly nofree norecurse nosync nounwind }
attributes #7 = { nounwind }
attributes #8 = { argmemonly nofree nounwind willreturn writeonly }
attributes #9 = { inaccessiblememonly nofree nounwind willreturn allocsize(0,1) }
attributes #10 = { cold minsize noreturn }
attributes #11 = { noinline }

!0 = distinct !{!0, !1}
!1 = !{!"llvm.loop.isvectorized", i32 1}
!2 = distinct !{!2, !1}
!3 = distinct !{!3, !1}
!4 = distinct !{!4, !1}
!5 = distinct !{!5, !1}
!6 = distinct !{!6, !1}
