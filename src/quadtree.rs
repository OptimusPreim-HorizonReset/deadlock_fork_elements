#![allow(dead_code)]
use crate::body::Body;
use ultraviolet::Vec3;

#[derive(Clone, Copy)]
pub struct Oct {
    pub center: Vec3,
    pub size: f32,
}

impl Oct {
    pub fn new_containing(bodies: &[Body]) -> Self {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut min_z = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut max_z = f32::MIN;

        for body in bodies {
            min_x = min_x.min(body.pos.x);
            min_y = min_y.min(body.pos.y);
            min_z = min_z.min(body.pos.z);
            max_x = max_x.max(body.pos.x);
            max_y = max_y.max(body.pos.y);
            max_z = max_z.max(body.pos.z);
        }

        let center = Vec3::new(
            (min_x + max_x) * 0.5,
            (min_y + max_y) * 0.5,
            (min_z + max_z) * 0.5,
        );
        let size = (max_x - min_x).max((max_y - min_y).max(max_z - min_z));

        Self { center, size }
    }

    pub fn find_octant(&self, pos: Vec3) -> usize {
        ((pos.z > self.center.z) as usize) << 2
            | ((pos.y > self.center.y) as usize) << 1
            | ((pos.x > self.center.x) as usize)
    }

    pub fn into_octant(mut self, octant: usize) -> Self {
        self.size *= 0.5;
        self.center.x += ((octant & 1) as f32 - 0.5) * self.size;
        self.center.y += (((octant >> 1) & 1) as f32 - 0.5) * self.size;
        self.center.z += (((octant >> 2) & 1) as f32 - 0.5) * self.size;
        self
    }

    pub fn subdivide(&self) -> [Oct; 8] {
        [0, 1, 2, 3, 4, 5, 6, 7].map(|i| self.into_octant(i))
    }
}

#[derive(Clone)]
pub struct Node {
    pub children: usize,
    pub next: usize,
    pub pos: Vec3,
    pub mass: f32,
    // Aggregated multipole quantities (added for GPDM multipole support)
    pub momentum: Vec3,
    pub angular_momentum: Vec3,
    pub oct: Oct,
    // Optional leaf body index for nearest-neighbor queries (None for internal nodes)
    pub body_index: Option<usize>,
}

impl Node {
    pub fn new(next: usize, oct: Oct) -> Self {
        Self {
            children: 0,
            next,
            pos: Vec3::zero(),
            mass: 0.0,
            momentum: Vec3::zero(),
            angular_momentum: Vec3::zero(),
            oct,
            body_index: None,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children == 0
    }

    pub fn is_branch(&self) -> bool {
        self.children != 0
    }

    pub fn is_empty(&self) -> bool {
        self.mass == 0.0
    }
}

pub struct Octree {
    pub t_sq: f32,
    pub e_sq: f32,
    pub nodes: Vec<Node>,
    pub parents: Vec<usize>,
}

impl Octree {
    pub const ROOT: usize = 0;

    pub fn new(theta: f32, epsilon: f32) -> Self {
        Self {
            t_sq: theta * theta,
            e_sq: epsilon * epsilon,
            nodes: Vec::new(),
            parents: Vec::new(),
        }
    }

    pub fn reserve(&mut self, nodes: usize, parents: usize) {
        if self.nodes.capacity() < nodes {
            self.nodes.reserve(nodes - self.nodes.capacity());
        }
        if self.parents.capacity() < parents {
            self.parents.reserve(parents - self.parents.capacity());
        }
    }

    pub fn clear(&mut self, oct: Oct) {
        self.parents.clear();
        if self.nodes.is_empty() {
            self.nodes.push(Node::new(0, oct));
        } else {
            self.nodes.truncate(1);
            self.nodes[0] = Node::new(0, oct);
        }
    }

    fn subdivide(&mut self, node: usize) -> usize {
        self.parents.push(node);
        let children = self.nodes.len();
        self.nodes[node].children = children;

        let nexts = [
            children + 1,
            children + 2,
            children + 3,
            children + 4,
            children + 5,
            children + 6,
            children + 7,
            self.nodes[node].next,
        ];
        let octs = self.nodes[node].oct.subdivide();
        for i in 0..8 {
            self.nodes.push(Node::new(nexts[i], octs[i]));
        }

        children
    }

    /// Insert a delegate with full multipole quantities (mass, momentum, angular momentum).
    /// This leaves the legacy `insert(pos, mass)` intact for callers that only provide mass.
    pub fn insert_delegate(
        &mut self,
        pos: Vec3,
        mass: f32,
        momentum: Vec3,
        angular_momentum: Vec3,
    ) {
        let mut node = Self::ROOT;

        while self.nodes[node].is_branch() {
            let octant = self.nodes[node].oct.find_octant(pos);
            node = self.nodes[node].children + octant;
        }

        if self.nodes[node].is_empty() {
            self.nodes[node].pos = pos;
            self.nodes[node].mass = mass;
            self.nodes[node].momentum = momentum;
            self.nodes[node].angular_momentum = angular_momentum;
            return;
        }

        let (p, m, pm, am) = (
            self.nodes[node].pos,
            self.nodes[node].mass,
            self.nodes[node].momentum,
            self.nodes[node].angular_momentum,
        );
        if pos == p {
            self.nodes[node].mass += mass;
            self.nodes[node].momentum += momentum;
            self.nodes[node].angular_momentum += angular_momentum;
            return;
        }

        loop {
            let children = self.subdivide(node);

            let o1 = self.nodes[node].oct.find_octant(p);
            let o2 = self.nodes[node].oct.find_octant(pos);

            if o1 == o2 {
                node = children + o1;
            } else {
                let n1 = children + o1;
                let n2 = children + o2;

                self.nodes[n1].pos = p;
                self.nodes[n1].mass = m;
                self.nodes[n1].momentum = pm;
                self.nodes[n1].angular_momentum = am;

                self.nodes[n2].pos = pos;
                self.nodes[n2].mass = mass;
                self.nodes[n2].momentum = momentum;
                self.nodes[n2].angular_momentum = angular_momentum;
                return;
            }
        }
    }

    pub fn insert(&mut self, pos: Vec3, mass: f32) {
        let mut node = Self::ROOT;

        while self.nodes[node].is_branch() {
            let octant = self.nodes[node].oct.find_octant(pos);
            node = self.nodes[node].children + octant;
        }

        if self.nodes[node].is_empty() {
            self.nodes[node].pos = pos;
            self.nodes[node].mass = mass;
            return;
        }

        let (p, m) = (self.nodes[node].pos, self.nodes[node].mass);
        if pos == p {
            self.nodes[node].mass += mass;
            return;
        }

        loop {
            let children = self.subdivide(node);

            let o1 = self.nodes[node].oct.find_octant(p);
            let o2 = self.nodes[node].oct.find_octant(pos);

            if o1 == o2 {
                node = children + o1;
            } else {
                let n1 = children + o1;
                let n2 = children + o2;

                self.nodes[n1].pos = p;
                self.nodes[n1].mass = m;
                self.nodes[n2].pos = pos;
                self.nodes[n2].mass = mass;
                return;
            }
        }
    }

    /// Insert a leaf and remember its original body index for neighbor queries.
    pub fn insert_with_index(&mut self, pos: Vec3, mass: f32, body_index: usize) {
        let mut node = Self::ROOT;

        while self.nodes[node].is_branch() {
            let octant = self.nodes[node].oct.find_octant(pos);
            node = self.nodes[node].children + octant;
        }

        if self.nodes[node].is_empty() {
            self.nodes[node].pos = pos;
            self.nodes[node].mass = mass;
            self.nodes[node].body_index = Some(body_index);
            return;
        }

        let (p, m, bi) = (
            self.nodes[node].pos,
            self.nodes[node].mass,
            self.nodes[node].body_index,
        );
        if pos == p {
            self.nodes[node].mass += mass;
            if self.nodes[node].body_index.is_none() {
                self.nodes[node].body_index = Some(body_index);
            }
            return;
        }

        loop {
            let children = self.subdivide(node);

            let o1 = self.nodes[node].oct.find_octant(p);
            let o2 = self.nodes[node].oct.find_octant(pos);

            if o1 == o2 {
                node = children + o1;
            } else {
                let n1 = children + o1;
                let n2 = children + o2;

                self.nodes[n1].pos = p;
                self.nodes[n1].mass = m;
                self.nodes[n1].body_index = bi;

                self.nodes[n2].pos = pos;
                self.nodes[n2].mass = mass;
                self.nodes[n2].body_index = Some(body_index);
                return;
            }
        }
    }

    /// Query the octree for leaf bodies whose positions are within `radius` of `center`.
    /// Results are appended to `out` (cleared at the start).
    pub fn query_radius(&self, center: Vec3, radius: f32, out: &mut Vec<usize>) {
        out.clear();
        if self.nodes.is_empty() {
            return;
        }
        let radius_sq = radius * radius;
        let mut stack: Vec<usize> = Vec::with_capacity(64);
        stack.push(Self::ROOT);

        while let Some(node_idx) = stack.pop() {
            let node = &self.nodes[node_idx];
            if !box_intersects_sphere(&node.oct, center, radius) {
                continue;
            }
            if node.is_leaf() {
                if let Some(bi) = node.body_index {
                    let d_sq = (node.pos - center).mag_sq();
                    if d_sq <= radius_sq {
                        out.push(bi);
                    }
                }
            } else {
                let start = node.children;
                for child in start..start + 8 {
                    stack.push(child);
                }
            }
        }
    }

    pub fn propagate(&mut self) {
        for &node in self.parents.iter().rev() {
            let i = self.nodes[node].children;
            let mut sum = Vec3::zero();
            let mut mass = 0.0;
            let mut momentum_sum = Vec3::zero();
            let mut angular_sum = Vec3::zero();

            for child in i..i + 8 {
                sum += self.nodes[child].pos * self.nodes[child].mass;
                mass += self.nodes[child].mass;
                momentum_sum += self.nodes[child].momentum;
                angular_sum += self.nodes[child].angular_momentum;
            }

            self.nodes[node].mass = mass;
            self.nodes[node].momentum = momentum_sum;
            self.nodes[node].angular_momentum = angular_sum;

            if mass > 0.0 {
                self.nodes[node].pos = sum / mass;
            } else {
                self.nodes[node].pos = Vec3::zero();
            }
        }
    }

    pub fn acc(&self, pos: Vec3, softening_sq: f32) -> Vec3 {
        let mut acc = Vec3::zero();

        let mut node = Self::ROOT;
        loop {
            let n = &self.nodes[node];
            let d = n.pos - pos;
            let d_sq = d.mag_sq();

            if n.is_leaf() || n.oct.size * n.oct.size < d_sq * self.t_sq {
                if d_sq > 0.0 {
                    let denom = (d_sq + softening_sq) * d_sq.sqrt();
                    acc += d * (n.mass / denom).min(f32::MAX);
                }

                if n.next == 0 {
                    break;
                }
                node = n.next;
            } else {
                node = n.children;
            }
        }

        acc
    }
}

fn box_intersects_sphere(oct: &Oct, center: Vec3, radius: f32) -> bool {
    let half = oct.size * 0.5;
    let mut dx = (center.x - oct.center.x).abs() - half;
    let mut dy = (center.y - oct.center.y).abs() - half;
    let mut dz = (center.z - oct.center.z).abs() - half;
    if dx < 0.0 { dx = 0.0; }
    if dy < 0.0 { dy = 0.0; }
    if dz < 0.0 { dz = 0.0; }
    let dist_sq = dx * dx + dy * dy + dz * dz;
    dist_sq <= radius * radius
}
