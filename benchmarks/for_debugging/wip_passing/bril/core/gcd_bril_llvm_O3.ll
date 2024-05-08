; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpOEpxnN/compile.ll'
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

; Function Attrs: argmemonly mustprogress nofree norecurse nosync nounwind readonly willreturn
define dso_local i32 @btoi(i8* nocapture readonly %0) local_unnamed_addr #1 {
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

; Function Attrs: nofree nounwind
define dso_local void @__main() local_unnamed_addr #0 {
b0:
  br label %inner_cond.preheader

inner_cond.preheader:                             ; preds = %b0, %inner_done
  %loop_counter_13 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %inner_done ]
  br label %inner_body

inner_body:                                       ; preds = %inner_cond.preheader, %__orig_main.exit
  %inner_counter_11 = phi i64 [ 10, %inner_cond.preheader ], [ %inner_counter_2, %__orig_main.exit ]
  %v2_01.i = icmp ult i64 %loop_counter_13, %inner_counter_11
  %v3_12.i = sub nsw i64 %loop_counter_13, %inner_counter_11
  %v3_03.i = sub nsw i64 %inner_counter_11, %loop_counter_13
  %v3_24.i = select i1 %v2_01.i, i64 %v3_03.i, i64 %v3_12.i
  %v4_05.i = icmp eq i64 %v3_24.i, 0
  br i1 %v4_05.i, label %__orig_main.exit, label %update.val.i

update.val.i:                                     ; preds = %inner_body, %update.val.i
  %v3_010.i = phi i64 [ %v3_0.i, %update.val.i ], [ %v3_03.i, %inner_body ]
  %v3_19.i = phi i64 [ %v3_1.i, %update.val.i ], [ %v3_12.i, %inner_body ]
  %v2_08.i = phi i1 [ %v2_0.i, %update.val.i ], [ %v2_01.i, %inner_body ]
  %v0_17.i = phi i64 [ %v0_1.v3_2.i, %update.val.i ], [ %loop_counter_13, %inner_body ]
  %v1_16.i = phi i64 [ %v3_2.v1_1.i, %update.val.i ], [ %inner_counter_11, %inner_body ]
  %v3_2.v1_1.i = select i1 %v2_08.i, i64 %v3_010.i, i64 %v1_16.i
  %v0_1.v3_2.i = select i1 %v2_08.i, i64 %v0_17.i, i64 %v3_19.i
  %v2_0.i = icmp slt i64 %v0_1.v3_2.i, %v3_2.v1_1.i
  %v3_1.i = sub i64 %v0_1.v3_2.i, %v3_2.v1_1.i
  %v3_0.i = sub i64 %v3_2.v1_1.i, %v0_1.v3_2.i
  %v3_2.i = select i1 %v2_0.i, i64 %v3_0.i, i64 %v3_1.i
  %v4_0.i = icmp eq i64 %v3_2.i, 0
  br i1 %v4_0.i, label %__orig_main.exit, label %update.val.i

__orig_main.exit:                                 ; preds = %update.val.i, %inner_body
  %v1_1.lcssa.i = phi i64 [ %inner_counter_11, %inner_body ], [ %v3_2.v1_1.i, %update.val.i ]
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %v1_1.lcssa.i) #3
  %1 = tail call i32 @putchar(i32 10) #3
  %inner_counter_2 = add nuw nsw i64 %inner_counter_11, 1
  %exitcond.not = icmp eq i64 %inner_counter_2, 1000
  br i1 %exitcond.not, label %inner_done, label %inner_body

inner_done:                                       ; preds = %__orig_main.exit
  %loop_counter_2 = add nuw nsw i64 %loop_counter_13, 1
  %exitcond4.not = icmp eq i64 %loop_counter_2, 1000
  br i1 %exitcond4.not, label %loop_done, label %inner_cond.preheader

loop_done:                                        ; preds = %inner_done
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %op1, i64 %op2) local_unnamed_addr #0 {
pre_entry:
  %v2_01 = icmp slt i64 %op1, %op2
  %v3_12 = sub i64 %op1, %op2
  %v3_03 = sub i64 %op2, %op1
  %v3_24 = select i1 %v2_01, i64 %v3_03, i64 %v3_12
  %v4_05 = icmp eq i64 %v3_24, 0
  br i1 %v4_05, label %program.end, label %update.val

update.val:                                       ; preds = %pre_entry, %update.val
  %v3_010 = phi i64 [ %v3_0, %update.val ], [ %v3_03, %pre_entry ]
  %v3_19 = phi i64 [ %v3_1, %update.val ], [ %v3_12, %pre_entry ]
  %v2_08 = phi i1 [ %v2_0, %update.val ], [ %v2_01, %pre_entry ]
  %v0_17 = phi i64 [ %v0_1.v3_2, %update.val ], [ %op1, %pre_entry ]
  %v1_16 = phi i64 [ %v3_2.v1_1, %update.val ], [ %op2, %pre_entry ]
  %v3_2.v1_1 = select i1 %v2_08, i64 %v3_010, i64 %v1_16
  %v0_1.v3_2 = select i1 %v2_08, i64 %v0_17, i64 %v3_19
  %v2_0 = icmp slt i64 %v0_1.v3_2, %v3_2.v1_1
  %v3_1 = sub i64 %v0_1.v3_2, %v3_2.v1_1
  %v3_0 = sub i64 %v3_2.v1_1, %v0_1.v3_2
  %v3_2 = select i1 %v2_0, i64 %v3_0, i64 %v3_1
  %v4_0 = icmp eq i64 %v3_2, 0
  br i1 %v4_0, label %program.end, label %update.val

program.end:                                      ; preds = %update.val, %pre_entry
  %v1_1.lcssa = phi i64 [ %op2, %pre_entry ], [ %v3_2.v1_1, %update.val ]
  %0 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %v1_1.lcssa) #3
  %1 = tail call i32 @putchar(i32 10) #3
  ret void
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %inner_cond.preheader.i, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #4
  ret i32 0

inner_cond.preheader.i:                           ; preds = %0, %inner_done.i
  %loop_counter_13.i = phi i64 [ %loop_counter_2.i, %inner_done.i ], [ 10, %0 ]
  br label %inner_body.i

inner_body.i:                                     ; preds = %__orig_main.exit.i, %inner_cond.preheader.i
  %inner_counter_11.i = phi i64 [ 10, %inner_cond.preheader.i ], [ %inner_counter_2.i, %__orig_main.exit.i ]
  %v2_01.i.i = icmp ult i64 %loop_counter_13.i, %inner_counter_11.i
  %v3_12.i.i = sub nsw i64 %loop_counter_13.i, %inner_counter_11.i
  %v3_03.i.i = sub nsw i64 %inner_counter_11.i, %loop_counter_13.i
  %v3_24.i.i = select i1 %v2_01.i.i, i64 %v3_03.i.i, i64 %v3_12.i.i
  %v4_05.i.i = icmp eq i64 %v3_24.i.i, 0
  br i1 %v4_05.i.i, label %__orig_main.exit.i, label %update.val.i.i

update.val.i.i:                                   ; preds = %inner_body.i, %update.val.i.i
  %v3_010.i.i = phi i64 [ %v3_0.i.i, %update.val.i.i ], [ %v3_03.i.i, %inner_body.i ]
  %v3_19.i.i = phi i64 [ %v3_1.i.i, %update.val.i.i ], [ %v3_12.i.i, %inner_body.i ]
  %v2_08.i.i = phi i1 [ %v2_0.i.i, %update.val.i.i ], [ %v2_01.i.i, %inner_body.i ]
  %v0_17.i.i = phi i64 [ %v0_1.v3_2.i.i, %update.val.i.i ], [ %loop_counter_13.i, %inner_body.i ]
  %v1_16.i.i = phi i64 [ %v3_2.v1_1.i.i, %update.val.i.i ], [ %inner_counter_11.i, %inner_body.i ]
  %v3_2.v1_1.i.i = select i1 %v2_08.i.i, i64 %v3_010.i.i, i64 %v1_16.i.i
  %v0_1.v3_2.i.i = select i1 %v2_08.i.i, i64 %v0_17.i.i, i64 %v3_19.i.i
  %v2_0.i.i = icmp slt i64 %v0_1.v3_2.i.i, %v3_2.v1_1.i.i
  %v3_1.i.i = sub i64 %v0_1.v3_2.i.i, %v3_2.v1_1.i.i
  %v3_0.i.i = sub i64 %v3_2.v1_1.i.i, %v0_1.v3_2.i.i
  %v3_2.i.i = select i1 %v2_0.i.i, i64 %v3_0.i.i, i64 %v3_1.i.i
  %v4_0.i.i = icmp eq i64 %v3_2.i.i, 0
  br i1 %v4_0.i.i, label %__orig_main.exit.i, label %update.val.i.i

__orig_main.exit.i:                               ; preds = %update.val.i.i, %inner_body.i
  %v1_1.lcssa.i.i = phi i64 [ %inner_counter_11.i, %inner_body.i ], [ %v3_2.v1_1.i.i, %update.val.i.i ]
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %v1_1.lcssa.i.i) #3
  %3 = tail call i32 @putchar(i32 10) #3
  %inner_counter_2.i = add nuw nsw i64 %inner_counter_11.i, 1
  %exitcond.not.i = icmp eq i64 %inner_counter_2.i, 1000
  br i1 %exitcond.not.i, label %inner_done.i, label %inner_body.i

inner_done.i:                                     ; preds = %__orig_main.exit.i
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_13.i, 1
  %exitcond4.not.i = icmp eq i64 %loop_counter_2.i, 1000
  br i1 %exitcond4.not.i, label %__main.exit, label %inner_cond.preheader.i

__main.exit:                                      ; preds = %inner_done.i
  ret i32 0
}

; Function Attrs: cold minsize noreturn
define internal void @main.cold.1(i32 %0) #2 {
newFuncRoot:
  br label %1

1:                                                ; preds = %newFuncRoot
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([33 x i8], [33 x i8]* @.str.4, i64 0, i64 0), i32 0, i32 %0)
  tail call void @exit(i32 2)
  unreachable
}

attributes #0 = { nofree nounwind }
attributes #1 = { argmemonly mustprogress nofree norecurse nosync nounwind readonly willreturn }
attributes #2 = { cold minsize noreturn }
attributes #3 = { nounwind }
attributes #4 = { noinline }
