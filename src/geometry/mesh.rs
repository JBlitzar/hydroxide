use std::io::{Read, Seek};

use crate::{
    bvh::AABB,
    geometry::Hittable,
    material::{HitRecord, Material},
    vec3::{Ray, Vec3},
};

#[derive(Clone)]
pub struct Triangle {
    pub(crate) v0: Vec3,
    pub(crate) v1: Vec3,
    pub(crate) v2: Vec3,
    pub normal: Vec3,
    pub n0: Vec3,
    pub n1: Vec3,
    pub n2: Vec3,
    pub e01: Vec3,
    pub e02: Vec3,
}

impl Triangle {
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3) -> Self {
        Self::new_with_normals(v0, v1, v2, None, None, None)
    }

    pub fn new_with_normals(
        v0: Vec3,
        v1: Vec3,
        v2: Vec3,
        n0: Option<Vec3>,
        n1: Option<Vec3>,
        n2: Option<Vec3>,
    ) -> Self {
        let e01 = v1.sub(&v0);
        let e02 = v2.sub(&v0);
        let normal = e01.cross(&e02).normalize();
        let n0 = n0.unwrap_or(normal);
        let n1 = n1.unwrap_or(normal);
        let n2 = n2.unwrap_or(normal);
        Triangle {
            v0,
            v1,
            v2,
            normal,
            n0,
            n1,
            n2,
            e01,
            e02,
        }
    }

    fn hit<'a>(&self, ray: &Ray, material: &'a dyn Material) -> Option<HitRecord<'a>> {
        // https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/moller-trumbore-ray-triangle-intersection.html#:~:text=.-,Implementation,-Implementing%20the%20M%C3%B6ller
        let v0v1 = self.e01;
        let v0v2 = self.e02;
        let pvec = ray.direction.cross(&v0v2);
        let det = v0v1.dot(&pvec);
        if det.abs() < 1e-8 {
            return None;
        }
        let inv_det = 1.0 / det;

        let tvec = ray.origin.sub(&self.v0);
        let u = tvec.dot(&pvec) * inv_det;
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let qvec = tvec.cross(&v0v1);
        let v = ray.direction.dot(&qvec) * inv_det;
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = v0v2.dot(&qvec) * inv_det;
        if t < 1e-8 {
            return None;
        }
        let w = 1.0 - u - v;
        let mut normal = self
            .n0
            .scalar_mul(w)
            .add(&self.n1.scalar_mul(u))
            .add(&self.n2.scalar_mul(v))
            .normalize();
        if ray.direction.dot(&normal) > 0.0 {
            normal = normal.scalar_mul(-1.0);
        }
        Some(HitRecord {
            point: ray
                .origin
                .add(&ray.direction.scalar_mul(t))
                .add(&normal.scalar_mul(1e-3)),
            normal,
            material,
            t,
        })
    }
}

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
    material: Box<dyn Material>,
    root: usize,
}

impl MeshBVH {
    fn new(triangles: Vec<Triangle>, material: Box<dyn Material>) -> Self {
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
            .map(|face| {
                [
                    face.vertices[0] as usize,
                    face.vertices[1] as usize,
                    face.vertices[2] as usize,
                ]
            })
            .collect();

        (raw_positions, raw_faces)
    }
    fn triangles_from_stl_data(
        raw_positions: Vec<Vec3>,
        raw_faces: Vec<[usize; 3]>,
        max_size: Option<f64>,
        offset: Vec3,
        rotation: Vec3,
    ) -> Vec<Triangle> {
        let offset = offset;
        let rotation = rotation;

        let raw_verts: Vec<[Vec3; 3]> = raw_faces
            .iter()
            .map(|idx| {
                [
                    raw_positions[idx[0]],
                    raw_positions[idx[1]],
                    raw_positions[idx[2]],
                ]
            })
            .collect();

        let scale = if let Some(max_size) = max_size {
            let mut small = Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
            let mut big = Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
            for [v0, v1, v2] in &raw_verts {
                for v in [v0, v1, v2] {
                    small.x = small.x.min(v.x);
                    small.y = small.y.min(v.y);
                    small.z = small.z.min(v.z);
                    big.x = big.x.max(v.x);
                    big.y = big.y.max(v.y);
                    big.z = big.z.max(v.z);
                }
            }
            let extent = (big.x - small.x).max(big.y - small.y).max(big.z - small.z);
            max_size / extent
        } else {
            1.0
        };

        // Build smooth vertex normals by averaging adjacent face normals.
        let mut normal_sums = vec![Vec3::ZERO; raw_positions.len()];
        for [i0, i1, i2] in &raw_faces {
            let v0 = raw_positions[*i0];
            let v1 = raw_positions[*i1];
            let v2 = raw_positions[*i2];
            let face_n = v1.sub(&v0).cross(&v2.sub(&v0)).normalize();
            normal_sums[*i0] = normal_sums[*i0].add(&face_n);
            normal_sums[*i1] = normal_sums[*i1].add(&face_n);
            normal_sums[*i2] = normal_sums[*i2].add(&face_n);
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

        let triangles: Vec<Triangle> = raw_faces
            .into_iter()
            .map(|[i0, i1, i2]| {
                let v0 = raw_positions[i0];
                let v1 = raw_positions[i1];
                let v2 = raw_positions[i2];

                let tv0 = v0.scalar_mul(scale).rotate(&rotation).add(&offset);
                let tv1 = v1.scalar_mul(scale).rotate(&rotation).add(&offset);
                let tv2 = v2.scalar_mul(scale).rotate(&rotation).add(&offset);

                let tn0 = vertex_normals[i0].rotate(&rotation).normalize();
                let tn1 = vertex_normals[i1].rotate(&rotation).normalize();
                let tn2 = vertex_normals[i2].rotate(&rotation).normalize();

                Triangle::new_with_normals(tv0, tv1, tv2, Some(tn0), Some(tn1), Some(tn2))
            })
            .collect();

        #[cfg(debug_assertions)]
        println!("Num triangles: {}", triangles.len());

        triangles
    }

    #[cfg(feature = "native")]
    pub fn from_stl(
        path: &str,
        material: Box<dyn Material>,
        max_size: Option<f64>,
        offset: Option<Vec3>,
        rotation: Option<Vec3>,
    ) -> Self {
        let mut file = std::fs::File::open(path).expect("failed to open STL file");
        let (pos, faces) = Self::parse_stl_read(&mut file);
        let tris = Self::triangles_from_stl_data(
            pos,
            faces,
            max_size,
            offset.unwrap_or(Vec3::ZERO),
            rotation.unwrap_or(Vec3::ZERO),
        );
        MeshBVH::new(tris, material)
    }

    pub fn from_stl_bytes(
        data: &[u8],
        material: Box<dyn Material>,
        max_size: Option<f64>,
        offset: Option<Vec3>,
        rotation: Option<Vec3>,
    ) -> Self {
        let mut cursor = std::io::Cursor::new(data);
        let (pos, faces) = Self::parse_stl_read(&mut cursor);
        let tris = Self::triangles_from_stl_data(
            pos,
            faces,
            max_size,
            offset.unwrap_or(Vec3::ZERO),
            rotation.unwrap_or(Vec3::ZERO),
        );
        MeshBVH::new(tris, material)
    }

    pub fn build_cube(center: Vec3, size: f64, material: Box<dyn Material>) -> Self {
        let half = size / 2.0;
        let v0 = center.add(&Vec3::new(-half, -half, -half));
        let v1 = center.add(&Vec3::new(half, -half, -half));
        let v2 = center.add(&Vec3::new(half, half, -half));
        let v3 = center.add(&Vec3::new(-half, half, -half));
        let v4 = center.add(&Vec3::new(-half, -half, half));
        let v5 = center.add(&Vec3::new(half, -half, half));
        let v6 = center.add(&Vec3::new(half, half, half));
        let v7 = center.add(&Vec3::new(-half, half, half));

        let triangles = vec![
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
        ];

        MeshBVH::new(triangles, material)
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
