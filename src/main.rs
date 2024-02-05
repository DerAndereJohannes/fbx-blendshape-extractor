use std::{io::{BufWriter, Write}, fs::File, collections::HashMap};

use serde::{Serialize, Deserialize};

use fbx::{self, Node, Property};

fn main() {
    // Define constants
    // const NEWSHAPENAME: &'static str = "cprint";
    const FBXFILENAME: &'static str = "sample.fbx";
    const VERTEXFILENAME: &'static str = "sample-verts.json";
    const NEWSHAPENAME: &'static str = "BodyHeavy";

    // Load the vertex file
    let file = std::fs::File::open(VERTEXFILENAME).expect("Failed to open file");
    let reader = std::io::BufReader::new(file);
    let render_meshes: RendererMeshes = serde_json::from_reader(reader).unwrap();


    // load the fbx file
    let file = std::fs::File::open(FBXFILENAME).expect("Failed to open file");
    let reader = std::io::BufReader::new(file);
    let file = fbx::File::read_from(reader).unwrap();

    // Extract wanted blendshapes from the file
    let mut new_blendshapes: Vec<BlendShape> = vec![];
    file.children[8].children.iter()
                             .filter(|x| {
                                 if let Property::String(name) = &x.properties[1] {
                                     name.contains(NEWSHAPENAME) && 
                                         x.name == "Geometry" && 
                                         !name.contains("cbs") && 
                                         !name.contains("Tear") && 
                                         !name.contains("Mouth") && 
                                         !name.contains("Eyelashes") && 
                                         !name.contains("Eyes")
                                 } else {
                                     false
                                 }
                             }).for_each(|x| new_blendshapes.push(BlendShape::new(x)));

    let mut blendshape_holder:BlendShapes = BlendShapes { shapes: new_blendshapes };

    // create the new blendshape vertices and indices
    for shape in blendshape_holder.shapes.iter_mut() {
        let related_geometry = render_meshes.meshes.iter().find(|x| shape.name.contains(&x.name.replace(".Shape", ""))).unwrap();
        let mut new_verts: Vec<f64> = Vec::with_capacity(related_geometry.vertices.len() - *shape.indices.last().unwrap() as usize);

        let highest_index: usize = *shape.indices.last().unwrap() as usize;

        let matching = find_matching_indices(&related_geometry.vertices[..=highest_index], &related_geometry.vertices[(highest_index + 1)..]);

        for i in 0..new_verts.len() {
            new_verts[i] = shape.vertices[*matching.get(&(i as i64)).unwrap().first().unwrap() as usize];
        }

        shape.vertices.append(&mut new_verts);
        shape.indices.extend::<Vec<i64>>((highest_index as i64..related_geometry.vertices.len() as i64).collect());
    }

    // export new file
    let output_file = File::create("sample_new.json").unwrap();
    let mut writer = BufWriter::new(output_file);
    serde_json::to_writer(&mut writer, &blendshape_holder).unwrap();
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


#[derive(Deserialize, Debug, Default, PartialEq)]
struct Vector3 {
    x: f64,
    y: f64,
    z: f64
}

#[derive(Serialize, Debug, Default)]
struct BlendShapes {
    shapes: Vec<BlendShape>
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

    pub fn new(input_node: &Node) -> BlendShape {
        let id: i64 = Self::extract_i64(&input_node.properties[0]).unwrap();
        let name: String = Self::extract_string(&input_node.properties[1]).unwrap().replace("\0\u{1}Geometry", "");
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

