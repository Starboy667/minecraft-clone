#![allow(dead_code)]

use cgmath::*;

use crate::common::{Mesh, Vertex};
#[path = "../src/math_func.rs"]
mod math_func;

pub fn torus_data(
    r_torus: f32,
    r_tube: f32,
    n_torus: usize,
    n_tube: usize,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    let mut positions: Vec<[f32; 3]> =
        Vec::with_capacity((4 * (n_torus - 1) * (n_tube - 1)) as usize);
    let mut normals: Vec<[f32; 3]> =
        Vec::with_capacity((4 * (n_torus - 1) * (n_tube - 1)) as usize);

    for i in 0..n_torus - 1 {
        for j in 0..n_tube - 1 {
            let u = i as f32 * 360.0 / (n_torus as f32 - 1.0);
            let v = j as f32 * 360.0 / (n_tube as f32 - 1.0);
            let u1 = (i as f32 + 1.0) * 360.0 / (n_torus as f32 - 1.0);
            let v1 = (j as f32 + 1.0) * 360.0 / (n_tube as f32 - 1.0);
            let p0 = math_func::torus_position(r_torus, r_tube, Deg(u), Deg(v));
            let p1 = math_func::torus_position(r_torus, r_tube, Deg(u1), Deg(v));
            let p2 = math_func::torus_position(r_torus, r_tube, Deg(u1), Deg(v1));
            let p3 = math_func::torus_position(r_torus, r_tube, Deg(u), Deg(v1));

            // positions
            positions.push(p0);
            positions.push(p1);
            positions.push(p2);
            positions.push(p2);
            positions.push(p3);
            positions.push(p0);

            // normals
            let ca = Vector3::new(p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]);
            let db = Vector3::new(p3[0] - p1[0], p3[1] - p1[1], p3[2] - p1[2]);
            let cp = (ca.cross(db)).normalize();

            normals.push([cp[0], cp[1], cp[2]]);
            normals.push([cp[0], cp[1], cp[2]]);
            normals.push([cp[0], cp[1], cp[2]]);
            normals.push([cp[0], cp[1], cp[2]]);
            normals.push([cp[0], cp[1], cp[2]]);
            normals.push([cp[0], cp[1], cp[2]]);
        }
    }
    (positions, normals)
}

// pub fn sphere_data(r: f32, u: usize, v: usize) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>) {
// pub fn sphere_data(r: f32, u: usize, v: usize) -> Mesh {
//     let mut positions: Vec<[f32; 3]> = Vec::with_capacity((4 * (u - 1) * (v - 1)) as usize);
//     let mut normals: Vec<[f32; 3]> = Vec::with_capacity((4 * (u - 1) * (v - 1)) as usize);
//     let uvs: Vec<[f32; 2]> = Vec::with_capacity((4 * (u - 1) * (v - 1)) as usize);

//     for i in 0..u - 1 {
//         for j in 0..v - 1 {
//             let theta = i as f32 * 180.0 / (u as f32 - 1.0);
//             let phi = j as f32 * 360.0 / (v as f32 - 1.0);
//             let theta1 = (i as f32 + 1.0) * 180.0 / (u as f32 - 1.0);
//             let phi1 = (j as f32 + 1.0) * 360.0 / (v as f32 - 1.0);
//             let p0 = math_func::sphere_position(r, Deg(theta), Deg(phi));
//             let p1 = math_func::sphere_position(r, Deg(theta1), Deg(phi));
//             let p2 = math_func::sphere_position(r, Deg(theta1), Deg(phi1));
//             let p3 = math_func::sphere_position(r, Deg(theta), Deg(phi1));

//             // positions
//             positions.push(p0);
//             positions.push(p1);
//             positions.push(p3);
//             positions.push(p1);
//             positions.push(p2);
//             positions.push(p3);

//             // normals
//             normals.push([p0[0] / r, p0[1] / r, p0[2] / r]);
//             normals.push([p1[0] / r, p1[1] / r, p1[2] / r]);
//             normals.push([p3[0] / r, p3[1] / r, p3[2] / r]);
//             normals.push([p1[0] / r, p1[1] / r, p1[2] / r]);
//             normals.push([p2[0] / r, p2[1] / r, p2[2] / r]);
//             normals.push([p3[0] / r, p3[1] / r, p3[2] / r]);
//         }
//     }
// }
// pub fn create_sphere(radius: f32, sectors: usize, stacks: usize) -> Mesh {
//     let mut vertices = Vec::new();
//     let mut indices = Vec::new();

//     for i in 0..stacks {
//         let theta = std::f32::consts::PI * i as f32 / (stacks as f32 - 1.0);
//         for j in 0..sectors {
//             let phi = 2.0 * std::f32::consts::PI * j as f32 / (sectors as f32 - 1.0);

//             let x = radius * theta.sin() * phi.cos();
//             let y = radius * theta.sin() * phi.sin();
//             let z = radius * theta.cos();

//             let u = j as f32 / (sectors as f32 - 1.0);
//             let v = i as f32 / (stacks as f32 - 1.0);

//             vertices.push(Vertex {
//                 position: [x, y, z, 1.0],
//                 normal: [x / radius, y / radius, z / radius, 1.0],
//                 color: [1.0, 0.0, 0.0, 1.0],
//                 // uv: [u, v],
//             });
//         }
//     }

//     // Generating indices
//     for i in 0..stacks - 1 {
//         for j in 0..sectors - 1 {
//             let first = i * sectors + j;
//             let second = first + sectors;

//             // Create the first triangle
//             indices.push(first as u32);
//             indices.push(second as u32);
//             indices.push((first + 1) as u32);

//             // Create the second triangle
//             indices.push((first + 1) as u32);
//             indices.push(second as u32);
//             indices.push((second + 1) as u32);
//         }
//     }

//     Mesh { vertices, indices }
// }

pub fn cube_positions() -> Vec<[f32; 3]> {
    [
        // front (0, 0, 1)
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [-1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0],
        [1.0, -1.0, 1.0],
        [1.0, 1.0, 1.0],
        // right (1.0, 0, 0)
        [1.0, -1.0, 1.0],
        [1.0, -1.0, -1.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [1.0, -1.0, -1.0],
        [1.0, 1.0, -1.0],
        // back (0, 0, -1.0)
        [1.0, -1.0, -1.0],
        [-1.0, -1.0, -1.0],
        [1.0, 1.0, -1.0],
        [1.0, 1.0, -1.0],
        [-1.0, -1.0, -1.0],
        [-1.0, 1.0, -1.0],
        // left (-1.0, 0, 0)
        [-1.0, -1.0, -1.0],
        [-1.0, -1.0, 1.0],
        [-1.0, 1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0],
        [-1.0, 1.0, 1.0],
        // top (0, 1.0, 0)
        [-1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [-1.0, 1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, -1.0],
        // bottom (0, -1.0, 0)
        [-1.0, -1.0, -1.0],
        [1.0, -1.0, -1.0],
        [-1.0, -1.0, 1.0],
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, -1.0],
        [1.0, -1.0, 1.0],
    ]
    .to_vec()
}

pub fn cube_colors() -> Vec<[f32; 3]> {
    [
        // front - blue
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        // right - red
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        // back - yellow
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        // left - aqua
        [0.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
        // top - green
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        // bottom - fuchsia
        [1.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
    ]
    .to_vec()
}

pub fn cube_normals() -> Vec<[f32; 3]> {
    [
        // front
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        // right
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        // back
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        // left
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        // top
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        // bottom
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
    ]
    .to_vec()
}

pub fn generate_cube_vertices(cx: f32, cy: f32, cz: f32, side_length: f32) -> Vec<[f32; 3]> {
    let half_side = side_length / 2.0;

    vec![
        // Front face
        [cx - half_side, cy - half_side, cz + half_side],
        [cx + half_side, cy - half_side, cz + half_side],
        [cx - half_side, cy + half_side, cz + half_side],
        [cx + half_side, cy + half_side, cz + half_side],
        [cx - half_side, cy + half_side, cz + half_side],
        [cx + half_side, cy - half_side, cz + half_side],
        // Right face
        [cx + half_side, cy - half_side, cz + half_side],
        [cx + half_side, cy - half_side, cz - half_side],
        [cx + half_side, cy + half_side, cz + half_side],
        [cx + half_side, cy + half_side, cz - half_side],
        [cx + half_side, cy + half_side, cz + half_side],
        [cx + half_side, cy - half_side, cz - half_side],
        // Back face
        [cx + half_side, cy - half_side, cz - half_side],
        [cx - half_side, cy - half_side, cz - half_side],
        [cx + half_side, cy + half_side, cz - half_side],
        [cx - half_side, cy + half_side, cz - half_side],
        [cx + half_side, cy + half_side, cz - half_side],
        [cx - half_side, cy - half_side, cz - half_side],
        // Left face
        [cx - half_side, cy - half_side, cz - half_side],
        [cx - half_side, cy - half_side, cz + half_side],
        [cx - half_side, cy + half_side, cz - half_side],
        [cx - half_side, cy + half_side, cz + half_side],
        [cx - half_side, cy + half_side, cz - half_side],
        [cx - half_side, cy - half_side, cz + half_side],
        // Top face
        [cx - half_side, cy + half_side, cz + half_side],
        [cx + half_side, cy + half_side, cz + half_side],
        [cx - half_side, cy + half_side, cz - half_side],
        [cx + half_side, cy + half_side, cz - half_side],
        [cx - half_side, cy + half_side, cz - half_side],
        [cx + half_side, cy + half_side, cz + half_side],
        // Bottom face
        [cx - half_side, cy - half_side, cz - half_side],
        [cx + half_side, cy - half_side, cz - half_side],
        [cx - half_side, cy - half_side, cz + half_side],
        [cx + half_side, cy - half_side, cz + half_side],
        [cx - half_side, cy - half_side, cz + half_side],
        [cx + half_side, cy - half_side, cz - half_side],
    ]
}

fn generate_cube_texture_coordinates() -> Vec<[f32; 2]> {
    let texture_coordinates = vec![
        // Front face
        [0.0, 1.0 - 0.0],
        [1.0, 1.0 - 0.0],
        [0.0, 1.0 - 1.0],
        [1.0, 1.0 - 1.0],
        [0.0, 1.0 - 1.0],
        [1.0, 1.0 - 0.0],
        // Right face
        [0.0, 1.0 - 0.0],
        [1.0, 1.0 - 0.0],
        [0.0, 1.0 - 1.0],
        [1.0, 1.0 - 1.0],
        [0.0, 1.0 - 1.0],
        [1.0, 1.0 - 0.0],
        // Back face
        [0.0, 1.0 - 0.0],
        [1.0, 1.0 - 0.0],
        [0.0, 1.0 - 1.0],
        [1.0, 1.0 - 1.0],
        [0.0, 1.0 - 1.0],
        [1.0, 1.0 - 0.0],
        // Left face
        [0.0, 1.0 - 0.0],
        [1.0, 1.0 - 0.0],
        [0.0, 1.0 - 1.0],
        [1.0, 1.0 - 1.0],
        [0.0, 1.0 - 1.0],
        [1.0, 1.0 - 0.0],
        // Top face
        [0.0, 1.0 - 0.0],
        [1.0, 1.0 - 0.0],
        [0.0, 1.0 - 1.0],
        [1.0, 1.0 - 1.0],
        [0.0, 1.0 - 1.0],
        [1.0, 1.0 - 0.0],
        // Bottom face
        [0.0, 1.0 - 0.0],
        [1.0, 1.0 - 0.0],
        [0.0, 1.0 - 1.0],
        [1.0, 1.0 - 1.0],
        [0.0, 1.0 - 1.0],
        [1.0, 1.0 - 0.0],
    ];
    texture_coordinates
}

pub fn create_cube(pos: [f32; 3], size: f32) -> Mesh {
    // let positeions = cube_positions();
    let colors = cube_colors();
    let normals = cube_normals();
    // let position = generate_cube_vertices(pos[0], pos[1], pos[2], 1.0);
    let texture_coordinates = generate_cube_texture_coordinates();
    let position = generate_cube_vertices(pos[0], pos[1], pos[2], 1.0) // Generates a unit cube
        .iter()
        .map(|&vertex| [vertex[0] * size, vertex[1] * size, vertex[2] * size])
        .collect::<Vec<_>>();
    let vertices: Vec<Vertex> = position
        .iter()
        .zip(
            colors
                .iter()
                .zip(normals.iter().zip(texture_coordinates.iter())),
        )
        .map(|(position, (color, (normal, text_coords)))| Vertex {
            position: [position[0], position[1], position[2], 1.0],
            normal: [normal[0], normal[1], normal[2], 1.0],
            color: [color[0], color[1], color[2], 1.0],
            tex_coords: *text_coords, // Assuming text_coords is of the form [f32; 2]
        })
        .collect();

    let indices: Vec<u32> = (0..vertices.len() as u32).collect();

    Mesh { vertices, indices }
}
