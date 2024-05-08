; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpeABilJ/compile.ll'
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

; Function Attrs: argmemonly mustprogress nofree norecurse nosync nounwind willreturn
define dso_local i64 @__rand(i64* nocapture %seq, i64 %max) local_unnamed_addr #4 {
pre_entry:
  %x_0 = load i64, i64* %seq, align 8
  %ax_0 = mul i64 %x_0, 25214903917
  %axpc_0 = add i64 %ax_0, 11
  %next_2 = srem i64 %axpc_0, 281474976710656
  store i64 %next_2, i64* %seq, align 8
  %0 = srem i64 %next_2, %max
  ret i64 %0
}

; Function Attrs: nofree nounwind
define dso_local noalias i64* @__randarray(i64 %size, i64* nocapture %rng) local_unnamed_addr #0 {
pre_entry:
  %z0 = shl i64 %size, 3
  %z1 = tail call i8* @malloc(i64 %z0)
  %arr_0 = bitcast i8* %z1 to i64*
  %cond_01 = icmp sgt i64 %size, 0
  br i1 %cond_01, label %body.lr.ph, label %done

body.lr.ph:                                       ; preds = %pre_entry
  %rng.promoted = load i64, i64* %rng, align 8
  br label %body

body:                                             ; preds = %body.lr.ph, %body
  %next_2.i3 = phi i64 [ %rng.promoted, %body.lr.ph ], [ %next_2.i, %body ]
  %i_12 = phi i64 [ 0, %body.lr.ph ], [ %i_2, %body ]
  %ax_0.i = mul i64 %next_2.i3, 25214903917
  %axpc_0.i = add i64 %ax_0.i, 11
  %next_2.i = srem i64 %axpc_0.i, 281474976710656
  %0 = srem i64 %next_2.i, 2
  %1 = tail call i64 @llvm.smax.i64(i64 %0, i64 0)
  %loc_0 = getelementptr inbounds i64, i64* %arr_0, i64 %i_12
  store i64 %1, i64* %loc_0, align 8
  %i_2 = add nuw nsw i64 %i_12, 1
  %exitcond.not = icmp eq i64 %i_2, %size
  br i1 %exitcond.not, label %loop.done_crit_edge, label %body

loop.done_crit_edge:                              ; preds = %body
  store i64 %next_2.i, i64* %rng, align 8
  br label %done

done:                                             ; preds = %loop.done_crit_edge, %pre_entry
  ret i64* %arr_0
}

; Function Attrs: nofree nounwind
define dso_local void @__printarray(i64 %size, i64* nocapture readonly %arr) local_unnamed_addr #0 {
pre_entry:
  %cond_01 = icmp sgt i64 %size, 0
  br i1 %cond_01, label %body, label %done

body:                                             ; preds = %pre_entry, %body
  %i_12 = phi i64 [ %i_2, %body ], [ 0, %pre_entry ]
  %loc_0 = getelementptr inbounds i64, i64* %arr, i64 %i_12
  %val_0 = load i64, i64* %loc_0, align 8
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0) #6
  %1 = tail call i32 @putchar(i32 10) #6
  %i_2 = add nuw nsw i64 %i_12, 1
  %exitcond.not = icmp eq i64 %i_2, %size
  br i1 %exitcond.not, label %done, label %body

done:                                             ; preds = %body, %pre_entry
  ret void
}

; Function Attrs: nofree nounwind
define dso_local noalias i64* @__zeroarray(i64 %size) local_unnamed_addr #0 {
pre_entry:
  %z0 = shl i64 %size, 3
  %z1 = tail call i8* @malloc(i64 %z0)
  %cond_01 = icmp sgt i64 %size, 0
  br i1 %cond_01, label %body.preheader, label %done

body.preheader:                                   ; preds = %pre_entry
  call void @llvm.memset.p0i8.i64(i8* align 8 %z1, i8 0, i64 %z0, i1 false)
  br label %done

done:                                             ; preds = %body.preheader, %pre_entry
  %arr_0 = bitcast i8* %z1 to i64*
  ret i64* %arr_0
}

; Function Attrs: argmemonly nofree norecurse nosync nounwind
define dso_local i64 @__adj2csr(i64 %num_nodes, i64* nocapture readonly %adjmat, i64* nocapture writeonly %csr_offset, i64* nocapture writeonly %csr_edges) local_unnamed_addr #5 {
pre_entry:
  %row_cond_05 = icmp sgt i64 %num_nodes, 0
  br i1 %row_cond_05, label %iter_col.preheader.us, label %row_done

iter_col.preheader.us:                            ; preds = %pre_entry, %iter_col.col_done_crit_edge.us
  %num_edges_17.us = phi i64 [ %num_edges_4.us, %iter_col.col_done_crit_edge.us ], [ 0, %pre_entry ]
  %row_16.us = phi i64 [ %row_2.us, %iter_col.col_done_crit_edge.us ], [ 0, %pre_entry ]
  %row_tmp_0.us = mul i64 %row_16.us, %num_nodes
  br label %col_body.us

col_body.us:                                      ; preds = %iter_col.preheader.us, %col_end.us
  %num_edges_24.us = phi i64 [ %num_edges_17.us, %iter_col.preheader.us ], [ %num_edges_4.us, %col_end.us ]
  %col_12.us = phi i64 [ 0, %iter_col.preheader.us ], [ %col_2.us, %col_end.us ]
  %node_idx_0.us = add i64 %col_12.us, %row_tmp_0.us
  %node_loc_0.us = getelementptr inbounds i64, i64* %adjmat, i64 %node_idx_0.us
  %node_val_0.us = load i64, i64* %node_loc_0.us, align 8
  %cond_0.us = icmp eq i64 %node_val_0.us, 1
  br i1 %cond_0.us, label %if_body.us, label %col_end.us

if_body.us:                                       ; preds = %col_body.us
  %edge_loc_0.us = getelementptr inbounds i64, i64* %csr_edges, i64 %num_edges_24.us
  store i64 %col_12.us, i64* %edge_loc_0.us, align 8
  %num_edges_3.us = add i64 %num_edges_24.us, 1
  br label %col_end.us

col_end.us:                                       ; preds = %if_body.us, %col_body.us
  %num_edges_4.us = phi i64 [ %num_edges_3.us, %if_body.us ], [ %num_edges_24.us, %col_body.us ]
  %col_2.us = add nuw nsw i64 %col_12.us, 1
  %exitcond.not = icmp eq i64 %col_2.us, %num_nodes
  br i1 %exitcond.not, label %iter_col.col_done_crit_edge.us, label %col_body.us

iter_col.col_done_crit_edge.us:                   ; preds = %col_end.us
  %offset_loc_0.us = getelementptr inbounds i64, i64* %csr_offset, i64 %row_16.us
  store i64 %num_edges_4.us, i64* %offset_loc_0.us, align 8
  %row_2.us = add nuw nsw i64 %row_16.us, 1
  %exitcond8.not = icmp eq i64 %row_2.us, %num_nodes
  br i1 %exitcond8.not, label %row_done, label %iter_col.preheader.us

row_done:                                         ; preds = %iter_col.col_done_crit_edge.us, %pre_entry
  ret i64 undef
}

; Function Attrs: nounwind
define dso_local void @__main() local_unnamed_addr #6 {
b0:
  br label %loop_body

loop_body:                                        ; preds = %b0, %loop_body
  %loop_counter_11 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %loop_body ]
  tail call void @__orig_main(i64 %loop_counter_11)
  %loop_counter_2 = add nuw nsw i64 %loop_counter_11, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 150
  br i1 %exitcond.not, label %loop_done, label %loop_body

loop_done:                                        ; preds = %loop_body
  ret void
}

; Function Attrs: nounwind
define dso_local void @__orig_main(i64 %num_nodes) local_unnamed_addr #6 {
pre_entry:
  %sqsize_0 = mul i64 %num_nodes, %num_nodes
  %z0.i = shl i64 %sqsize_0, 3
  %z1.i = tail call i8* @malloc(i64 %z0.i) #6
  %arr_0.i = bitcast i8* %z1.i to i64*
  %cond_01.i = icmp sgt i64 %sqsize_0, 0
  br i1 %cond_01.i, label %body.i, label %__zeroarray.exit.thread

body.i:                                           ; preds = %pre_entry, %body.i
  %next_2.i3.i = phi i64 [ %next_2.i.i, %body.i ], [ 2348512, %pre_entry ]
  %i_12.i = phi i64 [ %i_2.i, %body.i ], [ 0, %pre_entry ]
  %ax_0.i.i = mul i64 %next_2.i3.i, 25214903917
  %axpc_0.i.i = add i64 %ax_0.i.i, 11
  %next_2.i.i = srem i64 %axpc_0.i.i, 281474976710656
  %0 = srem i64 %next_2.i.i, 2
  %1 = tail call i64 @llvm.smax.i64(i64 %0, i64 0) #6
  %loc_0.i = getelementptr inbounds i64, i64* %arr_0.i, i64 %i_12.i
  store i64 %1, i64* %loc_0.i, align 8
  %i_2.i = add nuw nsw i64 %i_12.i, 1
  %exitcond.not.i = icmp eq i64 %i_2.i, %sqsize_0
  br i1 %exitcond.not.i, label %body.preheader.i8, label %body.i

__zeroarray.exit.thread:                          ; preds = %pre_entry
  %z1.i226 = tail call i8* @malloc(i64 %z0.i) #6
  %z1.i630 = tail call i8* @malloc(i64 %z0.i) #6
  br label %__zeroarray.exit10

body.preheader.i8:                                ; preds = %body.i
  %calloc = call i8* @calloc(i64 1, i64 %z0.i)
  %calloc35 = call i8* @calloc(i64 1, i64 %z0.i)
  br label %__zeroarray.exit10

__zeroarray.exit10:                               ; preds = %__zeroarray.exit.thread, %body.preheader.i8
  %z1.i634 = phi i8* [ %z1.i630, %__zeroarray.exit.thread ], [ %calloc35, %body.preheader.i8 ]
  %z1.i22731 = phi i8* [ %z1.i226, %__zeroarray.exit.thread ], [ %calloc, %body.preheader.i8 ]
  %arr_0.i432 = bitcast i8* %z1.i22731 to i64*
  %arr_0.i9 = bitcast i8* %z1.i634 to i64*
  %row_cond_05.i = icmp sgt i64 %num_nodes, 0
  br i1 %row_cond_05.i, label %iter_col.preheader.us.i, label %__adj2csr.exit

iter_col.preheader.us.i:                          ; preds = %__zeroarray.exit10, %iter_col.col_done_crit_edge.us.i
  %num_edges_17.us.i = phi i64 [ %num_edges_4.us.i, %iter_col.col_done_crit_edge.us.i ], [ 0, %__zeroarray.exit10 ]
  %row_16.us.i = phi i64 [ %row_2.us.i, %iter_col.col_done_crit_edge.us.i ], [ 0, %__zeroarray.exit10 ]
  %row_tmp_0.us.i = mul i64 %row_16.us.i, %num_nodes
  br label %col_body.us.i

col_body.us.i:                                    ; preds = %col_end.us.i, %iter_col.preheader.us.i
  %num_edges_24.us.i = phi i64 [ %num_edges_17.us.i, %iter_col.preheader.us.i ], [ %num_edges_4.us.i, %col_end.us.i ]
  %col_12.us.i = phi i64 [ 0, %iter_col.preheader.us.i ], [ %col_2.us.i, %col_end.us.i ]
  %node_idx_0.us.i = add i64 %col_12.us.i, %row_tmp_0.us.i
  %node_loc_0.us.i = getelementptr inbounds i64, i64* %arr_0.i, i64 %node_idx_0.us.i
  %node_val_0.us.i = load i64, i64* %node_loc_0.us.i, align 8
  %cond_0.us.i = icmp eq i64 %node_val_0.us.i, 1
  br i1 %cond_0.us.i, label %if_body.us.i, label %col_end.us.i

if_body.us.i:                                     ; preds = %col_body.us.i
  %edge_loc_0.us.i = getelementptr inbounds i64, i64* %arr_0.i9, i64 %num_edges_24.us.i
  store i64 %col_12.us.i, i64* %edge_loc_0.us.i, align 8
  %num_edges_3.us.i = add i64 %num_edges_24.us.i, 1
  br label %col_end.us.i

col_end.us.i:                                     ; preds = %if_body.us.i, %col_body.us.i
  %num_edges_4.us.i = phi i64 [ %num_edges_3.us.i, %if_body.us.i ], [ %num_edges_24.us.i, %col_body.us.i ]
  %col_2.us.i = add nuw nsw i64 %col_12.us.i, 1
  %exitcond.not.i11 = icmp eq i64 %col_2.us.i, %num_nodes
  br i1 %exitcond.not.i11, label %iter_col.col_done_crit_edge.us.i, label %col_body.us.i

iter_col.col_done_crit_edge.us.i:                 ; preds = %col_end.us.i
  %offset_loc_0.us.i = getelementptr inbounds i64, i64* %arr_0.i432, i64 %row_16.us.i
  store i64 %num_edges_4.us.i, i64* %offset_loc_0.us.i, align 8
  %row_2.us.i = add nuw nsw i64 %row_16.us.i, 1
  %exitcond8.not.i = icmp eq i64 %row_2.us.i, %num_nodes
  br i1 %exitcond8.not.i, label %__adj2csr.exit, label %iter_col.preheader.us.i

__adj2csr.exit:                                   ; preds = %iter_col.col_done_crit_edge.us.i, %__zeroarray.exit10
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %num_nodes) #6
  %3 = tail call i32 @putchar(i32 10) #6
  %4 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 undef) #6
  %5 = tail call i32 @putchar(i32 10) #6
  br i1 %cond_01.i, label %body.i17, label %__printarray.exit

body.i17:                                         ; preds = %__adj2csr.exit, %body.i17
  %i_12.i13 = phi i64 [ %i_2.i15, %body.i17 ], [ 0, %__adj2csr.exit ]
  %loc_0.i14 = getelementptr inbounds i64, i64* %arr_0.i, i64 %i_12.i13
  %val_0.i = load i64, i64* %loc_0.i14, align 8
  %6 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0.i) #6
  %7 = tail call i32 @putchar(i32 10) #6
  %i_2.i15 = add nuw nsw i64 %i_12.i13, 1
  %exitcond.not.i16 = icmp eq i64 %i_2.i15, %sqsize_0
  br i1 %exitcond.not.i16, label %__printarray.exit, label %body.i17

__printarray.exit:                                ; preds = %body.i17, %__adj2csr.exit
  br i1 %row_cond_05.i, label %body.i24, label %__printarray.exit25

body.i24:                                         ; preds = %__printarray.exit, %body.i24
  %i_12.i19 = phi i64 [ %i_2.i22, %body.i24 ], [ 0, %__printarray.exit ]
  %loc_0.i20 = getelementptr inbounds i64, i64* %arr_0.i432, i64 %i_12.i19
  %val_0.i21 = load i64, i64* %loc_0.i20, align 8
  %8 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %val_0.i21) #6
  %9 = tail call i32 @putchar(i32 10) #6
  %i_2.i22 = add nuw nsw i64 %i_12.i19, 1
  %exitcond.not.i23 = icmp eq i64 %i_2.i22, %num_nodes
  br i1 %exitcond.not.i23, label %__printarray.exit25, label %body.i24

__printarray.exit25:                              ; preds = %body.i24, %__printarray.exit
  tail call void @free(i8* %z1.i)
  tail call void @free(i8* %z1.i22731)
  tail call void @free(i8* %z1.i634)
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
  tail call void @__orig_main(i64 %loop_counter_11.i) #6
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 150
  br i1 %exitcond.not.i, label %__main.exit, label %loop_body.i

__main.exit:                                      ; preds = %loop_body.i
  ret i32 0
}

; Function Attrs: nocallback nofree nosync nounwind readnone speculatable willreturn
declare i64 @llvm.smax.i64(i64, i64) #7

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
attributes #4 = { argmemonly mustprogress nofree norecurse nosync nounwind willreturn }
attributes #5 = { argmemonly nofree norecurse nosync nounwind }
attributes #6 = { nounwind }
attributes #7 = { nocallback nofree nosync nounwind readnone speculatable willreturn }
attributes #8 = { argmemonly nofree nounwind willreturn writeonly }
attributes #9 = { inaccessiblememonly nofree nounwind willreturn allocsize(0,1) }
attributes #10 = { cold minsize noreturn }
attributes #11 = { noinline }
