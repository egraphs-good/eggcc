// ARGS: 0.0 0.0 0.0 50.0 50.0

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

    build_cube(xpos, ypos, width, height, triangles);

    // For each bvh, store the indices of the two children
    // Use -1 for no child
    let first_child: [i64; 2] = [-1, -1];
    let mut bvh_children: [[i64; 2]; 100] = [first_child; 100];
    let first_start: [f64; 3] = [0.0; 3];
    let first_end: [f64; 3] = [0.0; 3];
    // For each bvh, store the start position
    let mut bvh_start: [[f64; 3]; 100] = [first_start; 100];
    // For each bvh, store the end position
    let mut bvh_end: [[f64; 3]; 100] = [first_end; 100];

    // initialize bvh_children
    let mut ind: i64 = 1;
    while ind < 100 {
        let mut next_children: [i64; 2] = [-1, -1];
        bvh_children[ind as usize] = next_children;
        ind = ind + 1;
    }

    // initialize bvh_start
    let mut ind: i64 = 1;
    while ind < 100 {
        let mut next_start: [f64; 3] = [0.0; 3];
        bvh_start[ind as usize] = next_start;
        ind = ind + 1;
    }

    // initialize bvh_end
    let mut ind: i64 = 1;
    while ind < 100 {
        let mut next_end: [f64; 3] = [0.0; 3];
        bvh_end[ind as usize] = next_end;
        ind = ind + 1;
    }

    let res: f64 = triangles[10][0][0];

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

    // drop bvh_children, bvh_start, bvh_end
    let mut ind: i64 = 0;
    while ind < 100 {
        drop(bvh_children[ind as usize]);
        drop(bvh_start[ind as usize]);
        drop(bvh_end[ind as usize]);
        ind = ind + 1;
    }
    drop(bvh_children);
    drop(bvh_start);
    drop(bvh_end);
    println!("{}", res);
}

fn build_cube(xpos: f64, ypos: f64, width: f64, height: f64, mut triangles: [[[f64; 3]; 3]; 100]) {
    let num_rectangles: i64 = 25;
    let rect_width: f64 = width / 25.0;

    // build the front face, clockwise triangles
    let mut i: i64 = 0;
    let mut ti: i64 = 0;
    let mut cx: f64 = xpos + width / 2.0;
    while i < num_rectangles {
        // top left corner
        triangles[ti as usize][0][0] = cx;
        triangles[ti as usize][0][1] = ypos;
        triangles[ti as usize][0][2] = 0.0;

        // top right corner
        triangles[ti as usize][1][0] = cx + rect_width;
        triangles[ti as usize][1][1] = ypos;
        triangles[ti as usize][1][2] = 0.0;

        // bottom left corner
        triangles[ti as usize][2][0] = cx;
        triangles[ti as usize][2][1] = ypos;
        triangles[ti as usize][2][2] = -height;

        ti = ti + 1;
        // top right corner
        triangles[ti as usize][0][0] = cx + rect_width;
        triangles[ti as usize][0][1] = ypos;
        triangles[ti as usize][0][2] = 0.0;

        // bottom right corner
        triangles[ti as usize][1][0] = cx + rect_width;
        triangles[ti as usize][1][1] = ypos;
        triangles[ti as usize][1][2] = -height;

        // bottom left corner
        triangles[ti as usize][2][0] = cx;
        triangles[ti as usize][2][1] = ypos;
        triangles[ti as usize][2][2] = -height;

        ti = ti + 1;
        cx = cx + rect_width;
        i = i + 1;
    }

    // build the top face
    let mut i: i64 = 0;
    let mut cx: f64 = xpos + width / 2.0;
    while i < num_rectangles {
        // front left corner
        triangles[ti as usize][0][0] = cx;
        triangles[ti as usize][0][1] = ypos;
        triangles[ti as usize][0][2] = 0.0;

        // back left corner
        triangles[ti as usize][1][0] = cx;
        triangles[ti as usize][1][1] = ypos + height;
        triangles[ti as usize][1][2] = 0.0;

        // back right corner
        triangles[ti as usize][2][0] = cx + rect_width;
        triangles[ti as usize][2][1] = ypos + height;
        triangles[ti as usize][2][2] = 0.0;

        ti = ti + 1;
        // front left corner
        triangles[ti as usize][0][0] = cx;
        triangles[ti as usize][0][1] = ypos;
        triangles[ti as usize][0][2] = 0.0;

        // back right corner
        triangles[ti as usize][1][0] = cx + rect_width;
        triangles[ti as usize][1][1] = ypos + height;
        triangles[ti as usize][1][2] = 0.0;

        // front right corner
        triangles[ti as usize][2][0] = cx + rect_width;
        triangles[ti as usize][2][1] = ypos;
        triangles[ti as usize][2][2] = 0.0;

        ti = ti + 1;
        cx = cx + rect_width;
        i = i + 1;
    }
}
