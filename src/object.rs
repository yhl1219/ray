use crate::config::*;
use crate::common::*;
use crate::material::Material;

use std::sync::Arc;

type DynMaterial = dyn Material + Sync + Send;

pub trait Object {
    fn intersect(&self, rec: &mut HitRecord, ray: &Ray) -> bool;
    fn bounding_box(&self) -> AABB;
}

pub type Primitive = dyn Object + Sync + Send;

pub struct Sphere {
    c: Point3f,
    r: Fp,
    mat: Arc<DynMaterial>
}

impl Sphere {
    pub fn new(center: Point3f, radius: Fp, material: Arc<DynMaterial>) -> Sphere {
        Sphere {
            c: center,
            r: radius,
            mat: material,
        }
    }
}

impl Object for Sphere {
    fn intersect(&self, rec: &mut HitRecord, ray: &Ray) -> bool {
        let oc = ray.o - self.c;
        // let a = ray.d.norm_squared(); // assert_eq!(a, 1.0);
        let a = 1.0;
        let half_b = oc.dot(&ray.d);
        let c = oc.norm_squared() - self.r * self.r;

        let delta = half_b * half_b - a * c;
        if delta < 0.0 {
            return false
        }

        let sqrtd = delta.sqrt();
        let mut t = (-half_b - sqrtd) / a;
        if t < T_MIN {
            t = (-half_b + sqrtd) / a;
            if t < T_MIN {
                return false
            }
        }

        rec.t = t;
        rec.pos = ray.at(t);
        rec.norm = (rec.pos - self.c) / self.r; // outward normal
        rec.mat = Some(self.mat.clone());
        true
    }

    fn bounding_box(&self) -> AABB {
        let r = self.r;
        let rv = Vector3f::new(r, r, r);
        AABB { min: self.c - rv, max: self.c + rv }
    }
}

struct Triangle {
    a: Point3f,
    b: Point3f,
    c: Point3f,
    norm: Vector3f,
    mat: Arc<DynMaterial>,
}

impl Triangle {
    pub fn new(a: Point3f, b: Point3f, c: Point3f, material: Arc<DynMaterial>) -> Self {
        let ab = b - a;
        let ac = c - a;
        Self {
            a, b, c,
            norm: ab.cross(&ac).normalize(),
            mat: material,
        }
    }
}

impl Object for Triangle {
    fn intersect(&self, rec: &mut HitRecord, ray: &Ray) -> bool {
        let e1 = self.b - self.a;
        let e2 = self.c - self.a;
        let p = ray.d.cross(&e2);
        let det = e1.dot(&p);
        let abs_det = det.abs();
        if abs_det < 1e-6 {
            return false;
        }

        let t = if det > 0.0 { ray.o - self.a } else { self.a - ray.o };
        let udet = t.dot(&p);
        if udet < 0.0 || udet > abs_det {
            return false;
        }

        let q = t.cross(&e1);
        let vdet = ray.d.dot(&q);
        if vdet < 0.0 || udet + vdet > abs_det {
            return false;
        }

        let time = e2.dot(&q).abs() / abs_det;
        if time < T_MIN {
            return false;
        }
        
        rec.t = time;
        rec.pos = ray.at(time);
        rec.norm = self.norm;
        rec.mat = Some(self.mat.clone());
        true
    }

    fn bounding_box(&self) -> AABB {
        let a = &self.a;
        let b = &self.b;
        let c = &self.c;
        AABB {
            min: point_min(a, &point_min(b, c)),
            max: point_max(a, &point_max(b, c)),
        }
    }
}

pub enum BVHNode {
    Leaf { bbox: AABB, obj: Arc<Primitive> },
    Node { bbox: AABB, lc: Arc<BVHNode>, rc: Arc<BVHNode> },
}

impl BVHNode {
    fn get_bbox(&self) -> &AABB {
        match self {
            BVHNode::Leaf { bbox, .. } => bbox,
            BVHNode::Node { bbox, .. } => bbox,
        }
    }

    pub fn build(objs: &mut [Arc<Primitive>], n: usize, depth: usize) -> Arc<BVHNode> {
        if n <= 1 {
            let obj = &objs[0];
            let leaf = BVHNode::Leaf { 
                bbox: obj.bounding_box(),
                obj: obj.clone(),
            };
            // println!("depth {} leaf bbox {:?}", depth, obj.bounding_box());
            return Arc::new(leaf);
        }

        let axis = depth % 3;
        objs.sort_by(|a, b| {
            let x = a.bounding_box().center()[axis];
            let y = b.bounding_box().center()[axis];
            x.partial_cmp(&y).unwrap()
        });
        
        let m = n / 2;
        let (l, r) = objs.split_at_mut(m);
        let lc = BVHNode::build(l, m, depth + 1);
        let rc = BVHNode::build(r, n - m, depth + 1);
        let bbox = AABB::union(lc.get_bbox(), rc.get_bbox());
        // println!("depth {} node bbox {:?} from Union ({:?},{:?})", depth, bbox, lc.get_bbox(), rc.get_bbox());
        let node = BVHNode::Node { bbox, lc, rc };
        Arc::new(node)
    }

    pub fn from_obj(file_name: &str, transform: &Affine3f, material: Arc<DynMaterial>) -> Arc<Self> {
        let mut vertices: Vec<Point3f> = Vec::new();
        let mut faces: Vec<Arc<Primitive>> = Vec::new();
    
        let content = std::fs::read_to_string(file_name).unwrap();
        for (i, line) in content.lines().enumerate() {
            let tokens: Vec<_> = line.split(' ').collect();
            match tokens[0] {
                "v" => {
                    let x = tokens[1].parse::<Fp>().unwrap();
                    let y = tokens[2].parse::<Fp>().unwrap();
                    let z = tokens[3].parse::<Fp>().unwrap();
                    vertices.push(transform * Point3f::new(x, y, z));
                },
                "f" => {
                    let a = tokens[1].parse::<usize>().unwrap() - 1;
                    let b = tokens[2].parse::<usize>().unwrap() - 1;
                    let c = tokens[3].parse::<usize>().unwrap() - 1;
                    let face = Triangle::new(vertices[a], vertices[b], vertices[c], material.clone());
                    faces.push(Arc::new(face));
                },
                _ => {
                    println!("unrecognized line #{}: {}", i + 1, line);
                }
            }
        }

        let nr_faces = faces.len();
        Self::build(faces.as_mut_slice(), nr_faces, 0)
    }
}

impl Object for BVHNode {
    fn intersect(&self, rec: &mut HitRecord, ray: &Ray) -> bool {
        match self {
            BVHNode::Leaf { obj, .. } => {
                return obj.intersect(rec, ray);
            },
            BVHNode::Node { bbox, lc, rc } => {
                if !bbox.intersect(ray) {
                    return false;
                }
                let mut rec_l = HitRecord::new();
                let mut rec_r = HitRecord::new();
                let hit_l = lc.intersect(&mut rec_l, ray);
                let hit_r = rc.intersect(&mut rec_r, ray);
                if hit_l && hit_r {
                    *rec = if rec_l.t < rec_r.t { rec_l } else { rec_r };
                } else if hit_l {
                    *rec = rec_l;
                } else if hit_r {
                    *rec = rec_r;
                }
                hit_l || hit_r
            }
        }
    }

    fn bounding_box(&self) -> AABB {
        *self.get_bbox()
    }
}

impl AABB {
    fn intersect(&self, r: &Ray) -> bool {
        let inv_d = Vector3f::new(1.0 / r.d.x, 1.0 / r.d.y, 1.0 / r.d.z);
        let t_min = Point3f::from((self.min - r.o).component_mul(&inv_d));
        let t_max = Point3f::from((self.max - r.o).component_mul(&inv_d));

        let p = point_min(&t_min, &t_max);
        let q = point_max(&t_min, &t_max);
        let u = p.x.max(p.y.max(p.z));
        let v = q.x.min(q.y.min(q.z));
        u <= v && v > 0.0
    }
}
