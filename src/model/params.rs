use serde::Deserialize;

#[derive(Deserialize)]
pub struct Params {
    scale: Option<u32>,
}

impl Params {
    pub fn get_scale(&self) -> Option<u32> {
        self.scale
    }
}