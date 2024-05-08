
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


define dso_local i64* @__pack(i64 %size, i64 %n1, i64 %n2, i64 %n3, i64 %n4, i64 %n5, i64 %n6) {
pre_entry:
  %z0 = mul i64 %size, 8
  %z1 = call i8* @malloc(i64 %z0)
  %array_0 = bitcast i8* %z1 to i64*
  %loc_0 = getelementptr inbounds i64, i64* %array_0, i64 0
  store i64 %n1, i64* %loc_0
  %i_1 = add i64 0, 1
  %loc_1 = getelementptr inbounds i64, i64* %array_0, i64 %i_1
  store i64 %n2, i64* %loc_1
  %i_2 = add i64 %i_1, 1
  %loc_2 = getelementptr inbounds i64, i64* %array_0, i64 %i_2
  store i64 %n3, i64* %loc_2
  %i_3 = add i64 %i_2, 1
  %loc_3 = getelementptr inbounds i64, i64* %array_0, i64 %i_3
  store i64 %n4, i64* %loc_3
  %i_4 = add i64 %i_3, 1
  %loc_4 = getelementptr inbounds i64, i64* %array_0, i64 %i_4
  store i64 %n5, i64* %loc_4
  %i_5 = add i64 %i_4, 1
  %loc_5 = getelementptr inbounds i64, i64* %array_0, i64 %i_5
  store i64 %n6, i64* %loc_5
  ret i64* %array_0

}


define dso_local void @__print_array(i64* %array, i64 %size) {
pre_entry:
  br label %loop
loop:
  %i_1 = phi i64 [ %i_2, %loop_end ], [ 0, %pre_entry ]
  %cond_0 = icmp slt i64 %i_1, %size
  br i1 %cond_0, label %body, label %done
body:
  %loc_0 = getelementptr inbounds i64, i64* %array, i64 %i_1
  %val_0 = load i64, i64* %loc_0
  call void @print_int(i64 %val_0)
  call void @print_newline()
  br label %loop_end
loop_end:
  %i_2 = add i64 %i_1, 1
  br label %loop
done:
  ret void

}


define dso_local i64 @__partition(i64* %array, i64 %l, i64 %r) {
pre_entry:
  %pivotloc_0 = getelementptr inbounds i64, i64* %array, i64 %r
  %pivot_0 = load i64, i64* %pivotloc_0
  br label %loop
loop:
  %j_1 = phi i64 [ %j_2, %loop_end ], [ %l, %pre_entry ]
  %i_1 = phi i64 [ %i_3, %loop_end ], [ %l, %pre_entry ]
  %cond_0 = icmp slt i64 %j_1, %r
  br i1 %cond_0, label %body, label %done
body:
  %curloc_0 = getelementptr inbounds i64, i64* %array, i64 %j_1
  %cur_0 = load i64, i64* %curloc_0
  %swap_0 = icmp sle i64 %cur_0, %pivot_0
  br i1 %swap_0, label %swap_j, label %loop_end
swap_j:
  %iloc_0 = getelementptr inbounds i64, i64* %array, i64 %i_1
  %ival_0 = load i64, i64* %iloc_0
  store i64 %ival_0, i64* %curloc_0
  store i64 %cur_0, i64* %iloc_0
  %i_2 = add i64 %i_1, 1
  br label %loop_end
loop_end:
  %i_3 = phi i64 [ %i_2, %swap_j ], [ %i_1, %body ]
  %j_2 = add i64 %j_1, 1
  br label %loop
done:
  %iloc_1 = getelementptr inbounds i64, i64* %array, i64 %i_3
  %ival_1 = load i64, i64* %iloc_1
  store i64 %pivot_0, i64* %iloc_1
  store i64 %ival_1, i64* %pivotloc_0
  ret i64 %i_3

}


define dso_local i64 @__quickselect(i64* %array, i64 %l, i64 %r, i64 %k) {
pre_entry:
  %index_0 = call i64 @__partition(i64* %array, i64 %l, i64 %r)
  %ipos_0 = sub i64 %index_0, %l
  %kpos_0 = sub i64 %k, 1
  %ieqk_0 = icmp eq i64 %ipos_0, %kpos_0
  br i1 %ieqk_0, label %found, label %not_found
found:
  %iloc_0 = getelementptr inbounds i64, i64* %array, i64 %index_0
  %i_0 = load i64, i64* %iloc_0
  ret i64 %i_0
not_found:
  %igtk_0 = icmp sgt i64 %ipos_0, %kpos_0
  br i1 %igtk_0, label %greater, label %less
greater:
  %newr_0 = sub i64 %index_0, 1
  %i_1 = call i64 @__quickselect(i64* %array, i64 %l, i64 %newr_0, i64 %k)
  ret i64 %i_1
less:
  %newl_0 = add i64 %index_0, 1
  %newk_0 = sub i64 %k, %index_0
  %newk_1 = add i64 %newk_0, %l
  %newk_2 = sub i64 %newk_1, 1
  %i_2 = call i64 @__quickselect(i64* %array, i64 %newl_0, i64 %r, i64 %newk_2)
  ret i64 %i_2

}


define dso_local void @__main() {
b0:
  br label %loop_cond
loop_cond:
  %loop_counter_1 = phi i64 [ %loop_counter_2, %loop_body ], [ 10, %b0 ]
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 1000000
  br i1 %loop_cond_0, label %loop_body, label %loop_done
loop_body:
  call void @__orig_main(i64 %loop_counter_1)
  %loop_counter_2 = add i64 %loop_counter_1, 1
  br label %loop_cond
loop_done:
  ret void

}


define dso_local void @__orig_main(i64 %x) {
pre_entry:
  %array_0 = call i64* @__pack(i64 6, i64 97, i64 108, i64 98, i64 101, i64 114, i64 116)
  %output_0 = call i64 @__quickselect(i64* %array_0, i64 0, i64 5, i64 4)
  %val_0 = add i64 %output_0, %x
  call void @print_int(i64 %val_0)
  call void @print_newline()
  %z0 = bitcast i64* %array_0 to i8*
  call void @free(i8* %z0)
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

