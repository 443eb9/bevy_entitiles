pub trait TiledObject {
    fn spawn();
}

pub trait TiledEnum {
    fn get_identifier(ident: &str) -> Self;
}
