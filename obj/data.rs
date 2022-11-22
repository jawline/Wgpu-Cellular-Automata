use super::face_element::{Face, FaceElement};
use glam::{Vec3, Vec4};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct Data {
    pub vertices: Vec<Vec4>,
    pub texture_coordinates: Vec<Vec3>,
    pub vertex_normals: Vec<Vec3>,
    pub faces: Vec<Face>,
}

impl Data {
    fn read_line(&mut self, line: &str) -> Result<(), Box<dyn Error>> {
        if line == "" {
            return Ok(());
        }
        let mut parts = line.trim().split(' ');

        match parts.next() {
            Some("v") => {
                /* Vertex */
                let mut parts = parts.map(|x| x.parse::<f32>());
                let x = parts.next().ok_or::<Box<dyn Error>>(
                    "obj v is not followed by three or four floats".into(),
                )??;
                let y = parts.next().ok_or::<Box<dyn Error>>(
                    "obj v is not followed by three or four floats".into(),
                )??;
                let z = parts.next().ok_or::<Box<dyn Error>>(
                    "obj v is not followed by three or four floats".into(),
                )??;

                let w = parts.next().unwrap_or(Ok(1.0))?;

                self.vertices.push(Vec4::new(x, y, z, w));
                Ok(())
            }
            Some("tc") => {
                /* Texture coordinate */
                let mut parts = parts.map(|x| x.parse::<f32>());
                let u = parts.next().ok_or::<Box<dyn Error>>(
                    "obj tc is not followed by one, two or three floats".into(),
                )??;

                let v = parts.next().unwrap_or(Ok(0.0))?;
                let w = parts.next().unwrap_or(Ok(0.0))?;

                self.texture_coordinates.push(Vec3::new(u, v, w));

                Ok(())
            }
            Some("vn") => {
                /* Vertex normal */

                let mut parts = parts.map(|x| x.parse::<f32>());

                let x = parts
                    .next()
                    .ok_or::<Box<dyn Error>>("obj vn is not followed by three floats".into())??;

                let y = parts
                    .next()
                    .ok_or::<Box<dyn Error>>("obj vn is not followed by three floats".into())??;

                let z = parts
                    .next()
                    .ok_or::<Box<dyn Error>>("obj vn is not followed by three floats".into())??;

                self.vertex_normals.push(Vec3::new(x, y, z));

                Ok(())
            }
            Some("vp") => {
                /* Parameter space vertices (TODO) */
                panic!("I don't know this format");
            }
            Some("f") => {
                /* Face */
                let mut face_elements = Vec::new();
                for part in parts {
                    let new_face = FaceElement::of_string(part)?;
                    face_elements.push(new_face);
                }
                self.faces.push(face_elements);
                Ok(())
            }
            Some("l") => panic!("polylines are unsupported"),
            Some("#") => {
                /* Comment line */
                Ok(())
            }
            Some("mtllib") =>
            /* TODO: Support materials */
            {
                Ok(())
            }
            Some("usemtl") =>
            /* TODO: Support materials */
            {
                Ok(())
            }
            Some("g") =>
            /* TODO: Support groups */
            {
                Ok(())
            }
            Some("s") =>
            /* TODO: Support smooth shading */
            {
                Ok(())
            }
            Some(part) => Err(format!(
                "obj in bad format: {} {:?}",
                part,
                parts.collect::<Vec<&str>>()
            )
            .into()),
            None => {
                /* Empty line */
                Ok(())
            }
        }
    }

    pub fn from_file(filepath: &str) -> Result<Self, Box<dyn Error>> {
        let mut obj_data = Self {
            vertices: Vec::new(),
            texture_coordinates: Vec::new(),
            vertex_normals: Vec::new(),
            faces: Vec::new(),
        };

        for line in BufReader::new(File::open(filepath)?).lines() {
            obj_data.read_line(&line?)?;
        }

        Ok(obj_data)
    }
}
