use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct Params {
    scale: Option<u32>,
}

impl Params {
    pub fn get_scale(&self) -> Option<u32> {
        self.scale
    }
}