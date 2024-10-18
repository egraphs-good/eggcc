// ARGS: 0.0 0.0 0.0 50.0 50.0

// builds a cube mesh with two sides
// z is positive going up, x goes positive to the right
// y is positive going into the screen
// then builds a bvh tree from the cube mesh
fn main(xpos: f64, ypos: f64, zpos: f64, width: f64, height: f64) {
    let num_triangles: i64 = 100;
    let mut first_point: [f64; 3] = [0.0; 3];
    // 100 triangles, 3 points per triangle
    // hack: need a single point to initialize the array
    let mut triangles: [[f64; 3]; 300] = [first_point; 300];

    // allocate the rest of the points
    let mut ind: i64 = 1;
    while ind < 300 {
        let mut next_point: [f64; 3] = [0.0; 3];
        triangles[ind] = next_point;
        ind = ind + 1;
    }

    build_cube(xpos, ypos, width, height, triangles);

    let res: f64 = triangles[10][0];
    let mut ind: i64 = 0;
    while ind < 300 {
        drop(triangles[ind]);
        ind = ind + 1;
    }
    drop(triangles);
    println!("{}", res);
}

fn build_cube(xpos: f64, ypos: f64, width: f64, height: f64, triangles: [[f64; 3]; 300]) {
    let num_triangles: i64 = 100;
    let num_rectangles: i64 = 25;
    let rect_width: f64 = width / 25.0;

    // build the front face, clockwise triangles
    let mut i: i64 = 0;
    let mut ti: i64 = 0;
    let mut cx: f64 = xpos + width / 2.0;
    while i < num_rectangles {
        // top left corner
        triangles[ti][0] = cx;
        triangles[ti][1] = ypos;
        triangles[ti][2] = 0.0;

        // top right corner
        triangles[ti + 1][0] = cx + rect_width;
        triangles[ti + 1][1] = ypos;
        triangles[ti + 1][2] = 0.0;

        // bottom left corner
        triangles[ti + 2][0] = cx;
        triangles[ti + 2][1] = ypos;
        triangles[ti + 2][2] = -height;

        ti = ti + 3;
        // top right corner
        triangles[ti][0] = cx + rect_width;
        triangles[ti][1] = ypos;
        triangles[ti][2] = 0.0;

        // bottom right corner
        triangles[ti + 1][0] = cx + rect_width;
        triangles[ti + 1][1] = ypos;
        triangles[ti + 1][2] = -height;

        // bottom left corner
        triangles[ti + 2][0] = cx;
        triangles[ti + 2][1] = ypos;
        triangles[ti + 2][2] = -height;

        ti = ti + 3;
        cx = cx + rect_width;
        i = i + 1;
    }

    // build the top face
    let mut i: i64 = 0;
    let mut cx: f64 = xpos + width / 2.0;
    while i < num_rectangles {
        // front left corner
        triangles[ti][0] = cx;
        triangles[ti][1] = ypos;
        triangles[ti][2] = 0.0;

        // back left corner
        triangles[ti + 1][0] = cx;
        triangles[ti + 1][1] = ypos + height;
        triangles[ti + 1][2] = 0.0;

        // back right corner
        triangles[ti + 2][0] = cx + rect_width;
        triangles[ti + 2][1] = ypos + height;
        triangles[ti + 2][2] = 0.0;

        ti = ti + 3;
        // front left corner
        triangles[ti][0] = cx;
        triangles[ti][1] = ypos;
        triangles[ti][2] = 0.0;

        // back right corner
        triangles[ti + 1][0] = cx + rect_width;
        triangles[ti + 1][1] = ypos + height;
        triangles[ti + 1][2] = 0.0;

        // front right corner
        triangles[ti + 2][0] = cx + rect_width;
        triangles[ti + 2][1] = ypos;
        triangles[ti + 2][2] = 0.0;

        ti = ti + 3;
        cx = cx + rect_width;
        i = i + 1;
    }

    //if !(ti == 300) {
    // this should never happen
    // TODO we can't put an error message here, it's ignored by the rs2bril compiler
    //println!("{}", ti);
    //}
}
