use std::cell::*;
use std::cmp::*;
use std::collections::*;
use std::ops::DerefMut;
use std::rc::Rc;

type ARGB = [u8; 4];

// region OctTree Quantizer

type Link = Rc<RefCell<Node>>;

#[derive(Debug)]
enum Node {
    Leaf {
        red_sum: u32,
        green_sum: u32,
        blue_sum: u32,
        pixel_count: u32,
    },
    Branch {
        children: [Option<Link>; 8],
    },
}

impl Node {
    fn value(&self) -> (u32, u32, u32, u32) {
        match *self {
            Node::Leaf {
                red_sum,
                green_sum,
                blue_sum,
                pixel_count,
            } => (red_sum, green_sum, blue_sum, pixel_count),
            Node::Branch { .. } => (0, 0, 0, 0),
        }
    }

    fn palette_sort(&self, colors: &mut BTreeSet<ARGB>) {
        match self {
            Node::Leaf {
                red_sum,
                green_sum,
                blue_sum,
                pixel_count,
            } => {
                let color = [0xFF,
                    ((red_sum / pixel_count) & 0xFF) as u8,
                    ((green_sum / pixel_count) & 0xFF) as u8,
                    ((blue_sum / pixel_count) & 0xFF) as u8
                ];

                colors.insert(color);
            }
            Node::Branch { children } => {
                for child in children.iter() {
                    match child {
                        None => {}
                        Some(link) => {
                            link.try_borrow_mut()
                                .expect("palette get node")
                                .palette_sort(colors);
                        }
                    }
                }
            }
        }
    }

    fn palette(&self, colors: &mut HashSet<ARGB>) {
        match self {
            Node::Leaf {
                red_sum,
                green_sum,
                blue_sum,
                pixel_count,
            } => {
                let color = [0xFF,
                    ((red_sum / pixel_count) & 0xFF) as u8,
                    ((green_sum / pixel_count) & 0xFF) as u8,
                    ((blue_sum / pixel_count) & 0xFF) as u8
                ];

                colors.insert(color);
            }
            Node::Branch { children } => {
                for child in children.iter() {
                    match child {
                        None => {}
                        Some(link) => {
                            link.try_borrow_mut()
                                .expect("palette get node")
                                .palette(colors);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct OctTree {
    leaf_count: u32,
    root: Option<Link>,
    node_list: [Vec<Link>; 8],
}

const MASK: [u8; 8] = [0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01];

impl OctTree {
    fn new() -> Self {
        let mut tree = OctTree {
            leaf_count: 0,
            root: None,
            node_list: MASK.map(|_| Vec::new()),
        };

        tree.root = Some(tree.create_node(0));

        tree
    }

    fn create_node(&mut self, level: usize) -> Link {
        if level == 8 {
            self.leaf_count += 1;

            let node = Node::Leaf {
                red_sum: 0,
                green_sum: 0,
                blue_sum: 0,
                pixel_count: 0,
            };

            Rc::new(RefCell::new(node))
        } else {
            let node = Node::Branch {
                children: MASK.map(|_| None as Option<Link>),
            };
            let link = Rc::new(RefCell::new(node));
            self.node_list[level].push(link.clone());

            link
        }
    }

    fn add_color(&mut self, link: &mut Link, color: &ARGB, in_level: usize) {
        let mut node: RefMut<Node> = link.try_borrow_mut().expect("add_color get node");

        match node.deref_mut() {
            Node::Leaf {
                pixel_count,
                red_sum,
                green_sum,
                blue_sum,
                ..
            } => {
                *pixel_count += 1;
                *red_sum += color[1] as u32;
                *green_sum += color[2] as u32;
                *blue_sum += color[3] as u32;
            }
            Node::Branch { children, .. } => {
                let shift = 7 - in_level;
                let mask = MASK[in_level];
                let n_index = (((color[1] & mask) as usize >> shift << 2)
                    | ((color[2] & mask) as usize >> shift << 1)
                    | ((color[3] & mask) as usize >> shift)) as usize;

                let link = children[n_index]
                    .get_or_insert_with(|| self.create_node(in_level + 1));

                self.add_color(link, color, in_level + 1)
            }
        }
    }

    fn reduce_tree(&mut self) {
        let mut red_sum: u32 = 0;
        let mut green_sum: u32 = 0;
        let mut blue_sum: u32 = 0;
        let mut pixel_count: u32 = 0;

        let link = self
            .node_list
            .iter_mut()
            .rfind(|vec| !vec.is_empty())
            .expect("reduce_tree get vec")
            .pop()
            .expect("reduce_tree get link");

        let mut current = link.try_borrow_mut().expect("reduce_tree get current");

        match current.deref_mut() {
            Node::Leaf { .. } => {}
            Node::Branch { children, .. } => {
                for child in children.iter() {
                    match child {
                        None => {}
                        Some(link) => {
                            let (red, green, blue, pixel) = link.try_borrow()
                                .expect("get link value")
                                .value();
                            red_sum += red;
                            green_sum += green;
                            blue_sum += blue;
                            pixel_count += pixel;

                            self.leaf_count -= 1
                        }
                    }
                }
            }
        }

        let leaf = Node::Leaf {
            red_sum,
            green_sum,
            blue_sum,
            pixel_count,
        };

        *current = leaf;

        self.leaf_count += 1
    }

    fn color_palette(&self, sort: bool) -> Box<[ARGB]> {
        let node: Ref<Node> = self.root.as_ref().unwrap().try_borrow().unwrap();

        if sort {
            let mut palette = BTreeSet::new();
            node.palette_sort(&mut palette);
            palette.into_iter().collect()
        } else {
            let mut palette = HashSet::new();
            node.palette(&mut palette);
            palette.into_iter().collect()
        }
    }
}

pub fn octtree_quantizer(colors: &[ARGB], max_color_count: u32, sort: bool) -> Box<[ARGB]> {
    let mut tree = OctTree::new();
    let mut root = tree.root.clone().unwrap();

    for color in colors {
        tree.add_color(&mut root, color, 0);

        while tree.leaf_count > max_color_count {
            tree.reduce_tree()
        }
    }

    tree.color_palette(sort)
}

// end region


// region MedianCut Quantizer

#[derive(Debug)]
struct Cluster {
    raw: Vec<ARGB>,
    largest_spread: u8,
    component_with_largest_spread: usize,
}

impl Cluster {
    fn new(colors: &[ARGB]) -> Self {
        let raw = Vec::from(colors);
        let mut largest_spread: u8 = 0;
        let mut component_with_largest_spread: usize = 1;

        for component in 1..3 {
            let mut min_: u8 = 0xFF;
            let mut max_: u8 = 0x00;
            for color in &raw {
                min_ = min(min_, color[component]);
                max_ = max(max_, color[component]);
            }

            let spread = max_ - min_;

            if spread > largest_spread {
                largest_spread = spread;
                component_with_largest_spread = component;
            }
        }

        Cluster {
            raw,
            largest_spread,
            component_with_largest_spread,
        }
    }

    fn split(&mut self) -> Vec<Cluster> {
        self.raw.sort_by_key(|color| color[self.component_with_largest_spread]);
        let median = self.raw.len() / 2;

        vec![
            Cluster::new(&self.raw[..median]),
            Cluster::new(&self.raw[median..]),
        ]
    }

    fn avg(&self) -> ARGB {
        let total = self.raw.len() as u64;
        let sum = self.raw.iter().fold([0u64; 3], |acc, color| {
            [
                acc[0] + color[1] as u64,
                acc[1] + color[2] as u64,
                acc[2] + color[3] as u64
            ]
        });

        [
            0xFF,
            (sum[0] / total) as u8,
            (sum[1] / total) as u8,
            (sum[2] / total) as u8
        ]
    }
}

pub fn mediancut_quantizer(colors: &[ARGB], max_color_count: u32, sort: bool) -> Box<[ARGB]> {
    let mut clusters = Vec::with_capacity(max_color_count as usize);

    clusters.push(Cluster::new(colors));

    while clusters.len() < max_color_count as usize {
        clusters.sort_by_key(|cluster| cluster.largest_spread);
        let mut cluster = clusters.pop().expect("mediancut get cluster");
        clusters.append(&mut cluster.split());
    }

    if sort {
        let mut palette = BTreeSet::new();
        for cluster in &clusters {
            palette.insert(cluster.avg());
        }
        palette.into_iter().collect()
    } else {
        let mut palette = HashSet::new();
        for cluster in &clusters {
            palette.insert(cluster.avg());
        }
        palette.into_iter().collect()
    }
}

// end region

// region KMeans Quantizer

fn init_centroids(elements: &HashMap<ARGB, usize>, capacity: usize) -> HashSet<ARGB> {
    let mut set = HashSet::with_capacity(capacity * 3);
    let mut keys = elements.keys();
    while set.len() < capacity {
        match keys.next() {
            None => {}
            Some(color) => {
                set.insert(*color);
            }
        }
    }

    set
}

fn distinct_elements(colors: &[ARGB], capacity: usize) -> HashMap<ARGB, usize> {
    let mut map = HashMap::with_capacity(capacity);

    for color in colors {
        let value = match map.get(color) {
            None => {
                0
            }
            Some(count) => {
                count + 1
            }
        };

        map.insert(*color, value);
    }

    map
}

fn centroid(elements: &HashMap<ARGB, usize>) -> ARGB {
    let mut sum = [0usize;3];
    for (color, count) in elements {
        sum = [
            sum[0] + color[0] as usize * count,
            sum[1] + color[1] as usize * count,
            sum[2] + color[2] as usize * count,
        ]
    }

    let size = elements.len();
    [
        0xFF,
        (sum[0] / size) as u8,
        (sum[1] / size) as u8,
        (sum[2] / size) as u8,
    ]
}

fn nearest_color(centroids: &HashSet<ARGB>, color: &ARGB) -> ARGB {
    centroids.iter().min_by_key(|item| {
        item[1] * color[1] + color[2] * color[2] + color[3] * color[3]
    }).expect("get nearest fail.").clone()
}

pub fn kmeans_quantizer(colors: &[ARGB], max_color_count: u32, sort: bool) -> Box<[ARGB]> {
    let elements = distinct_elements(colors, max_color_count as usize);
    let mut centroids = init_centroids(&elements, max_color_count as usize);
    let rc = Rc::new(RefCell::new(HashMap::with_capacity(elements.capacity())));

    for (color, count) in elements {
        let nearest = nearest_color(&centroids, &color);
        let rc = rc.clone();
        let mut clusters = rc.borrow_mut();
        let record = clusters.get_mut(&nearest);
        match record {
            None => {
                clusters.insert(nearest,  HashMap::from([(color, count)]));
            }
            Some(hash_map) => {
                hash_map.insert(color, count);
            }
        }
    }

    while !centroids.is_empty() {
        let rc = rc.clone();
        let mut clusters = rc.borrow_mut();
        for old in &centroids {
            let cluster = clusters.remove(old)
                .unwrap_or_else(|| HashMap::new());
            let new = centroid(&cluster);

            clusters.insert(new, cluster);
        }
        centroids.clear();

        let ids: HashSet<ARGB> = clusters.keys().map(|i| i.clone()).collect();
        for (centroid, cluster) in clusters.iter_mut() {
            for (color, count) in cluster.clone() {
                let new_centroid = nearest_color(&ids, &color);
                let rc = rc.clone();
                let mut clusters = rc.borrow_mut();

                let record = clusters.get_mut(&new_centroid);
                match record {
                    None => {
                        clusters.insert(new_centroid,  HashMap::from([(color.clone(), count)]));
                    }
                    Some(hash_map) => {
                        hash_map.insert(color.clone(), count);
                    }
                };
                cluster.remove(&color);

                centroids.insert(*centroid);
                centroids.insert(new_centroid);
            }
        }
    }

    let clusters = rc.borrow();
    let iter = clusters.keys().map(|i| i.clone());

    if sort {
        BTreeSet::from_iter(iter).into_iter().collect()
    } else {
        iter.collect()
    }
}

// end region