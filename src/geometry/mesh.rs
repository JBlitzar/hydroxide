use std::io::{Read, Seek};

use crate::{
    aabb::AABB,
    geometry::{Hittable, triangle::Triangle},
    material::{HitRecord, Material},
    vec3::{Ray, Vec3},
};

pub struct MeshBVHNode {
    bbox: AABB,
    left: usize,
    right: usize,
    is_leaf: bool,
    triangle_index: usize,
}

pub struct MeshBVH {
    nodes: Vec<MeshBVHNode>,
    triangles: Vec<Triangle>,
    pub material: Box<dyn Material>,
    root: usize,
}

impl MeshBVH {
    pub fn triangles(&self) -> &[Triangle] {
        &self.triangles
    }

    pub fn from_indexed(
        positions: &[Vec3],
        faces: &[[u32; 3]],
        material: Box<dyn Material>,
    ) -> Self {
        Self::new(Self::triangles_from_indexed(positions, faces), material)
    }

    pub fn with_material(&self, material: Box<dyn Material>) -> Self {
        Self::new(self.triangles.clone(), material)
    }

    pub fn new(triangles: Vec<Triangle>, material: Box<dyn Material>) -> Self {
        let mut bvh = MeshBVH {
            nodes: Vec::new(),
            triangles,
            material,
            root: 0,
        };
        bvh.root = bvh.build_bvh(0, bvh.triangles.len());
        bvh
    }

    fn parse_stl_read<R: Read + Seek>(reader: &mut R) -> (Vec<Vec3>, Vec<[usize; 3]>) {
        let stl = stl_io::read_stl(reader).expect("Failed to read");
        let raw_positions: Vec<Vec3> = stl
            .vertices
            .iter()
            .map(|v| Vec3::new(v[0] as f64, v[1] as f64, v[2] as f64))
            .collect();
        let raw_faces: Vec<[usize; 3]> = stl
            .faces
            .iter()
            .map(|face| [face.vertices[0], face.vertices[1], face.vertices[2]])
            .collect();

        (raw_positions, raw_faces)
    }
    pub fn triangles_from_indexed(positions: &[Vec3], faces: &[[u32; 3]]) -> Vec<Triangle> {
        let mut normal_sums = vec![Vec3::ZERO; positions.len()];
        for [i0, i1, i2] in faces {
            let v0 = positions[*i0 as usize];
            let v1 = positions[*i1 as usize];
            let v2 = positions[*i2 as usize];
            let face_n = v1.sub(&v0).cross(&v2.sub(&v0)).normalize();
            normal_sums[*i0 as usize] = normal_sums[*i0 as usize].add(&face_n);
            normal_sums[*i1 as usize] = normal_sums[*i1 as usize].add(&face_n);
            normal_sums[*i2 as usize] = normal_sums[*i2 as usize].add(&face_n);
        }

        let vertex_normals: Vec<Vec3> = normal_sums
            .into_iter()
            .map(|n| {
                if n.length_squared() == 0.0 {
                    Vec3::new(0.0, 1.0, 0.0)
                } else {
                    n.normalize()
                }
            })
            .collect();

        faces
            .iter()
            .map(|[i0, i1, i2]| {
                Triangle::new_with_normals(
                    positions[*i0 as usize],
                    positions[*i1 as usize],
                    positions[*i2 as usize],
                    Some(vertex_normals[*i0 as usize]),
                    Some(vertex_normals[*i1 as usize]),
                    Some(vertex_normals[*i2 as usize]),
                )
            })
            .collect()
    }

    fn transform_stl_data(
        raw_positions: Vec<Vec3>,
        raw_faces: Vec<[usize; 3]>,
        max_size: Option<f64>,
        offset: Vec3,
        rotation: Vec3,
    ) -> (Vec<Vec3>, Vec<[u32; 3]>) {
        let scale = if let Some(max_size) = max_size {
            let mut small = Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
            let mut big = Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
            for pos in &raw_positions {
                small.x = small.x.min(pos.x);
                small.y = small.y.min(pos.y);
                small.z = small.z.min(pos.z);
                big.x = big.x.max(pos.x);
                big.y = big.y.max(pos.y);
                big.z = big.z.max(pos.z);
            }
            let extent = (big.x - small.x).max(big.y - small.y).max(big.z - small.z);
            max_size / extent
        } else {
            1.0
        };

        let positions = raw_positions
            .iter()
            .map(|p| p.scalar_mul(scale).rotate(&rotation).add(&offset))
            .collect();
        let faces = raw_faces
            .iter()
            .map(|[i0, i1, i2]| [*i0 as u32, *i1 as u32, *i2 as u32])
            .collect();
        (positions, faces)
    }

    fn triangles_from_stl_data(
        raw_positions: Vec<Vec3>,
        raw_faces: Vec<[usize; 3]>,
        max_size: Option<f64>,
        offset: Vec3,
        rotation: Vec3,
    ) -> Vec<Triangle> {
        let (positions, faces) =
            Self::transform_stl_data(raw_positions, raw_faces, max_size, offset, rotation);
        Self::triangles_from_indexed(&positions, &faces)
    }

    #[cfg(feature = "native")]
    pub fn load_stl_indexed(
        path: &str,
        max_size: Option<f64>,
        offset: Option<Vec3>,
        rotation: Option<Vec3>,
    ) -> (Vec<Vec3>, Vec<[u32; 3]>) {
        let mut file = std::fs::File::open(path).expect("failed to open STL file");
        let (pos, faces) = Self::parse_stl_read(&mut file);
        Self::transform_stl_data(
            pos,
            faces,
            max_size,
            offset.unwrap_or(Vec3::ZERO),
            rotation.unwrap_or(Vec3::ZERO),
        )
    }

    pub fn load_stl_bytes_indexed(
        data: &[u8],
        max_size: Option<f64>,
        offset: Option<Vec3>,
        rotation: Option<Vec3>,
    ) -> (Vec<Vec3>, Vec<[u32; 3]>) {
        let mut cursor = std::io::Cursor::new(data);
        let (pos, faces) = Self::parse_stl_read(&mut cursor);
        Self::transform_stl_data(
            pos,
            faces,
            max_size,
            offset.unwrap_or(Vec3::ZERO),
            rotation.unwrap_or(Vec3::ZERO),
        )
    }

    #[cfg(feature = "native")]
    pub fn load_stl_triangles(
        path: &str,
        max_size: Option<f64>,
        offset: Option<Vec3>,
        rotation: Option<Vec3>,
    ) -> Vec<Triangle> {
        let mut file = std::fs::File::open(path).expect("failed to open STL file");
        let (pos, faces) = Self::parse_stl_read(&mut file);
        Self::triangles_from_stl_data(
            pos,
            faces,
            max_size,
            offset.unwrap_or(Vec3::ZERO),
            rotation.unwrap_or(Vec3::ZERO),
        )
    }

    pub fn load_stl_bytes_triangles(
        data: &[u8],
        max_size: Option<f64>,
        offset: Option<Vec3>,
        rotation: Option<Vec3>,
    ) -> Vec<Triangle> {
        let mut cursor = std::io::Cursor::new(data);
        let (pos, faces) = Self::parse_stl_read(&mut cursor);
        Self::triangles_from_stl_data(
            pos,
            faces,
            max_size,
            offset.unwrap_or(Vec3::ZERO),
            rotation.unwrap_or(Vec3::ZERO),
        )
    }

    #[cfg(feature = "native")]
    pub fn from_stl(
        path: &str,
        material: Box<dyn Material>,
        max_size: Option<f64>,
        offset: Option<Vec3>,
        rotation: Option<Vec3>,
    ) -> Self {
        let tris = Self::load_stl_triangles(path, max_size, offset, rotation);
        MeshBVH::new(tris, material)
    }

    pub fn from_stl_bytes(
        data: &[u8],
        material: Box<dyn Material>,
        max_size: Option<f64>,
        offset: Option<Vec3>,
        rotation: Option<Vec3>,
    ) -> Self {
        let tris = Self::load_stl_bytes_triangles(data, max_size, offset, rotation);
        MeshBVH::new(tris, material)
    }

    pub fn cube_indexed(center: Vec3, size: f64) -> (Vec<Vec3>, Vec<[u32; 3]>) {
        let half = size / 2.0;
        let positions = vec![
            center.add(&Vec3::new(-half, -half, -half)),
            center.add(&Vec3::new(half, -half, -half)),
            center.add(&Vec3::new(half, half, -half)),
            center.add(&Vec3::new(-half, half, -half)),
            center.add(&Vec3::new(-half, -half, half)),
            center.add(&Vec3::new(half, -half, half)),
            center.add(&Vec3::new(half, half, half)),
            center.add(&Vec3::new(-half, half, half)),
        ];
        let faces = vec![
            [0, 1, 2],
            [0, 2, 3],
            [1, 5, 6],
            [1, 6, 2],
            [5, 4, 7],
            [5, 7, 6],
            [4, 0, 3],
            [4, 3, 7],
            [3, 2, 6],
            [3, 6, 7],
            [4, 5, 1],
            [4, 1, 0],
        ];
        (positions, faces)
    }

    pub fn cube_triangles(center: Vec3, size: f64) -> Vec<Triangle> {
        let half = size / 2.0;
        let v0 = center.add(&Vec3::new(-half, -half, -half));
        let v1 = center.add(&Vec3::new(half, -half, -half));
        let v2 = center.add(&Vec3::new(half, half, -half));
        let v3 = center.add(&Vec3::new(-half, half, -half));
        let v4 = center.add(&Vec3::new(-half, -half, half));
        let v5 = center.add(&Vec3::new(half, -half, half));
        let v6 = center.add(&Vec3::new(half, half, half));
        let v7 = center.add(&Vec3::new(-half, half, half));

        vec![
            Triangle::new(v0, v1, v2),
            Triangle::new(v0, v2, v3),
            Triangle::new(v1, v5, v6),
            Triangle::new(v1, v6, v2),
            Triangle::new(v5, v4, v7),
            Triangle::new(v5, v7, v6),
            Triangle::new(v4, v0, v3),
            Triangle::new(v4, v3, v7),
            Triangle::new(v3, v2, v6),
            Triangle::new(v3, v6, v7),
            Triangle::new(v4, v5, v1),
            Triangle::new(v4, v1, v0),
        ]
    }

    pub fn build_cube(center: Vec3, size: f64, material: Box<dyn Material>) -> Self {
        MeshBVH::new(Self::cube_triangles(center, size), material)
    }

    fn compute_bbox(&self, start: usize, end: usize) -> AABB {
        let mut small = Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut big = Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

        for i in start..end {
            let tri = &self.triangles[i];
            for v in [&tri.v0, &tri.v1, &tri.v2] {
                small.x = small.x.min(v.x);
                small.y = small.y.min(v.y);
                small.z = small.z.min(v.z);
                big.x = big.x.max(v.x);
                big.y = big.y.max(v.y);
                big.z = big.z.max(v.z);
            }
        }
        AABB::new(small, big)
    }

    fn build_bvh(&mut self, start: usize, end: usize) -> usize {
        let node_index = self.nodes.len();
        let bbox = self.compute_bbox(start, end);
        let axis = bbox.widest_axis();
        self.triangles[start..end].sort_by(|a, b| {
            let ca = (a.v0[axis] + a.v1[axis] + a.v2[axis]) / 3.0;
            let cb = (b.v0[axis] + b.v1[axis] + b.v2[axis]) / 3.0;
            ca.partial_cmp(&cb).unwrap()
        });

        self.nodes.push(MeshBVHNode {
            bbox,
            left: 0,
            right: 0,
            is_leaf: false,
            triangle_index: 0,
        });

        if end - start == 1 {
            self.nodes[node_index].is_leaf = true;
            self.nodes[node_index].triangle_index = start;
            return node_index;
        }

        let mid = (start + end) / 2;
        self.nodes[node_index].left = self.build_bvh(start, mid);
        self.nodes[node_index].right = self.build_bvh(mid, end);
        node_index
    }

    fn hit(&self, ray: &Ray, t_max: f64) -> Option<HitRecord<'_>> {
        self.hit_node(ray, self.root, t_max)
    }

    fn hit_node(&self, ray: &Ray, idx: usize, t_max: f64) -> Option<HitRecord<'_>> {
        let node = &self.nodes[idx];
        if !node.bbox.hit(ray, t_max) {
            return None;
        }

        if node.is_leaf {
            return self.triangles[node.triangle_index].hit(ray, self.material.as_ref());
        }

        let left = self.hit_node(ray, node.left, t_max);

        let right = self.hit_node(
            ray,
            node.right,
            t_max.min(left.as_ref().map_or(f64::INFINITY, |hit| hit.t)),
        );

        match (left, right) {
            (Some(l), Some(r)) => {
                if l.t < r.t {
                    Some(l)
                } else {
                    Some(r)
                }
            }
            (Some(l), None) => Some(l),
            (None, Some(r)) => Some(r),
            (None, None) => None,
        }
    }
}

impl Hittable for MeshBVH {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn hit(&'_ self, ray: &Ray, t_max: f64) -> Option<HitRecord<'_>> {
        self.hit(ray, t_max)
    }

    fn bounding_box(&self) -> AABB {
        if self.nodes.is_empty() {
            AABB::new(Vec3::ZERO, Vec3::ZERO)
        } else {
            self.nodes[self.root].bbox.clone()
        }
    }
}
