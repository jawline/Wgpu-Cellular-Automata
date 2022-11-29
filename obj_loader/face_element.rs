use std::error::Error;

#[derive(Debug)]
pub struct FaceElement {
    pub vertex: usize,
    pub texture_coordinate: Option<usize>,
    pub normal: Option<usize>,
}

impl FaceElement {
    pub fn of_string(line: &str) -> Result<FaceElement, Box<dyn Error>> {
        let mut parts = line.split('/');

        let vertex = parts
            .next()
            .ok_or("face element is not in the form vertex/texture_coord/normal")?
            .parse::<usize>()?;
        let texture_coordinate = parts.next();
        let normal = parts.next();

        let (texture_coordinate, normal) = match (texture_coordinate, normal) {
            (Some(texture_coordinate), Some(normal)) => {
                let texture_coordinate = match texture_coordinate {
                    "" => None,
                    v => Some(v.parse::<usize>()?),
                };

                let normal = match normal {
                    "" => None,
                    v => Some(v.parse::<usize>()?),
                };
                (texture_coordinate, normal)
            }
            _ => (None, None),
        };

        Ok(Self {
            vertex: vertex,
            texture_coordinate,
            normal,
        })
    }
}

pub type Face = Vec<FaceElement>;
