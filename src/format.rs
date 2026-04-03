/// The firmware format of a model file.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Format {
    Edgetx,
    Ethos,
    JetiDuplex,
}
