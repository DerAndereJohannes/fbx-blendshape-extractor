use std::{io::{BufWriter, Write}, fs::File, collections::HashMap};
use serde::{Serialize, Deserialize};
use fbx::{self, Node, Property};

fn main() {
    // Define constants
    const FBXFILENAME: &'static str = "cprint-wrapped.fbx";
    const VERTEXFILENAME: &'static str = "genesis-verts.json";
    const BLENDSHAPEFILENAME: &'static str = "cprint-blendshapes.json";
    const OLDSHAPENAME: &'static str = "cprint";
    const NEWSHAPENAME: &'static str = "cprint";

    // keep the vert file consistent, but allow for alternative FBX file naming
    let input_fbx: String = {
        if std::env::args().len() == 3 {
            std::env::args().nth(1).unwrap_or(FBXFILENAME.to_string()).to_string()
        } else {
            FBXFILENAME.to_string()
        }
    };

    let input_verts: String = {
        if std::env::args().len() == 3 {
            std::env::args().nth(2).unwrap_or(VERTEXFILENAME.to_string()).to_string()
        } else {
            VERTEXFILENAME.to_string()
        }
    };

    // Load the vertex file
    let file = std::fs::File::open(&input_verts).unwrap_or_else(|_err| {
        unwrap_handler(format!("The file '{}' could not be found. Please make sure that it is in the folder.", input_verts).as_str());
        std::process::exit(0);
    });

    let reader = std::io::BufReader::new(file);
    let render_meshes: RendererMeshes = serde_json::from_reader(reader).unwrap();


    // load the fbx file
    let file = std::fs::File::open(&input_fbx).unwrap_or_else(|_err| {
        unwrap_handler(format!("The file '{}' could not be found. Please make sure that it is in the folder.", input_fbx).as_str());
        std::process::exit(0);
    });

    let reader = std::io::BufReader::new(file);
    let file = fbx::File::read_from(reader).unwrap();

    // Extract wanted blendshapes from the file
    let mut new_blendshapes: Vec<BlendShape> = vec![];
    file.children[8].children.iter()
                             .filter(|x| {
                                 if let Property::String(name) = &x.properties[1] {
                                     name.contains(OLDSHAPENAME) && 
                                         x.name == "Geometry" && 
                                         !name.contains("cbs") && 
                                         !name.contains("Tear") && 
                                         !name.contains("Mouth") && 
                                         !name.contains("Eyelashes") && 
                                         !name.contains("Eyes")
                                 } else {
                                     false
                                 }
                             }).for_each(|x| new_blendshapes.push(BlendShape::new(x, OLDSHAPENAME, NEWSHAPENAME)));

    let blendshape_holder:BlendShapes = BlendShapes { blendshapes: new_blendshapes };
    let mut unity_blendshapes: UnityBlendShapes = UnityBlendShapes { meshes: vec![] };

    // create the new blendshape vertices
    for shape in blendshape_holder.blendshapes.iter() {
        let related_geometry = render_meshes.meshes.iter().find(|x| shape.name.contains(&x.name.replace(".Shape", ""))).unwrap();
        let mut new_shape = ExchangeShapeReference::new(format!("{}__{}", related_geometry.name.replace(".Shape", ""), NEWSHAPENAME), &shape, related_geometry.vertices.len());
        let highest_index: usize = *shape.indices.last().unwrap() as usize;

        let matching = find_matching_indices(&related_geometry.vertices[..=highest_index], &related_geometry.vertices[(highest_index + 1)..]);
        for (key, value) in matching.iter() {
            if value.len() > 0 {
                let first_value: &Vector3 = &new_shape.vertices[*value.first().unwrap() as usize].clone();
                new_shape.vertices[*key as usize + new_shape.cutoff].update(first_value.x, first_value.y, first_value.z);
            }
        }
        unity_blendshapes.meshes.push(new_shape);
    }

    // export new file
    let output_file = File::create(BLENDSHAPEFILENAME).unwrap();
    let mut writer = BufWriter::new(output_file);
    serde_json::to_writer(&mut writer, &unity_blendshapes).unwrap();
    writer.flush().unwrap();
}

fn find_matching_indices(array: &[Vector3], to_match: &[Vector3]) -> HashMap<i64, Vec<i64>> {
    let mut index_map: HashMap<i64, Vec<i64>> = HashMap::with_capacity(to_match.len());

    for i in 0..to_match.len() {
        index_map.insert(i as i64, vec![]);
    }

    array.iter()
         .enumerate()
         .for_each(|(i, x)| {
            to_match.iter()
                    .enumerate()
                    .for_each(|(j, y)| {
                        if x == y {
                            index_map.get_mut(&(j as i64)).unwrap().push(i as i64);
                        }
                    });
         });

    index_map
}


#[derive(Deserialize, Debug, Default)]
struct RendererMeshes {
    meshes: Vec<SkinnedMeshRenderer>
}

#[derive(Deserialize, Debug, Default)]
struct SkinnedMeshRenderer {
    name: String,
    vertices: Vec<Vector3>
}


#[derive(Serialize, Deserialize, Debug, Clone,  Default, PartialEq)]
struct Vector3 {
    x: f64,
    y: f64,
    z: f64
}

impl Vector3 {
    pub fn update(&mut self, x: f64, y: f64, z: f64) {
        self.x = x;
        self.y = y;
        self.z = z;
    }
}

#[derive(Serialize, Debug, Default)]
struct BlendShapes {
    blendshapes: Vec<BlendShape>
}


#[derive(Serialize, Debug, Default)]
struct UnityBlendShapes {
    meshes: Vec<ExchangeShapeReference>
}

#[derive(Serialize, Debug, Default)]
struct ExchangeShapeReference {
    name: String,
    #[serde(skip_serializing)]
    cutoff: usize,
    vertices: Vec<Vector3>
}

impl ExchangeShapeReference {
    pub fn new(name: String, blendshape: &BlendShape, vertex_count: usize) -> ExchangeShapeReference {
        let mut vertices = vec![Vector3::default(); vertex_count];

        for (i, index) in blendshape.indices.iter().enumerate() {
            let start: usize = i * 3;
            vertices[*index as usize].update(-blendshape.vertices[start], blendshape.vertices[start + 1], blendshape.vertices[start + 2]); 
        }
        ExchangeShapeReference { name, cutoff: *blendshape.indices.last().unwrap() as usize + 1, vertices }
    }
}

#[derive(Serialize, Debug, Default)]
struct BlendShape {
    id: i64,
    name: String,
    indices: Vec<i64>,
    vertices: Vec<f64>,
    normals: Vec<f64>
}

impl BlendShape {

    pub fn new(input_node: &Node, old_shape_name: &str, new_shape_name: &str) -> BlendShape {
        let id: i64 = Self::extract_i64(&input_node.properties[0]).unwrap();
        let name: String = Self::extract_string(&input_node.properties[1]).unwrap().replace("\0\u{1}Geometry", "").replace(old_shape_name, new_shape_name);
        let indices: Vec<i64> = Self::extract_i64_array(&input_node.children[1].properties[0]).unwrap_or_default();
        let vertices: Vec<f64> = Self::extract_f64_array(&input_node.children[2].properties[0]).unwrap_or_default();
        let normals: Vec<f64>;
        if input_node.children.len() == 4 {
            normals = Self::extract_f64_array(&input_node.children[3].properties[0]).unwrap_or_default();
        } else {
            normals = vec![];
        }
        
        BlendShape { id, name, indices, vertices, normals }
    }

    fn extract_string(prop: &Property) -> Option<String> {
        match prop {
            fbx::Property::String(value) => {Some(value.to_owned())},
            _ => None
        }
    }

    fn extract_i64(prop: &Property) -> Option<i64> {
        match prop {
            fbx::Property::I64(value) => {Some(value.to_owned())},
            _ => None
        }
    }

    fn extract_f64_array(prop: &Property) -> Option<Vec<f64>> {
        match prop {
            fbx::Property::F64Array(value) => {Some(value.to_owned())},
            fbx::Property::F32Array(value) => {Some(value.iter().map(|&x| x as f64).collect::<Vec<f64>>().to_owned())}
            _ => None
        }
    }

    fn extract_i64_array(prop: &Property) -> Option<Vec<i64>> {
        match prop {
            fbx::Property::I64Array(value) => {Some(value.to_owned())},
            fbx::Property::I32Array(value) => {Some(value.iter().map(|&x| x as i64).collect::<Vec<i64>>().to_owned())}
            _ => None
        }
    }
}

fn unwrap_handler(err: &str) {
    eprintln!("Error: {}", err);
    println!("Press Enter to exit...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read line");
}
