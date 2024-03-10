use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub enum Access {
    #[serde(alias = "r", alias = "R", alias = "read")]
    Read,
    #[serde(alias = "w", alias = "W", alias = "write")]
    Write,
    #[serde(
        alias = "rw",
        alias = "wr",
        alias = "RW",
        alias = "WR",
        alias = "readwrite"
    )]
    ReadWrite,
}

impl Default for Access {
    fn default() -> Self {
        Self::ReadWrite
    }
}
