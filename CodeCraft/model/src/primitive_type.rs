#[derive(Clone, Debug, PartialEq, Eq, Hash, trans::Trans)]
pub enum PrimitiveType {
    Lines,
    Triangles,
}
