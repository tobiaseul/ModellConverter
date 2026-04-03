/// The firmware format of a model file.
#[derive(Clone, Debug, PartialEq)]
pub enum Format {
    Edgetx,
    Ethos,
    JetiDuplex,
}
