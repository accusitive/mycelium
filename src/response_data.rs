use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ResponseData {
    pub version: Version,
    pub players: Players,
    pub description: Description,
    pub favicon: Option<String>
}

#[derive(Debug, Serialize)]
pub struct Version {
    pub name: String,
    pub protocol: u32
}

#[derive(Debug, Serialize)]
pub struct Players {
    pub max: i32, 
    pub online: i32,
    pub sample: Vec<Sample>
}
#[derive(Debug, Serialize)]
pub struct Sample {
    pub name: String,
    pub id: String
}
#[derive(Debug, Serialize)]
pub struct Description {
    pub text: String
}