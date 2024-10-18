// ARGS: 0.0 0.0 0.0 50.0 50.0

// builds a cube mesh
// then builds a bvh tree from the cube mesh
fn main(xpos: f64, ypos: f64, zpos: f64, width: f64, height: f64) {
    let num_triangles: i64 = 100;
    let triangles: [f64; 300] = [0.0; 300];
    build_cube(xpos, ypos, width, height, triangles);

    let res: f64 = triangles[0];
    drop(triangles);
    println!("{}", res);
}

fn build_cube(xpos: f64, ypos: f64, width: f64, height: f64, triangles: [f64; 300]) {
    let num_triangles: i64 = 100;
    let 
    triangles[0] = xpos;
    triangles[1] = ypos;
}
