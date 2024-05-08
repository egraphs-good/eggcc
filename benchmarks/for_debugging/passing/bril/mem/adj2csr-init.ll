
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


define dso_local i64 @__rand(i64* %seq, i64 %max) {
pre_entry:
  %x_0 = load i64, i64* %seq
  %ax_0 = mul i64 25214903917, %x_0
  %axpc_0 = add i64 %ax_0, 11
  %next_0 = sdiv i64 %axpc_0, 281474976710656
  %next_1 = mul i64 %next_0, 281474976710656
  %next_2 = sub i64 %axpc_0, %next_1
  store i64 %next_2, i64* %seq
  %val_0 = sdiv i64 %next_2, %max
  %val_1 = mul i64 %val_0, %max
  %val_2 = sub i64 %next_2, %val_1
  ret i64 %val_2

}


define dso_local i64* @__randarray(i64 %size, i64* %rng) {
pre_entry:
  %z0 = mul i64 %size, 8
  %z1 = call i8* @malloc(i64 %z0)
  %arr_0 = bitcast i8* %z1 to i64*
  br label %loop
loop:
  %i_1 = phi i64 [ %i_2, %loop_end ], [ 0, %pre_entry ]
  %cond_0 = icmp slt i64 %i_1, %size
  br i1 %cond_0, label %body, label %done
body:
  %val_0 = call i64 @__rand(i64* %rng, i64 2)
  %if_cond_0 = icmp slt i64 %val_0, 0
  br i1 %if_cond_0, label %if_body, label %if_done
if_body:
  br label %if_done
if_done:
  %val_2 = phi i64 [ 0, %if_body ], [ %val_0, %body ]
  %loc_0 = getelementptr inbounds i64, i64* %arr_0, i64 %i_1
  store i64 %val_2, i64* %loc_0
  br label %loop_end
loop_end:
  %i_2 = add i64 %i_1, 1
  br label %loop
done:
  ret i64* %arr_0

}


define dso_local void @__printarray(i64 %size, i64* %arr) {
pre_entry:
  br label %loop
loop:
  %i_1 = phi i64 [ %i_2, %loop_end ], [ 0, %pre_entry ]
  %cond_0 = icmp slt i64 %i_1, %size
  br i1 %cond_0, label %body, label %done
body:
  %loc_0 = getelementptr inbounds i64, i64* %arr, i64 %i_1
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


define dso_local i64* @__zeroarray(i64 %size) {
pre_entry:
  %z0 = mul i64 %size, 8
  %z1 = call i8* @malloc(i64 %z0)
  %arr_0 = bitcast i8* %z1 to i64*
  br label %loop
loop:
  %i_1 = phi i64 [ %i_2, %loop_end ], [ 0, %pre_entry ]
  %cond_0 = icmp slt i64 %i_1, %size
  br i1 %cond_0, label %body, label %done
body:
  %loc_0 = getelementptr inbounds i64, i64* %arr_0, i64 %i_1
  store i64 0, i64* %loc_0
  br label %loop_end
loop_end:
  %i_2 = add i64 %i_1, 1
  br label %loop
done:
  ret i64* %arr_0

}


define dso_local i64 @__adj2csr(i64 %num_nodes, i64* %adjmat, i64* %csr_offset, i64* %csr_edges) {
pre_entry:
  br label %iter_row
iter_row:
  %row_1 = phi i64 [ %row_2, %row_end ], [ 0, %pre_entry ]
  %num_edges_1 = phi i64 [ %num_edges_4, %row_end ], [ 0, %pre_entry ]
  %row_cond_0 = icmp slt i64 %row_1, %num_nodes
  br i1 %row_cond_0, label %iter_col, label %row_done
iter_col:
  %col_1 = phi i64 [ %col_2, %col_end ], [ 0, %iter_row ]
  %num_edges_2 = phi i64 [ %num_edges_4, %col_end ], [ %num_edges_1, %iter_row ]
  %col_cond_0 = icmp slt i64 %col_1, %num_nodes
  br i1 %col_cond_0, label %col_body, label %col_done
col_body:
  %row_tmp_0 = mul i64 %row_1, %num_nodes
  %node_idx_0 = add i64 %row_tmp_0, %col_1
  %node_loc_0 = getelementptr inbounds i64, i64* %adjmat, i64 %node_idx_0
  %node_val_0 = load i64, i64* %node_loc_0
  %cond_0 = icmp eq i64 %node_val_0, 1
  br i1 %cond_0, label %if_body, label %col_end
if_body:
  %edge_loc_0 = getelementptr inbounds i64, i64* %csr_edges, i64 %num_edges_2
  store i64 %col_1, i64* %edge_loc_0
  %num_edges_3 = add i64 %num_edges_2, 1
  br label %col_end
col_end:
  %num_edges_4 = phi i64 [ %num_edges_3, %if_body ], [ %num_edges_2, %col_body ]
  %col_2 = add i64 %col_1, 1
  br label %iter_col
col_done:
  %offset_loc_0 = getelementptr inbounds i64, i64* %csr_offset, i64 %row_1
  store i64 %num_edges_4, i64* %offset_loc_0
  br label %row_end
row_end:
  %row_2 = add i64 %row_1, 1
  br label %iter_row
row_done:
  ret i64 %num_edges_4

}


define dso_local void @__main() {
b0:
  br label %loop_cond
loop_cond:
  %loop_counter_1 = phi i64 [ %loop_counter_2, %loop_body ], [ 10, %b0 ]
  %loop_cond_0 = icmp slt i64 %loop_counter_1, 150
  br i1 %loop_cond_0, label %loop_body, label %loop_done
loop_body:
  call void @__orig_main(i64 %loop_counter_1)
  %loop_counter_2 = add i64 %loop_counter_1, 1
  br label %loop_cond
loop_done:
  ret void

}


define dso_local void @__orig_main(i64 %num_nodes) {
pre_entry:
  %z0 = mul i64 1, 8
  %z1 = call i8* @malloc(i64 %z0)
  %rng_0 = bitcast i8* %z1 to i64*
  store i64 2348512, i64* %rng_0
  %sqsize_0 = mul i64 %num_nodes, %num_nodes
  %adjmat_0 = call i64* @__randarray(i64 %sqsize_0, i64* %rng_0)
  %csr_offset_0 = call i64* @__zeroarray(i64 %sqsize_0)
  %csr_edges_0 = call i64* @__zeroarray(i64 %sqsize_0)
  %num_edges_0 = call i64 @__adj2csr(i64 %num_nodes, i64* %adjmat_0, i64* %csr_offset_0, i64* %csr_edges_0)
  call void @print_int(i64 %num_nodes)
  call void @print_newline()
  call void @print_int(i64 %num_edges_0)
  call void @print_newline()
  call void @__printarray(i64 %sqsize_0, i64* %adjmat_0)
  call void @__printarray(i64 %num_nodes, i64* %csr_offset_0)
  call void @__printarray(i64 %num_edges_0, i64* %csr_edges_0)
  %z2 = bitcast i64* %adjmat_0 to i8*
  call void @free(i8* %z2)
  %z3 = bitcast i64* %csr_offset_0 to i8*
  call void @free(i8* %z3)
  %z4 = bitcast i64* %csr_edges_0 to i8*
  call void @free(i8* %z4)
  %z5 = bitcast i64* %rng_0 to i8*
  call void @free(i8* %z5)
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

