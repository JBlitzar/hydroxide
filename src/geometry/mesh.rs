use crate::{
    bvh::AABB,
    geometry::Hittable,
    material::{HitRecord, Material},
    vec3::{Ray, Vec3},
};

pub struct Triangle {
    pub(crate) v0: Vec3,
    pub(crate) v1: Vec3,
    pub(crate) v2: Vec3,
    pub normal: Vec3,
    pub e01: Vec3,
    pub e02: Vec3,
}

impl Triangle {
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3) -> Self {
        let e01 = v1.sub(&v0);
        let e02 = v2.sub(&v0);
        let normal = e01.cross(&e02).normalize();
        Triangle {
            v0,
            v1,
            v2,
            normal,
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
        if (det.abs() < 1e-8) {
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
        let normal = if ray.direction.dot(&self.normal) > 0.0 {
            self.normal.scalar_mul(-1.0)
        } else {
            self.normal
        };
        Some(HitRecord {
            point: ray.origin.add(&ray.direction.scalar_mul(t)),
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
    pub fn from_stl(path: &str, material: Box<dyn Material>) -> Self {
        let mut file = std::fs::File::open(path).expect("failed to open STL file");
        let stl = stl_io::read_stl(&mut file).expect("failed to read STL file");
        let triangles = stl
            .faces
            .into_iter()
            .map(|face| {
                let v0 = Vec3::new(
                    stl.vertices[face.vertices[0] as usize][0] as f64,
                    stl.vertices[face.vertices[0] as usize][1] as f64,
                    stl.vertices[face.vertices[0] as usize][2] as f64,
                );
                let v1 = Vec3::new(
                    stl.vertices[face.vertices[1] as usize][0] as f64,
                    stl.vertices[face.vertices[1] as usize][1] as f64,
                    stl.vertices[face.vertices[1] as usize][2] as f64,
                );
                let v2 = Vec3::new(
                    stl.vertices[face.vertices[2] as usize][0] as f64,
                    stl.vertices[face.vertices[2] as usize][1] as f64,
                    stl.vertices[face.vertices[2] as usize][2] as f64,
                );
                Triangle::new(v0, v1, v2)
            })
            .collect();
        MeshBVH::new(triangles, material)
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

    fn hit(&self, ray: &Ray) -> Option<HitRecord> {
        self.hit_node(ray, self.root)
    }

    fn hit_node(&self, ray: &Ray, idx: usize) -> Option<HitRecord> {
        let node = &self.nodes[idx];
        if !node.bbox.hit(ray) { return None; }
        
        if node.is_leaf {
            // only here does material appear
            return self.triangles[node.triangle_index].hit(ray, self.material.as_ref());
        }
        
        let left = self.hit_node(ray, node.left);
        let right = self.hit_node(ray, node.right);


        match (left, right) {
            (Some(l), Some(r)) => if l.t < r.t { Some(l) } else { Some(r) },
            (Some(l), None) => Some(l),
            (None, Some(r)) => Some(r),
            (None, None) => None,
        }
    }



}

impl Hittable for MeshBVH {
    fn hit(&'_ self, ray: &Ray) -> Option<HitRecord<'_>> {
        self.hit(ray)
    }

    fn bounding_box(&self) -> AABB {
        if self.nodes.is_empty() {
            AABB::new(Vec3::ZERO, Vec3::ZERO)
        } else {
            self.nodes[self.root].bbox.clone()
        }
    }
}