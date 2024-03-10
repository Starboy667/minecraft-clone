mod common;
mod math_func;
#[path = "surface_data.rs"]
mod sd;
use std::f32::consts::PI;

use common::{Mesh, Vertex};
use vertex_data::{create_cube, cube_colors, cube_normals, cube_positions};

// let mut function_selection = 0;
// let args: Vec<String> = std::env::args().collect();
// if args.len() > 1 {
//     function_selection = args[1].parse().unwrap();
// }

// let ps_struct: sd::ParametricSurface;

// if function_selection == 1 {
//     ps_struct = sd::ParametricSurface {
//         f: math_func::klein_bottle,
//         umin: 0.0,
//         umax: PI,
//         vmin: 0.0,
//         vmax: 2.0 * PI,
//         u_segments: 120,
//         v_segments: 40,
//         scale: 1.0,
//         ..Default::default()
//     };
// } else if function_selection == 2 {
//     ps_struct = sd::ParametricSurface {
//         f: math_func::wellenkugel,
//         umin: 0.0,
//         umax: 14.5,
//         vmin: 0.0,
//         vmax: 5.0,
//         u_segments: 100,
//         v_segments: 50,
//         scale: 0.17,
//         colormap_name: "cool",
//         ..Default::default()
//     };
// } else {
//     ps_struct = sd::ParametricSurface {
//         ..Default::default()
//     };
// }
mod camera;
mod texture;
mod vertex_data;

fn main() {
    // let (pos_data1, normal_data1, color_data1, index_data1) =
    //     sd::ParametricSurface::new(sd::ParametricSurface {
    //         f: math_func::klein_bottle,
    //         umin: 0.0,
    //         umax: PI,
    //         vmin: 0.0,
    //         vmax: 2.0 * PI,
    //         u_segments: 120,
    //         v_segments: 40,
    //         scale: 1.0,
    //         ..Default::default()
    //     });
    let light_data = common::light([1.0, 1.0, 1.0], 0.1, 0.8, 0.4, 30.0, 1);
    // change run to handle multiple vertex data
    // let a: Vec<(
    //     Option<Vec<[f32; 3]>>,
    //     Option<Vec<[f32; 3]>>,
    //     Option<Vec<[f32; 3]>>,
    //     Option<Vec<u32>>,
    // )> = vec![
    let mut meshes: Vec<Vec<Vec<Mesh>>> = vec![];
    // meshes.push(create_sphere(1.0, 100, 50));
    // meshes.push(sd::ParametricSurface::new(sd::ParametricSurface {
    //     f: math_func::klein_bottle,
    //     umin: 0.0,
    //     umax: PI,
    //     vmin: 0.0,
    //     vmax: 2.0 * PI,
    //     u_segments: 120,
    //     v_segments: 40,
    //     scale: 1.0,
    //     ..Default::default()
    // }));
    // meshes.push(create_cube([0.0, 0.0, 0.0], 10.0));
    let gap = 1.0;
    // y
    for x in 0..16 {
        meshes.push(vec![]);
        // z
        for y in 0..16 {
            meshes[x].push(vec![]);
            // x
            for z in 0..16 {
                meshes[x][y].push(create_cube(
                    [x as f32 * gap, y as f32 * gap, z as f32 * gap],
                    1.0,
                ));
            }
        }
    }
    // for x in 0..10 {
    //     meshes[x].push(vec![]);
    //     for y in 0..10 {
    //         meshes[x][y].push(vec![]);
    //         for z in 0..10 {
    //             meshes[x][y][z] =
    //                 create_cube([x as f32 * gap, y as f32 * gap, z as f32 * gap], 1.0);
    //             // meshes.push(create_cube(
    //             //     [x as f32 * gap, y as f32 * gap, z as f32 * gap],
    //             //     1.0,
    //             // ));
    //         }
    //     }
    // }
    common::run(&meshes, light_data);
}
