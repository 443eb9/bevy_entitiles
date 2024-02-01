use std::path::{Component, Path, PathBuf};

pub trait AssetPath {
    fn to_asset_path(&self) -> PathBuf;
}

impl AssetPath for Path {
    fn to_asset_path(&self) -> PathBuf {
        to_asset_path(self)
    }
}

impl AssetPath for PathBuf {
    fn to_asset_path(&self) -> PathBuf {
        to_asset_path(self)
    }
}

/// Converts a path to an asset path.
/// 
/// # Example
/// ```rust
/// assert_eq!(to_asset_path("C:\\Project\\assets\\project\\../test_image.png", "test_image.png"));
/// ```
pub fn to_asset_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();

    let mut p = Vec::new();
    path.components().for_each(|comp| match comp {
        Component::ParentDir => {
            p.pop();
        }
        Component::Normal(c) => {
            if c == "assets" {
                p.clear();
            } else {
                p.push(c);
            }
        }
        _ => {}
    });
    let result = p.into_iter().fold(PathBuf::new(), |mut path, comp| {
        path.push(comp);
        path
    });
    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_asset_path() {
        dbg!(to_asset_path(
            "C:\\Project\\assets\\project\\../test_image.png"
        ));
    }
}
