; ModuleID = '/var/folders/jw/f07sz9zx0wqck930wjllkpyr0000gn/T/.tmpU6uiwY/compile.ll'
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
  br label %else.7.i.preheader.i.i.preheader

else.7.i.preheader.i.i.preheader:                 ; preds = %__orig_main.exit, %b0
  %loop_counter_12 = phi i64 [ 10, %b0 ], [ %loop_counter_2, %__orig_main.exit ]
  br label %else.7.i.preheader.i.i

else.7.i.preheader.i.i:                           ; preds = %else.7.i.preheader.i.i.preheader, %else.7.i.i
  %b_15.i.i = phi i64 [ %v15_0.i.i, %else.7.i.i ], [ %loop_counter_12, %else.7.i.preheader.i.i.preheader ]
  br label %else.7.i.i.i

else.7.i.i.i:                                     ; preds = %else.12.i.i.i, %else.7.i.preheader.i.i
  %b.tr6.i.i.i = phi i64 [ %0, %else.12.i.i.i ], [ %b_15.i.i, %else.7.i.preheader.i.i ]
  %a.tr5.i.i.i = phi i64 [ %b.tr6.i.i.i, %else.12.i.i.i ], [ %loop_counter_12, %else.7.i.preheader.i.i ]
  %v15.i.i.i = icmp eq i64 %b.tr6.i.i.i, 0
  br i1 %v15.i.i.i, label %else.7.i.i, label %else.12.i.i.i

else.12.i.i.i:                                    ; preds = %else.7.i.i.i
  %0 = srem i64 %a.tr5.i.i.i, %b.tr6.i.i.i
  %1 = tail call i64 @llvm.smax.i64(i64 %0, i64 %b.tr6.i.i.i) #6
  %v10_0.i.i.i = icmp eq i64 %1, 0
  br i1 %v10_0.i.i.i, label %__gcd.exit.i.i, label %else.7.i.i.i

__gcd.exit.i.i:                                   ; preds = %else.12.i.i.i
  %2 = tail call i64 @llvm.smin.i64(i64 %0, i64 %b.tr6.i.i.i) #6
  %v10_0.i.i = icmp eq i64 %2, 1
  br i1 %v10_0.i.i, label %then.7.i.i, label %else.7.i.i

then.7.i.i:                                       ; preds = %__gcd.exit.i.i
  %3 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %b_15.i.i) #6
  %4 = tail call i32 @putchar(i32 10) #6
  br label %else.7.i.i

else.7.i.i:                                       ; preds = %else.7.i.i.i, %then.7.i.i, %__gcd.exit.i.i
  %v15_0.i.i = add nsw i64 %b_15.i.i, -1
  %v4_0.i.i = icmp sgt i64 %b_15.i.i, 1
  br i1 %v4_0.i.i, label %else.7.i.preheader.i.i, label %__orig_main.exit

__orig_main.exit:                                 ; preds = %else.7.i.i
  %loop_counter_2 = add nuw nsw i64 %loop_counter_12, 1
  %exitcond.not = icmp eq i64 %loop_counter_2, 3000
  br i1 %exitcond.not, label %loop_done, label %else.7.i.preheader.i.i.preheader

loop_done:                                        ; preds = %__orig_main.exit
  ret void
}

; Function Attrs: nofree nounwind
define dso_local void @__orig_main(i64 %v0) local_unnamed_addr #0 {
pre_entry:
  %v4_04.i = icmp sgt i64 %v0, 0
  br i1 %v4_04.i, label %else.7.i.preheader.i, label %__relative_primes.exit

else.7.i.preheader.i:                             ; preds = %pre_entry, %else.7.i
  %b_15.i = phi i64 [ %v15_0.i, %else.7.i ], [ %v0, %pre_entry ]
  br label %else.7.i.i

else.7.i.i:                                       ; preds = %else.12.i.i, %else.7.i.preheader.i
  %b.tr6.i.i = phi i64 [ %0, %else.12.i.i ], [ %b_15.i, %else.7.i.preheader.i ]
  %a.tr5.i.i = phi i64 [ %b.tr6.i.i, %else.12.i.i ], [ %v0, %else.7.i.preheader.i ]
  %v15.i.i = icmp eq i64 %b.tr6.i.i, 0
  br i1 %v15.i.i, label %else.7.i, label %else.12.i.i

else.12.i.i:                                      ; preds = %else.7.i.i
  %0 = srem i64 %a.tr5.i.i, %b.tr6.i.i
  %1 = tail call i64 @llvm.smax.i64(i64 %0, i64 %b.tr6.i.i) #6
  %v10_0.i.i = icmp eq i64 %1, 0
  br i1 %v10_0.i.i, label %__gcd.exit.i, label %else.7.i.i

__gcd.exit.i:                                     ; preds = %else.12.i.i
  %2 = tail call i64 @llvm.smin.i64(i64 %0, i64 %b.tr6.i.i) #6
  %v10_0.i = icmp eq i64 %2, 1
  br i1 %v10_0.i, label %then.7.i, label %else.7.i

then.7.i:                                         ; preds = %__gcd.exit.i
  %3 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %b_15.i) #6
  %4 = tail call i32 @putchar(i32 10) #6
  br label %else.7.i

else.7.i:                                         ; preds = %else.7.i.i, %then.7.i, %__gcd.exit.i
  %v15_0.i = add nsw i64 %b_15.i, -1
  %v4_0.i = icmp sgt i64 %b_15.i, 1
  br i1 %v4_0.i, label %else.7.i.preheader.i, label %__relative_primes.exit

__relative_primes.exit:                           ; preds = %else.7.i, %pre_entry
  ret void
}

; Function Attrs: mustprogress nofree norecurse nosync nounwind readnone willreturn
define dso_local i64 @__mod(i64 %a, i64 %b) local_unnamed_addr #2 {
pre_entry:
  %0 = srem i64 %a, %b
  ret i64 %0
}

; Function Attrs: nofree nosync nounwind readnone
define dso_local i64 @__gcd(i64 %a, i64 %b) local_unnamed_addr #3 {
pre_entry:
  %0 = tail call i64 @llvm.smax.i64(i64 %b, i64 %a)
  %v10_04 = icmp eq i64 %0, 0
  br i1 %v10_04, label %common.ret.split.loop.exit2, label %else.7

common.ret.split.loop.exit2:                      ; preds = %else.12, %pre_entry
  %a.tr.lcssa = phi i64 [ %a, %pre_entry ], [ %b.tr6, %else.12 ]
  %b.tr.lcssa = phi i64 [ %b, %pre_entry ], [ %2, %else.12 ]
  %1 = tail call i64 @llvm.smin.i64(i64 %b.tr.lcssa, i64 %a.tr.lcssa)
  br label %common.ret

common.ret:                                       ; preds = %else.7, %common.ret.split.loop.exit2
  %common.ret.op = phi i64 [ %1, %common.ret.split.loop.exit2 ], [ 0, %else.7 ]
  ret i64 %common.ret.op

else.7:                                           ; preds = %pre_entry, %else.12
  %b.tr6 = phi i64 [ %2, %else.12 ], [ %b, %pre_entry ]
  %a.tr5 = phi i64 [ %b.tr6, %else.12 ], [ %a, %pre_entry ]
  %v15 = icmp eq i64 %b.tr6, 0
  br i1 %v15, label %common.ret, label %else.12

else.12:                                          ; preds = %else.7
  %2 = srem i64 %a.tr5, %b.tr6
  %3 = tail call i64 @llvm.smax.i64(i64 %2, i64 %b.tr6)
  %v10_0 = icmp eq i64 %3, 0
  br i1 %v10_0, label %common.ret.split.loop.exit2, label %else.7
}

; Function Attrs: nofree nounwind
define dso_local void @__relative_primes(i64 %a) local_unnamed_addr #0 {
pre_entry:
  %v4_04 = icmp sgt i64 %a, 0
  br i1 %v4_04, label %else.7.i.preheader, label %for.end.0

else.7.i.preheader:                               ; preds = %pre_entry, %else.7
  %b_15 = phi i64 [ %v15_0, %else.7 ], [ %a, %pre_entry ]
  br label %else.7.i

else.7.i:                                         ; preds = %else.7.i.preheader, %else.12.i
  %b.tr6.i = phi i64 [ %0, %else.12.i ], [ %b_15, %else.7.i.preheader ]
  %a.tr5.i = phi i64 [ %b.tr6.i, %else.12.i ], [ %a, %else.7.i.preheader ]
  %v15.i = icmp eq i64 %b.tr6.i, 0
  br i1 %v15.i, label %else.7, label %else.12.i

else.12.i:                                        ; preds = %else.7.i
  %0 = srem i64 %a.tr5.i, %b.tr6.i
  %1 = tail call i64 @llvm.smax.i64(i64 %0, i64 %b.tr6.i) #6
  %v10_0.i = icmp eq i64 %1, 0
  br i1 %v10_0.i, label %__gcd.exit, label %else.7.i

__gcd.exit:                                       ; preds = %else.12.i
  %2 = tail call i64 @llvm.smin.i64(i64 %0, i64 %b.tr6.i) #6
  %v10_0 = icmp eq i64 %2, 1
  br i1 %v10_0, label %then.7, label %else.7

then.7:                                           ; preds = %__gcd.exit
  %3 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %b_15) #6
  %4 = tail call i32 @putchar(i32 10) #6
  br label %else.7

else.7:                                           ; preds = %else.7.i, %then.7, %__gcd.exit
  %v15_0 = add nsw i64 %b_15, -1
  %v4_0 = icmp sgt i64 %b_15, 1
  br i1 %v4_0, label %else.7.i.preheader, label %for.end.0

for.end.0:                                        ; preds = %else.7, %pre_entry
  ret void
}

define dso_local i32 @main(i32 %argc, i8** nocapture readnone %argv) local_unnamed_addr {
  %1 = add nsw i32 %argc, -1
  %.not = icmp eq i32 %1, 0
  br i1 %.not, label %else.7.i.preheader.i.i.preheader.i, label %codeRepl

codeRepl:                                         ; preds = %0
  call void @main.cold.1(i32 %1) #7
  ret i32 0

else.7.i.preheader.i.i.preheader.i:               ; preds = %0, %__orig_main.exit.i
  %loop_counter_12.i = phi i64 [ %loop_counter_2.i, %__orig_main.exit.i ], [ 10, %0 ]
  br label %else.7.i.preheader.i.i.i

else.7.i.preheader.i.i.i:                         ; preds = %else.7.i.i.i, %else.7.i.preheader.i.i.preheader.i
  %b_15.i.i.i = phi i64 [ %v15_0.i.i.i, %else.7.i.i.i ], [ %loop_counter_12.i, %else.7.i.preheader.i.i.preheader.i ]
  br label %else.7.i.i.i.i

else.7.i.i.i.i:                                   ; preds = %else.12.i.i.i.i, %else.7.i.preheader.i.i.i
  %b.tr6.i.i.i.i = phi i64 [ %2, %else.12.i.i.i.i ], [ %b_15.i.i.i, %else.7.i.preheader.i.i.i ]
  %a.tr5.i.i.i.i = phi i64 [ %b.tr6.i.i.i.i, %else.12.i.i.i.i ], [ %loop_counter_12.i, %else.7.i.preheader.i.i.i ]
  %v15.i.i.i.i = icmp eq i64 %b.tr6.i.i.i.i, 0
  br i1 %v15.i.i.i.i, label %else.7.i.i.i, label %else.12.i.i.i.i

else.12.i.i.i.i:                                  ; preds = %else.7.i.i.i.i
  %2 = srem i64 %a.tr5.i.i.i.i, %b.tr6.i.i.i.i
  %3 = tail call i64 @llvm.smax.i64(i64 %2, i64 %b.tr6.i.i.i.i) #6
  %v10_0.i.i.i.i = icmp eq i64 %3, 0
  br i1 %v10_0.i.i.i.i, label %__gcd.exit.i.i.i, label %else.7.i.i.i.i

__gcd.exit.i.i.i:                                 ; preds = %else.12.i.i.i.i
  %4 = tail call i64 @llvm.smin.i64(i64 %2, i64 %b.tr6.i.i.i.i) #6
  %v10_0.i.i.i = icmp eq i64 %4, 1
  br i1 %v10_0.i.i.i, label %then.7.i.i.i, label %else.7.i.i.i

then.7.i.i.i:                                     ; preds = %__gcd.exit.i.i.i
  %5 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([4 x i8], [4 x i8]* @.str.2, i64 0, i64 0), i64 %b_15.i.i.i) #6
  %6 = tail call i32 @putchar(i32 10) #6
  br label %else.7.i.i.i

else.7.i.i.i:                                     ; preds = %else.7.i.i.i.i, %then.7.i.i.i, %__gcd.exit.i.i.i
  %v15_0.i.i.i = add nsw i64 %b_15.i.i.i, -1
  %v4_0.i.i.i = icmp sgt i64 %b_15.i.i.i, 1
  br i1 %v4_0.i.i.i, label %else.7.i.preheader.i.i.i, label %__orig_main.exit.i

__orig_main.exit.i:                               ; preds = %else.7.i.i.i
  %loop_counter_2.i = add nuw nsw i64 %loop_counter_12.i, 1
  %exitcond.not.i = icmp eq i64 %loop_counter_2.i, 3000
  br i1 %exitcond.not.i, label %__main.exit, label %else.7.i.preheader.i.i.preheader.i

__main.exit:                                      ; preds = %__orig_main.exit.i
  ret i32 0
}

; Function Attrs: nocallback nofree nosync nounwind readnone speculatable willreturn
declare i64 @llvm.smin.i64(i64, i64) #4

; Function Attrs: nocallback nofree nosync nounwind readnone speculatable willreturn
declare i64 @llvm.smax.i64(i64, i64) #4

; Function Attrs: cold minsize noreturn
define internal void @main.cold.1(i32 %0) #5 {
newFuncRoot:
  br label %1

1:                                                ; preds = %newFuncRoot
  %2 = tail call i32 (i8*, ...) @printf(i8* nonnull dereferenceable(1) getelementptr inbounds ([33 x i8], [33 x i8]* @.str.4, i64 0, i64 0), i32 0, i32 %0)
  tail call void @exit(i32 2)
  unreachable
}

attributes #0 = { nofree nounwind }
attributes #1 = { argmemonly mustprogress nofree norecurse nosync nounwind readonly willreturn }
attributes #2 = { mustprogress nofree norecurse nosync nounwind readnone willreturn }
attributes #3 = { nofree nosync nounwind readnone }
attributes #4 = { nocallback nofree nosync nounwind readnone speculatable willreturn }
attributes #5 = { cold minsize noreturn }
attributes #6 = { nounwind }
attributes #7 = { noinline }
