; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmps88j5S/adj2csr-init.ll'
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

; Function Attrs: mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: readwrite)
define dso_local i64 @__rand(ptr nocapture %seq, i64 %max) local_unnamed_addr #4 {
pre_entry:
  %x_0 = load i64, ptr %seq, align 8
  %x_0.fr = freeze i64 %x_0
  %ax_0 = mul i64 %x_0.fr, 25214903917
  %axpc_0 = add i64 %ax_0, 11
  %next_2 = srem i64 %axpc_0, 281474976710656
  store i64 %next_2, ptr %seq, align 8
  %0 = srem i64 %next_2, %max
  ret i64 %0
}

; Function Attrs: nofree nounwind memory(write, argmem: readwrite, inaccessiblemem: readwrite)
define dso_local noalias noundef ptr @__randarray(i64 %size, ptr nocapture %rng) local_unnamed_addr #5 {
pre_entry:
  %z0 = shl i64 %size, 3
  %z1 = tail call ptr @malloc(i64 %z0)
  %cond_01 = icmp sgt i64 %size, 0
  br i1 %cond_01, label %body.lr.ph, label %done

body.lr.ph:                                       ; preds = %pre_entry
  %rng.promoted = load i64, ptr %rng, align 8
  %0 = freeze i64 %rng.promoted
  br label %body

body:                                             ; preds = %body.lr.ph, %body
  %next_2.i3 = phi i64 [ %0, %body.lr.ph ], [ %next_2.i, %body ]
  %i_12 = phi i64 [ 0, %body.lr.ph ], [ %i_2, %body ]
  %ax_0.i = mul i64 %next_2.i3, 25214903917
  %axpc_0.i = add i64 %ax_0.i, 11
  %next_2.i = srem i64 %axpc_0.i, 281474976710656
  %1 = srem i64 %next_2.i, 2
  %spec.select = tail call i64 @llvm.smax.i64(i64 %1, i64 0)
  %loc_0 = getelementptr inbounds i64, ptr %z1, i64 %i_12
  store i64 %spec.select, ptr %loc_0, align 8
  %i_2 = add nuw nsw i64 %i_12, 1
  %exitcond.not = icmp eq i64 %i_2, %size
  br i1 %exitcond.not, label %loop.done_crit_edge, label %body

loop.done_crit_edge:                              ; preds = %body
  store i64 %next_2.i, ptr %rng, align 8
  br label %done

done:                                             ; preds = %loop.done_crit_edge, %pre_entry
  ret ptr %z1
}

; Function Attrs: nofree nounwind
define dso_local void @__printarray(i64 %size, ptr nocapture readonly %arr) local_unnamed_addr #0 {
pre_entry:
  %cond_01 = icmp sgt i64 %size, 0
  br i1 %cond_01, label %body, label %done

body:                                             ; preds = %pre_entry, %body
  %i_12 = phi i64 [ %i_2, %body ], [ 0, %pre_entry ]
  %loc_0 = getelementptr inbounds i64, ptr %arr, i64 %i_12
  %val_0 = load i64, ptr %loc_0, align 8
  %0 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %val_0)
  %1 = tail call i32 @putchar(i32 10)
  %i_2 = add nuw nsw i64 %i_12, 1
  %exitcond.not = icmp eq i64 %i_2, %size
  br i1 %exitcond.not, label %done, label %body

done:                                             ; preds = %body, %pre_entry
  ret void
}

; Function Attrs: mustprogress nofree nounwind willreturn memory(write, argmem: none, inaccessiblemem: readwrite)
define dso_local noalias noundef ptr @__zeroarray(i64 %size) local_unnamed_addr #6 {
pre_entry:
  %z0 = shl i64 %size, 3
  %z1 = tail call ptr @malloc(i64 %z0)
  %cond_01 = icmp sgt i64 %size, 0
  br i1 %cond_01, label %body.preheader, label %done

body.preheader:                                   ; preds = %pre_entry
  tail call void @llvm.memset.p0.i64(ptr align 8 %z1, i8 0, i64 %z0, i1 false)
  br label %done

done:                                             ; preds = %body.preheader, %pre_entry
  ret ptr %z1
}

; Function Attrs: nofree norecurse nosync nounwind memory(argmem: readwrite)
define dso_local i64 @__adj2csr(i64 %num_nodes, ptr nocapture readonly %adjmat, ptr nocapture writeonly %csr_offset, ptr nocapture writeonly %csr_edges) local_unnamed_addr #7 {
pre_entry:
  %row_cond_04 = icmp sgt i64 %num_nodes, 0
  br i1 %row_cond_04, label %iter_col.preheader.us, label %row_done

iter_col.preheader.us:                            ; preds = %pre_entry, %iter_col.col_done_crit_edge.us
  %num_edges_16.us = phi i64 [ %num_edges_4.us, %iter_col.col_done_crit_edge.us ], [ 0, %pre_entry ]
  %row_15.us = phi i64 [ %row_2.us, %iter_col.col_done_crit_edge.us ], [ 0, %pre_entry ]
  %row_tmp_0.us = mul i64 %row_15.us, %num_nodes
  %0 = getelementptr i64, ptr %adjmat, i64 %row_tmp_0.us
  br label %col_body.us

col_body.us:                                      ; preds = %iter_col.preheader.us, %col_end.us
  %num_edges_23.us = phi i64 [ %num_edges_16.us, %iter_col.preheader.us ], [ %num_edges_4.us, %col_end.us ]
  %col_12.us = phi i64 [ 0, %iter_col.preheader.us ], [ %col_2.us, %col_end.us ]
  %node_loc_0.us = getelementptr i64, ptr %0, i64 %col_12.us
  %node_val_0.us = load i64, ptr %node_loc_0.us, align 8
  %cond_0.us = icmp eq i64 %node_val_0.us, 1
  br i1 %cond_0.us, label %if_body.us, label %col_end.us

if_body.us:                                       ; preds = %col_body.us
  %edge_loc_0.us = getelementptr inbounds i64, ptr %csr_edges, i64 %num_edges_23.us
  store i64 %col_12.us, ptr %edge_loc_0.us, align 8
  %num_edges_3.us = add i64 %num_edges_23.us, 1
  br label %col_end.us

col_end.us:                                       ; preds = %if_body.us, %col_body.us
  %num_edges_4.us = phi i64 [ %num_edges_3.us, %if_body.us ], [ %num_edges_23.us, %col_body.us ]
  %col_2.us = add nuw nsw i64 %col_12.us, 1
  %exitcond.not = icmp eq i64 %col_2.us, %num_nodes
  br i1 %exitcond.not, label %iter_col.col_done_crit_edge.us, label %col_body.us

iter_col.col_done_crit_edge.us:                   ; preds = %col_end.us
  %offset_loc_0.us = getelementptr inbounds i64, ptr %csr_offset, i64 %row_15.us
  store i64 %num_edges_4.us, ptr %offset_loc_0.us, align 8
  %row_2.us = add nuw nsw i64 %row_15.us, 1
  %exitcond7.not = icmp eq i64 %row_2.us, %num_nodes
  br i1 %exitcond7.not, label %row_done, label %iter_col.preheader.us

row_done:                                         ; preds = %iter_col.col_done_crit_edge.us, %pre_entry
  ret i64 poison
}

; Function Attrs: nounwind
define dso_local void @__main() local_unnamed_addr #8 {
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
define dso_local void @__orig_main(i64 %num_nodes) local_unnamed_addr #8 {
pre_entry:
  %sqsize_0 = mul i64 %num_nodes, %num_nodes
  %z0.i = shl i64 %sqsize_0, 3
  %z1.i = tail call ptr @malloc(i64 %z0.i)
  %cond_01.i = icmp sgt i64 %sqsize_0, 0
  br i1 %cond_01.i, label %body.i, label %__randarray.exit.thread

__randarray.exit.thread:                          ; preds = %pre_entry
  %z1.i230 = tail call ptr @malloc(i64 %z0.i)
  %z1.i533 = tail call ptr @malloc(i64 %z0.i)
  br label %__zeroarray.exit8

body.i:                                           ; preds = %pre_entry, %body.i
  %next_2.i3.i = phi i64 [ %next_2.i.i, %body.i ], [ 2348512, %pre_entry ]
  %i_12.i = phi i64 [ %i_2.i, %body.i ], [ 0, %pre_entry ]
  %ax_0.i.i = mul i64 %next_2.i3.i, 25214903917
  %axpc_0.i.i = add i64 %ax_0.i.i, 11
  %next_2.i.i = srem i64 %axpc_0.i.i, 281474976710656
  %0 = srem i64 %next_2.i.i, 2
  %spec.select.i = tail call i64 @llvm.smax.i64(i64 %0, i64 0)
  %loc_0.i = getelementptr inbounds i64, ptr %z1.i, i64 %i_12.i
  store i64 %spec.select.i, ptr %loc_0.i, align 8
  %i_2.i = add nuw nsw i64 %i_12.i, 1
  %exitcond.not.i = icmp eq i64 %i_2.i, %sqsize_0
  br i1 %exitcond.not.i, label %__randarray.exit, label %body.i

__randarray.exit:                                 ; preds = %body.i
  %calloc = tail call ptr @calloc(i64 1, i64 %z0.i)
  %calloc36 = tail call ptr @calloc(i64 1, i64 %z0.i)
  br label %__zeroarray.exit8

__zeroarray.exit8:                                ; preds = %__randarray.exit.thread, %__randarray.exit
  %z1.i535 = phi ptr [ %z1.i533, %__randarray.exit.thread ], [ %calloc36, %__randarray.exit ]
  %z1.i23134 = phi ptr [ %z1.i230, %__randarray.exit.thread ], [ %calloc, %__randarray.exit ]
  %row_cond_04.i = icmp sgt i64 %num_nodes, 0
  br i1 %row_cond_04.i, label %iter_col.preheader.us.i, label %__adj2csr.exit

iter_col.preheader.us.i:                          ; preds = %__zeroarray.exit8, %iter_col.col_done_crit_edge.us.i
  %num_edges_16.us.i = phi i64 [ %num_edges_4.us.i, %iter_col.col_done_crit_edge.us.i ], [ 0, %__zeroarray.exit8 ]
  %row_15.us.i = phi i64 [ %row_2.us.i, %iter_col.col_done_crit_edge.us.i ], [ 0, %__zeroarray.exit8 ]
  %row_tmp_0.us.i = mul i64 %row_15.us.i, %num_nodes
  %1 = getelementptr i64, ptr %z1.i, i64 %row_tmp_0.us.i
  br label %col_body.us.i

col_body.us.i:                                    ; preds = %col_end.us.i, %iter_col.preheader.us.i
  %num_edges_23.us.i = phi i64 [ %num_edges_16.us.i, %iter_col.preheader.us.i ], [ %num_edges_4.us.i, %col_end.us.i ]
  %col_12.us.i = phi i64 [ 0, %iter_col.preheader.us.i ], [ %col_2.us.i, %col_end.us.i ]
  %node_loc_0.us.i = getelementptr i64, ptr %1, i64 %col_12.us.i
  %node_val_0.us.i = load i64, ptr %node_loc_0.us.i, align 8
  %cond_0.us.i = icmp eq i64 %node_val_0.us.i, 1
  br i1 %cond_0.us.i, label %if_body.us.i, label %col_end.us.i

if_body.us.i:                                     ; preds = %col_body.us.i
  %edge_loc_0.us.i = getelementptr inbounds i64, ptr %z1.i535, i64 %num_edges_23.us.i
  store i64 %col_12.us.i, ptr %edge_loc_0.us.i, align 8
  %num_edges_3.us.i = add i64 %num_edges_23.us.i, 1
  br label %col_end.us.i

col_end.us.i:                                     ; preds = %if_body.us.i, %col_body.us.i
  %num_edges_4.us.i = phi i64 [ %num_edges_3.us.i, %if_body.us.i ], [ %num_edges_23.us.i, %col_body.us.i ]
  %col_2.us.i = add nuw nsw i64 %col_12.us.i, 1
  %exitcond.not.i9 = icmp eq i64 %col_2.us.i, %num_nodes
  br i1 %exitcond.not.i9, label %iter_col.col_done_crit_edge.us.i, label %col_body.us.i

iter_col.col_done_crit_edge.us.i:                 ; preds = %col_end.us.i
  %offset_loc_0.us.i = getelementptr inbounds i64, ptr %z1.i23134, i64 %row_15.us.i
  store i64 %num_edges_4.us.i, ptr %offset_loc_0.us.i, align 8
  %row_2.us.i = add nuw nsw i64 %row_15.us.i, 1
  %exitcond7.not.i = icmp eq i64 %row_2.us.i, %num_nodes
  br i1 %exitcond7.not.i, label %__adj2csr.exit, label %iter_col.preheader.us.i

__adj2csr.exit:                                   ; preds = %iter_col.col_done_crit_edge.us.i, %__zeroarray.exit8
  %2 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %num_nodes)
  %3 = tail call i32 @putchar(i32 10)
  %4 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 poison)
  %5 = tail call i32 @putchar(i32 10)
  br i1 %cond_01.i, label %body.i11, label %__printarray.exit

body.i11:                                         ; preds = %__adj2csr.exit, %body.i11
  %i_12.i12 = phi i64 [ %i_2.i14, %body.i11 ], [ 0, %__adj2csr.exit ]
  %loc_0.i13 = getelementptr inbounds i64, ptr %z1.i, i64 %i_12.i12
  %val_0.i = load i64, ptr %loc_0.i13, align 8
  %6 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %val_0.i)
  %7 = tail call i32 @putchar(i32 10)
  %i_2.i14 = add nuw nsw i64 %i_12.i12, 1
  %exitcond.not.i15 = icmp eq i64 %i_2.i14, %sqsize_0
  br i1 %exitcond.not.i15, label %__printarray.exit, label %body.i11

__printarray.exit:                                ; preds = %body.i11, %__adj2csr.exit
  br i1 %row_cond_04.i, label %body.i17, label %__printarray.exit29

body.i17:                                         ; preds = %__printarray.exit, %body.i17
  %i_12.i18 = phi i64 [ %i_2.i21, %body.i17 ], [ 0, %__printarray.exit ]
  %loc_0.i19 = getelementptr inbounds i64, ptr %z1.i23134, i64 %i_12.i18
  %val_0.i20 = load i64, ptr %loc_0.i19, align 8
  %8 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %val_0.i20)
  %9 = tail call i32 @putchar(i32 10)
  %i_2.i21 = add nuw nsw i64 %i_12.i18, 1
  %exitcond.not.i22 = icmp eq i64 %i_2.i21, %num_nodes
  br i1 %exitcond.not.i22, label %__printarray.exit29, label %body.i17

__printarray.exit29:                              ; preds = %body.i17, %__printarray.exit
  %val_0.i27 = load i64, ptr %z1.i535, align 8
  %10 = tail call i32 (ptr, ...) @printf(ptr nonnull dereferenceable(1) @.str.2, i64 %val_0.i27)
  %11 = tail call i32 @putchar(i32 10)
  tail call void @free(ptr %z1.i)
  tail call void @free(ptr %z1.i23134)
  tail call void @free(ptr %z1.i535)
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
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 150
  br i1 %exitcond.not.i, label %__main.exit, label %loop_body.i

__main.exit:                                      ; preds = %loop_body.i
  ret i32 0
}

; Function Attrs: nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i64 @llvm.smax.i64(i64, i64) #9

; Function Attrs: nocallback nofree nounwind willreturn memory(argmem: write)
declare void @llvm.memset.p0.i64(ptr nocapture writeonly, i8, i64, i1 immarg) #10

; Function Attrs: nofree nounwind willreturn allockind("alloc,zeroed") allocsize(0,1) memory(inaccessiblemem: readwrite)
declare noalias noundef ptr @calloc(i64 noundef, i64 noundef) local_unnamed_addr #11

attributes #0 = { nofree nounwind }
attributes #1 = { mustprogress nofree nounwind willreturn allockind("alloc,uninitialized") allocsize(0) memory(inaccessiblemem: readwrite) "alloc-family"="malloc" }
attributes #2 = { mustprogress nounwind willreturn allockind("free") memory(argmem: readwrite, inaccessiblemem: readwrite) "alloc-family"="malloc" }
attributes #3 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: read) }
attributes #4 = { mustprogress nofree norecurse nosync nounwind willreturn memory(argmem: readwrite) }
attributes #5 = { nofree nounwind memory(write, argmem: readwrite, inaccessiblemem: readwrite) }
attributes #6 = { mustprogress nofree nounwind willreturn memory(write, argmem: none, inaccessiblemem: readwrite) }
attributes #7 = { nofree norecurse nosync nounwind memory(argmem: readwrite) }
attributes #8 = { nounwind }
attributes #9 = { nocallback nofree nosync nounwind speculatable willreturn memory(none) }
attributes #10 = { nocallback nofree nounwind willreturn memory(argmem: write) }
attributes #11 = { nofree nounwind willreturn allockind("alloc,zeroed") allocsize(0,1) memory(inaccessiblemem: readwrite) "alloc-family"="malloc" }
