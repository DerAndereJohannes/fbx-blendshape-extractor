use std::{io::{BufWriter, Write}, fs::File};

use serde::Serialize;

use fbx::{self, Node, Property};

fn main() {
    let file = std::fs::File::open("sample.fbx").expect("Failed to open file");

    let reader = std::io::BufReader::new(file);
    let file = fbx::File::read_from(reader).unwrap();
    
    let emaciated_geometry: Vec<&Node> = file.children[8].children.iter().filter(|x| {if let Property::String(name) = &x.properties[1] {name.contains("BodyHeavy") && x.name == "Geometry"} else {false}}).collect::<Vec<&Node>>();
    
    let main_mesh: &Node = &file.children[8].children[0];

    println!("{:?}", main_mesh.children[0].properties);

    match &main_mesh.children[0].properties[0] {
        Property::F64Array(a) => {println!("{:?}", a.len());},
        _ => {}
    }

    let mut new_blendshapes: Vec<BlendShape> = vec![];

    for n in emaciated_geometry.iter() {
        new_blendshapes.push(BlendShape::new(n))
    }

    let blendshape_holder:BlendShapes = BlendShapes { shapes: new_blendshapes };

    let output_file = File::create("sample.json").unwrap();
    let mut writer = BufWriter::new(output_file);
    serde_json::to_writer(&mut writer, &blendshape_holder).unwrap();
    writer.flush().unwrap();
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
        let indices: Vec<i64> = Self::extract_i64_array(&input_node.children[1].properties[0]).unwrap();
        let vertices: Vec<f64> = Self::extract_f64_array(&input_node.children[2].properties[0]).unwrap();
        let normals: Vec<f64> = Self::extract_f64_array(&input_node.children[3].properties[0]).unwrap();
        
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

