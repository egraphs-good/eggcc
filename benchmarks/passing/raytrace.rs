// ARGS: 0.0 0.0 0.0 50.0 50.0

// builds a cube mesh with two sides
// z is positive going up, x goes positive to the right
// y is positive going into the screen
// then builds a bvh tree from the cube mesh
fn main(xpos: f64, ypos: f64, zpos: f64, width: f64, height: f64) {
    let num_triangles: i64 = 100;
    let mut first_point: [f64; 3] = [0.0; 3];
    // 100 triangles, 3 points per triangle
    let mut triangles: [[f64; 3]; 300] = [first_point; 300];

    let mut ind: i64 = 1;
    while ind < 300 {
        let mut next_point: [f64; 3] = [0.0; 3];
        triangles[ind] = next_point;
        ind = ind + 1;
    }

    //build_cube(xpos, ypos, width, height, triangles);

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

    let mut i: i64 = 0;
    let mut ti: i64 = 0;
    let cx: f64 = xpos + width / 2.0;
    while i < num_rectangles {
        triangles[ti][0] = cx;
        triangles[ti][1] = ypos;
        triangles[ti][2] = 0.0;

        i = i + 1;
    }
}
