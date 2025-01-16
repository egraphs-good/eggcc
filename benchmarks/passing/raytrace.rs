// ARGS: -0.2 2.0 -0.2 0.4 0.4

// builds a cube mesh with two sides
// z is positive going up, x goes positive to the right
// y is positive going into the screen
// then builds a bvh tree from the cube mesh
fn main(xpos: f64, ypos: f64, zpos: f64, width: f64, height: f64) {
    let num_triangles: i64 = 100;
    let mut first_point: [f64; 3] = [0.0; 3];
    let mut first_triangle: [[f64; 3]; 3] = [first_point; 3];
    // 100 triangles, 3 points per triangle
    // hack: need a single point to initialize the array
    let mut triangles: [[[f64; 3]; 3]; 100] = [first_triangle; 100];

    // allocate triangles
    let mut ind: i64 = 0;
    while ind < 100 {
        let point1: [f64; 3] = [0.0; 3];
        let point2: [f64; 3] = [0.0; 3];
        let point3: [f64; 3] = [0.0; 3];
        let triangle: [[f64; 3]; 3] = [point1, point2, point3];
        triangles[ind as usize] = triangle;
        ind = ind + 1;
    }

    // drop the first triangle
    drop(first_point);
    drop(first_triangle);

    build_cube(xpos, ypos, zpos, width, height, &mut triangles);

    // For each bvh, store the indices of the two children
    // Use -1 for no child
    let first_child: [i64; 2] = [-1, -1];
    let mut bvh_children: [[i64; 2]; 100] = [first_child; 100];
    let dummy_point: [f64; 3] = [0.0; 3];
    let dummy_bbox: [[f64; 3]; 2] = [dummy_point, dummy_point];
    // For each bvh, store the start position
    let mut bvh_bbox: [[[f64; 3]; 2]; 100] = [dummy_bbox; 100];
    // for each bvh, store the start index into triangles
    let mut bvh_start: [i64; 100] = [0; 100];
    // for each bvh, store the size of the interval into triangles
    let mut bvh_size: [i64; 100] = [0; 100];

    // initialize bvh_children
    let mut ind: i64 = 1;
    while ind < 100 {
        let mut next_children: [i64; 2] = [-1, -1];
        bvh_children[ind as usize] = next_children;
        ind = ind + 1;
    }

    // initialize bvh_bbox
    let mut ind: i64 = 0;
    while ind < 100 {
        let start_point: [f64; 3] = [0.0; 3];
        let end_point: [f64; 3] = [0.0; 3];
        let next_bbox: [[f64; 3]; 2] = [start_point, end_point];
        bvh_bbox[ind as usize] = next_bbox;
        ind = ind + 1;
    }

    // build the bvh
    build_bvh(
        &mut triangles,
        &mut bvh_children,
        &mut bvh_bbox,
        &mut bvh_start,
        &mut bvh_size,
    );

    // define the output screen
    let dummy_row: [f64; 100] = [0.0; 100];
    let mut output: [[f64; 100]; 100] = [dummy_row; 100];
    drop(dummy_row);
    let mut i: i64 = 0;
    while i < 100 {
        let row: [f64; 100] = [0.0; 100];
        output[i as usize] = row;
        i = i + 1;
    }

    let light_pos: [f64; 3] = [0.0, 0.5, 2.0];

    // trace one ray per screen pixel
    let mut row2: i64 = 0;
    let mut pz: f64 = 0.0;
    while row2 < 100 {
        let mut col2: i64 = 0;
        let mut px: f64 = 0.0;
        while col2 < 100 {
            let myray: [[f64; 3]; 2] = sample_ray(px, pz);
            let mut res_ray: [[f64; 3]; 2] = [[0.0; 3], [0.0; 3]];
            let did_intersect: bool = trace(
                &triangles,
                &myray,
                &mut res_ray,
                &bvh_children,
                &bvh_bbox,
                &bvh_start,
                &bvh_size,
            );
            if did_intersect {
                let point_to_light: [f64; 3] = vec_sub(light_pos, res_ray[0]);
                let point_to_light_norm: [f64; 3] = vec_normalize(point_to_light);
                let light_dist: f64 = vec_len(point_to_light);

                let brightness: f64 = 1.0 / (light_dist * light_dist);
                let light_dot: f64 = dot(point_to_light_norm, res_ray[1]);
                let abs_light_dot: f64 = fabs(light_dot);

                output[row2 as usize][col2 as usize] = brightness * abs_light_dot;
                drop(point_to_light);
                drop(point_to_light_norm);
            }

            col2 = col2 + 1;
            px = px + 0.01;

            drop(myray[0]);
            drop(myray[1]);
            drop(myray);
            drop(res_ray[0]);
            drop(res_ray[1]);
            drop(res_ray);
        }
        row2 = row2 + 1;
        pz = pz + 0.01;
    }

    drop(light_pos);

    // the test version of the benchmark prints out the screen here
    // (aka raytracewithscreen.rs)
    //let mut rowi: i64 = 0;
    //while rowi < 100 {
    //let mut col: i64 = 0;
    //while col < 100 {
    //  let to_print: f64 = output[rowi as usize][col as usize];
    //println!("{}", to_print);
    //col = col + 1;
    //}
    //rowi = rowi + 1;
    //}

    let res: f64 = triangles[10][0][0];

    // drop screen
    let mut i: i64 = 0;
    while i < 100 {
        drop(output[i as usize]);
        i = i + 1;
    }
    drop(output);

    // drop dummy point and dummy bbox
    drop(dummy_point);
    drop(dummy_bbox);

    // drop the triangles
    let mut ind: i64 = 0;
    while ind < 100 {
        let mut point: i64 = 0;
        while point < 3 {
            drop(triangles[ind as usize][point as usize]);
            point = point + 1;
        }
        drop(triangles[ind as usize]);
        ind = ind + 1;
    }
    drop(triangles);

    // drop bvh_children, bvh_bbox
    let mut ind: i64 = 0;
    while ind < 100 {
        drop(bvh_children[ind as usize]);
        drop(bvh_bbox[ind as usize][0]);
        drop(bvh_bbox[ind as usize][1]);
        drop(bvh_bbox[ind as usize]);
        ind = ind + 1;
    }
    drop(bvh_children);
    drop(bvh_bbox);
    drop(bvh_start);
    drop(bvh_size);
    println!("{}", res);
}

fn vec_sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    return [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
}

fn vec_add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    return [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
}

fn vec_scale(a: [f64; 3], s: f64) -> [f64; 3] {
    return [a[0] * s, a[1] * s, a[2] * s];
}

fn fabs(x: f64) -> f64 {
    let mut res: f64 = x;
    if x < 0.0 {
        res = -res;
    }
    return res;
}

fn sqrt(x: f64) -> f64 {
    let mut guess: f64 = x;
    let tolerance: f64 = 1e-2; // Set precision level
    let mut num_iters: i64 = 100;

    while (num_iters > 0 && fabs(guess * guess - x) > tolerance) {
        guess = (guess + x / guess) / 2.0;
        num_iters = num_iters - 1;
    }

    return guess;
}

fn vec_len(a: [f64; 3]) -> f64 {
    return sqrt(a[0] * a[0] + a[1] * a[1] + a[2] * a[2]);
}

fn vec_normalize(a: [f64; 3]) -> [f64; 3] {
    let len: f64 = vec_len(a);
    return [a[0] / len, a[1] / len, a[2] / len];
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

// Constructs BVH tree, re-arranging the order of triangles
// so that the BVH can refer to intervals of in the triangles array.
fn build_bvh(
    triangles: &mut [[[f64; 3]; 3]; 100],
    bvh_children: &mut [[i64; 2]; 100],
    bvh_bbox: &mut [[[f64; 3]; 2]; 100],
    bvh_start: &mut [i64; 100],
    bvh_size: &mut [i64; 100],
) {
    make_leaf_node(
        0,
        100,
        triangles,
        bvh_children,
        bvh_bbox,
        bvh_start,
        bvh_size,
        0,
    );
    let mut bvh_next_free: i64 = 1;
    let mut node_stack: [i64; 100] = [0; 100];
    let mut stack_size: i64 = 1;

    let mut directions: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    let num_partitions_try: i64 = 32;

    while stack_size > 0 {
        let node: i64 = node_stack[stack_size as usize - 1];
        stack_size = stack_size - 1;

        let mut best_partition: [f64; 3] = [0.0; 3];
        let large_num: f64 = 9999999999999999.0;
        let mut best_cost: f64 = large_num;
        let mut direction_index: i64 = 0;
        while direction_index < 3 {
            let mut partition_i: i64 = 0;
            let mut partition_i_float: f64 = 0.0;
            while partition_i < num_partitions_try {
                let scaled_max: [f64; 3] = vec_scale(bvh_bbox[node as usize][1], 1.0 / 32.0);
                let scaled_min: [f64; 3] = vec_scale(bvh_bbox[node as usize][0], 1.0 / 32.0);
                let subtracted: [f64; 3] = vec_sub(scaled_max, scaled_min);
                let dist: f64 =
                    dot(subtracted, directions[direction_index as usize]) * partition_i_float;
                drop(scaled_max);
                drop(scaled_min);
                drop(subtracted);

                let partition: [f64; 3] = vec_scale(directions[direction_index as usize], dist);
                let cost: f64 = partition_cost(
                    node,
                    triangles,
                    bvh_children,
                    bvh_bbox,
                    bvh_start,
                    bvh_size,
                    &partition,
                );

                if cost < best_cost {
                    drop(best_partition);
                    best_cost = cost;
                    best_partition = partition;
                } else {
                    drop(partition);
                }

                partition_i = partition_i + 1;
                partition_i_float = partition_i_float + 1.0;
            }
            direction_index = direction_index + 1;
        }

        let middle: i64 = partition(
            node,
            triangles,
            bvh_children,
            bvh_bbox,
            bvh_start,
            bvh_size,
            &best_partition,
        );
        let left_child: i64 = bvh_next_free;
        make_leaf_node(
            bvh_start[node as usize],
            middle,
            triangles,
            bvh_children,
            bvh_bbox,
            bvh_start,
            bvh_size,
            bvh_next_free,
        );
        bvh_next_free = bvh_next_free + 1;
        let right_child: i64 = bvh_next_free;
        make_leaf_node(
            middle,
            bvh_start[node as usize] + bvh_size[node as usize],
            triangles,
            bvh_children,
            bvh_bbox,
            bvh_start,
            bvh_size,
            bvh_next_free,
        );
        bvh_next_free = bvh_next_free + 1;

        bvh_children[node as usize][0] = left_child;
        bvh_children[node as usize][1] = right_child;

        if bvh_size[left_child as usize] > 4 {
            node_stack[stack_size as usize] = left_child;
            stack_size = stack_size + 1;
        }

        if bvh_size[right_child as usize] > 4 {
            node_stack[stack_size as usize] = right_child;
            stack_size = stack_size + 1;
        }

        drop(best_partition);
    }

    drop(node_stack);
    drop(directions[0]);
    drop(directions[1]);
    drop(directions[2]);
    drop(directions);
}

// currently returns an end index somewhere in the middle
fn partition(
    node: i64,
    triangles: &mut [[[f64; 3]; 3]; 100],
    bvh_children: &mut [[i64; 2]; 100],
    bvh_bbox: &mut [[[f64; 3]; 2]; 100],
    bvh_start: &mut [i64; 100],
    bvh_size: &mut [i64; 100],
    partition_vec: &[f64; 3],
) -> i64 {
    if bvh_size[node as usize] == 1 {
        return bvh_start[node as usize] + 1;
    }
    return bvh_start[node as usize] + bvh_size[node as usize] / 2;
}

fn bbox_size(bbox: [[f64; 3]; 2]) -> f64 {
    let diff: [f64; 3] = vec_sub(bbox[1], bbox[0]);
    let size: f64 = vec_len(diff);
    drop(diff);
    return size;
}

fn partition_cost(
    node: i64,
    triangles: &mut [[[f64; 3]; 3]; 100],
    bvh_children: &mut [[i64; 2]; 100],
    bvh_bbox: &mut [[[f64; 3]; 2]; 100],
    bvh_start: &mut [i64; 100],
    bvh_size: &mut [i64; 100],
    partition_vec: &[f64; 3],
) -> f64 {
    let end: i64 = partition(
        node,
        triangles,
        bvh_children,
        bvh_bbox,
        bvh_start,
        bvh_size,
        partition_vec,
    );

    let bbox_left: [[f64; 3]; 2] = bbox_between(bvh_start[node as usize], end, triangles);
    let bbox_right: [[f64; 3]; 2] = bbox_between(
        end,
        bvh_start[node as usize] + bvh_size[node as usize],
        triangles,
    );
    let bbox_left_size: f64 = bbox_size(bbox_left);
    let bbox_right_size: f64 = bbox_size(bbox_right);
    let cost: f64 = 1.0 / (bbox_left_size * bbox_right_size);
    drop(bbox_left[0]);
    drop(bbox_left[1]);
    drop(bbox_left);
    drop(bbox_right[0]);
    drop(bbox_right[1]);
    drop(bbox_right);
    return cost;
}

fn fmin(a: f64, b: f64) -> f64 {
    let mut res: f64 = a;
    if b < a {
        res = b;
    }
    return res;
}

fn fmax(a: f64, b: f64) -> f64 {
    let mut res: f64 = a;
    if b > a {
        res = b;
    }
    return res;
}

fn point_max(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    return [fmax(a[0], b[0]), fmax(a[1], b[1]), fmax(a[2], b[2])];
}

fn point_min(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    return [fmin(a[0], b[0]), fmin(a[1], b[1]), fmin(a[2], b[2])];
}

fn triangle_min(tri: [[f64; 3]; 3]) -> [f64; 3] {
    let point1: [f64; 3] = tri[0];
    let point2: [f64; 3] = tri[1];
    let point3: [f64; 3] = tri[2];
    let min1: [f64; 3] = point_min(point1, point2);
    let min2: [f64; 3] = point_min(min1, point3);
    drop(min1);
    return min2;
}

fn triangle_max(tri: [[f64; 3]; 3]) -> [f64; 3] {
    let point1: [f64; 3] = tri[0];
    let point2: [f64; 3] = tri[1];
    let point3: [f64; 3] = tri[2];
    let max1: [f64; 3] = point_max(point1, point2);
    let max2: [f64; 3] = point_max(max1, point3);
    drop(max1);
    return max2;
}

fn bbox_triangle(tri: [[f64; 3]; 3]) -> [[f64; 3]; 2] {
    let min: [f64; 3] = triangle_min(tri);
    let max: [f64; 3] = triangle_max(tri);
    let bbox: [[f64; 3]; 2] = [min, max];
    return bbox;
}

fn bbox_union(a: [[f64; 3]; 2], b: [[f64; 3]; 2]) -> [[f64; 3]; 2] {
    let min: [f64; 3] = point_min(a[0], b[0]);
    let max: [f64; 3] = point_max(a[1], b[1]);
    let bbox: [[f64; 3]; 2] = [min, max];
    return bbox;
}

fn bbox_between(start: i64, end: i64, triangles: &mut [[[f64; 3]; 3]; 100]) -> [[f64; 3]; 2] {
    let mut i: i64 = start;
    let mut bbox: [[f64; 3]; 2] = bbox_triangle(triangles[i as usize]);
    while i < end {
        let next_bbox: [[f64; 3]; 2] = bbox_triangle(triangles[i as usize]);
        let bbox_old: [[f64; 3]; 2] = bbox;
        bbox = bbox_union(bbox, next_bbox);
        drop(bbox_old[0]);
        drop(bbox_old[1]);
        drop(bbox_old);
        drop(next_bbox[0]);
        drop(next_bbox[1]);
        drop(next_bbox);

        i = i + 1;
    }

    return bbox;
}

// start inclusive, end exclusive
fn make_leaf_node(
    start: i64,
    end: i64,
    triangles: &mut [[[f64; 3]; 3]; 100],
    bvh_children: &mut [[i64; 2]; 100],
    bvh_bbox: &mut [[[f64; 3]; 2]; 100],
    bvh_start: &mut [i64; 100],
    bvh_size: &mut [i64; 100],
    node_id: i64,
) {
    bvh_start[node_id as usize] = start;
    bvh_size[node_id as usize] = end - start;

    bvh_children[node_id as usize][0] = node_id;
    bvh_children[node_id as usize][1] = node_id;

    let bbox: [[f64; 3]; 2] = bbox_between(start, end, triangles);

    drop(bvh_bbox[node_id as usize][0]);
    drop(bvh_bbox[node_id as usize][1]);
    drop(bvh_bbox[node_id as usize]);
    bvh_bbox[node_id as usize] = bbox;
}

// builds a cube with a bottom, front, left corner at (xpos, ypos, zpos)
fn build_cube(
    xpos: f64,
    ypos: f64,
    zpos: f64,
    width: f64,
    height: f64,
    mut triangles: &mut [[[f64; 3]; 3]; 100],
) {
    let num_rectangles: i64 = 25;
    let rect_width: f64 = width / 25.0;

    // build the front face, cc-wise triangles
    let mut i: i64 = 0;
    let mut ti: i64 = 0;
    let mut cx: f64 = xpos;
    while i < num_rectangles {
        // top left corner
        triangles[ti as usize][0][0] = cx;
        triangles[ti as usize][0][1] = ypos;
        triangles[ti as usize][0][2] = zpos;

        // bottom left corner
        triangles[ti as usize][1][0] = cx;
        triangles[ti as usize][1][1] = ypos;
        triangles[ti as usize][1][2] = zpos + height;

        // top right corner
        triangles[ti as usize][2][0] = cx + rect_width;
        triangles[ti as usize][2][1] = ypos;
        triangles[ti as usize][2][2] = zpos;

        ti = ti + 1;
        // top right corner
        triangles[ti as usize][0][0] = cx + rect_width;
        triangles[ti as usize][0][1] = ypos;
        triangles[ti as usize][0][2] = zpos;

        // bottom left corner
        triangles[ti as usize][1][0] = cx;
        triangles[ti as usize][1][1] = ypos;
        triangles[ti as usize][1][2] = zpos + height;

        // bottom right corner
        triangles[ti as usize][2][0] = cx + rect_width;
        triangles[ti as usize][2][1] = ypos;
        triangles[ti as usize][2][2] = zpos + height;

        ti = ti + 1;
        cx = cx + rect_width;
        i = i + 1;
    }

    // build the top face
    let mut i: i64 = 0;
    let mut cx: f64 = xpos;
    while i < num_rectangles {
        // front left corner
        triangles[ti as usize][0][0] = cx;
        triangles[ti as usize][0][1] = ypos;
        triangles[ti as usize][0][2] = zpos + height;

        // back right corner
        triangles[ti as usize][1][0] = cx + rect_width;
        triangles[ti as usize][1][1] = ypos + height;
        triangles[ti as usize][1][2] = zpos + height;

        // back left corner
        triangles[ti as usize][2][0] = cx;
        triangles[ti as usize][2][1] = ypos + height;
        triangles[ti as usize][2][2] = zpos + height;

        ti = ti + 1;
        // front left corner
        triangles[ti as usize][0][0] = cx;
        triangles[ti as usize][0][1] = ypos;
        triangles[ti as usize][0][2] = zpos + height;

        // front right corner
        triangles[ti as usize][1][0] = cx + rect_width;
        triangles[ti as usize][1][1] = ypos;
        triangles[ti as usize][1][2] = zpos + height;

        // back right corner
        triangles[ti as usize][2][0] = cx + rect_width;
        triangles[ti as usize][2][1] = ypos + height;
        triangles[ti as usize][2][2] = zpos + height;

        ti = ti + 1;
        cx = cx + rect_width;
        i = i + 1;
    }
}

// sample a ray from the camera pixel at (px, pz)
// px and py are in the range [0, 1]
fn sample_ray(px: f64, pz: f64) -> [[f64; 3]; 2] {
    let camera_pos: [f64; 3] = [0.08, 1.7, 0.8];
    let camera_forward: [f64; 3] = [0.0, 1.0, -1.5];
    let camera_forward_norm: [f64; 3] = vec_normalize(camera_forward);

    // calculate light up vector by rotating light forward vector 90 degrees around x axis, where positive z is up
    let camera_up: [f64; 3] = [
        camera_forward_norm[0],
        -camera_forward_norm[2],
        camera_forward_norm[1],
    ];

    let camera_right: [f64; 3] = vec_cross(camera_up, camera_forward_norm);

    // now sample a camera ray
    // camera screen is a 0.2 x 0.2 square 0.2 units in front of the light
    let camera_forward_scaled: [f64; 3] = vec_scale(camera_forward_norm, 0.2);
    let camera_center_pos: [f64; 3] = vec_add(camera_pos, camera_forward_scaled);
    let camera_up_scaled: [f64; 3] = vec_scale(camera_up, 0.1);
    let camera_top_middle_pos: [f64; 3] = vec_add(camera_center_pos, camera_up_scaled);
    let camera_right_scaled: [f64; 3] = vec_scale(camera_right, 0.1);
    let camera_top_left_pos: [f64; 3] = vec_sub(camera_top_middle_pos, camera_right_scaled);
    let camera_top_right_pos: [f64; 3] = vec_add(camera_top_middle_pos, camera_right_scaled);
    let camera_bottom_middle_pos: [f64; 3] = vec_sub(camera_center_pos, camera_up_scaled);
    let camera_bottom_left_pos: [f64; 3] = vec_sub(camera_bottom_middle_pos, camera_right_scaled);

    let camera_screen_across: [f64; 3] = vec_sub(camera_top_right_pos, camera_top_left_pos);
    let camera_screen_down: [f64; 3] = vec_sub(camera_bottom_left_pos, camera_top_left_pos);

    let across_scaled: [f64; 3] = vec_scale(camera_screen_across, px);
    let mut ray_end: [f64; 3] = vec_add(camera_top_left_pos, across_scaled);
    let down_scaled: [f64; 3] = vec_scale(camera_screen_down, pz);
    ray_end = vec_add(ray_end, down_scaled);

    let diff: [f64; 3] = vec_sub(ray_end, camera_pos);
    let ray_dir: [f64; 3] = vec_normalize(diff);
    let ray: [[f64; 3]; 2] = [camera_pos, ray_dir];

    drop(camera_forward);
    drop(camera_forward_norm);
    drop(camera_up);
    drop(camera_right);
    drop(camera_forward_scaled);
    drop(camera_center_pos);
    drop(camera_up_scaled);
    drop(camera_top_middle_pos);
    drop(camera_right_scaled);
    drop(camera_top_left_pos);
    drop(camera_top_right_pos);
    drop(camera_bottom_middle_pos);
    drop(camera_bottom_left_pos);
    drop(camera_screen_across);
    drop(camera_screen_down);
    drop(across_scaled);
    drop(ray_end);
    drop(down_scaled);
    drop(diff);

    return ray;
}

fn vec_cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    return [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
}

// intersect a ray with a triangle
// ray is a source point and normalized direction
// triangle is 3 points, cc-wise
// store the output intersection point in output
// returns a bool indicating if the intersection is valid
fn intersect(tri: &[[f64; 3]; 3], ray: &[[f64; 3]; 2], output: &mut [f64; 3]) -> bool {
    let epsilon: f64 = 0.0000001;
    let edge1: [f64; 3] = vec_sub(tri[1], tri[0]);
    let edge2: [f64; 3] = vec_sub(tri[2], tri[0]);
    let h: [f64; 3] = vec_cross(ray[1], edge2);
    let a: f64 = dot(edge1, h);
    if a > -epsilon && a < epsilon {
        return false;
    }
    let f: f64 = 1.0 / a;
    let s: [f64; 3] = vec_sub(ray[0], tri[0]);
    let u: f64 = f * dot(s, h);
    if u < 0.0 || u > 1.0 {
        return false;
    }
    let q: [f64; 3] = vec_cross(s, edge1);
    let v: f64 = f * dot(ray[1], q);
    if v < 0.0 || u + v > 1.0 {
        return false;
    }
    let t: f64 = f * dot(edge2, q);
    if t > epsilon {
        output[0] = ray[0][0] + t * ray[1][0];
        output[1] = ray[0][1] + t * ray[1][1];
        output[2] = ray[0][2] + t * ray[1][2];
        return true;
    }
    return false;
}

// for a given bounding box, checks if a ray intersects with it
// needs to do this without casting to a boolean
fn bbox_ray_intersect(bbox: [[f64; 3]; 2], ray: &[[f64; 3]; 2]) -> bool {
    let epsilon: f64 = 0.0000001;
    let t1: f64 = (bbox[0][0] - ray[0][0]) / ray[1][0];
    let t2: f64 = (bbox[1][0] - ray[0][0]) / ray[1][0];
    let tmin: f64 = fmin(t1, t2);
    let tmax: f64 = fmax(t1, t2);

    let t3: f64 = (bbox[0][1] - ray[0][1]) / ray[1][1];
    let t4: f64 = (bbox[1][1] - ray[0][1]) / ray[1][1];
    let tmin2: f64 = fmin(t3, t4);
    let tmax2: f64 = fmax(t3, t4);

    let t5: f64 = (bbox[0][2] - ray[0][2]) / ray[1][2];
    let t6: f64 = (bbox[1][2] - ray[0][2]) / ray[1][2];
    let tmin3: f64 = fmin(t5, t6);
    let tmax3: f64 = fmax(t5, t6);

    let tmin_final: f64 = fmax(tmin, fmax(tmin2, tmin3));
    let tmax_final: f64 = fmin(tmax, fmin(tmax2, tmax3));

    return tmax_final >= tmin_final && tmax_final > epsilon;
}

// Trace a ray and return the new ray in the output
// Return if the ray intersected with any triangle
// a ray is a source point and normalized direction
// Uses the bvh tree to speed up intersection
fn trace(
    triangles: &[[[f64; 3]; 3]; 100],
    ray: &[[f64; 3]; 2],
    output: &mut [[f64; 3]; 2],
    bvh_children: &[[i64; 2]; 100],
    bvh_bbox: &[[[f64; 3]; 2]; 100],
    bvh_start: &[i64; 100],
    bvh_size: &[i64; 100],
) -> bool {
    let mut min_dist: f64 = 9999999999999999.0;
    let mut bvh_stack: [i64; 100] = [0; 100];
    let mut stack_size: i64 = 1;

    let mut did_intersect: bool = false;
    while stack_size > 0 {
        let node_index: i64 = bvh_stack[(stack_size - 1) as usize];
        let bbox: [[f64; 3]; 2] = bvh_bbox[node_index as usize];
        stack_size = stack_size - 1;
        if bbox_ray_intersect(bbox, ray) {
            let child1: i64 = bvh_children[node_index as usize][0];
            let child2: i64 = bvh_children[node_index as usize][1];
            if !(child1 == node_index) {
                bvh_stack[stack_size as usize] = child1;
                stack_size = stack_size + 1;
                bvh_stack[stack_size as usize] = child2;
                stack_size = stack_size + 1;
            } else {
                let mut i: i64 = bvh_start[node_index as usize];
                while i < bvh_start[node_index as usize] + bvh_size[node_index as usize] {
                    let tri: [[f64; 3]; 3] = triangles[i as usize];
                    let mut point: [f64; 3] = [0.0; 3];
                    let res: bool = intersect(&tri, ray, &mut point);
                    if res {
                        let subtracted: [f64; 3] = vec_sub(point, ray[0]);
                        let dist: f64 = vec_len(subtracted);
                        if dist < min_dist {
                            min_dist = dist;
                            output[0][0] = point[0];
                            output[0][1] = point[1];
                            output[0][2] = point[2];

                            // new ray direction is the normal of the triangle
                            let edge1: [f64; 3] = vec_sub(tri[1], tri[0]);
                            let edge2: [f64; 3] = vec_sub(tri[2], tri[0]);
                            let normal: [f64; 3] = vec_cross(edge1, edge2);
                            let normalized_normal: [f64; 3] = vec_normalize(normal);
                            output[1][0] = normalized_normal[0];
                            output[1][1] = normalized_normal[1];
                            output[1][2] = normalized_normal[2];
                            drop(edge1);
                            drop(edge2);
                            drop(normal);
                            drop(normalized_normal);
                            did_intersect = true;
                        }
                        drop(subtracted);
                    }
                    drop(point);
                    i = i + 1;
                }
            }
        }
    }
    return did_intersect;
}
