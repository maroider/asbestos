use std::{borrow::Cow, io, path::Path};

use asbestos_shared::protocol::{MappingKind, Mappings};

use super::MAPPINGS;

/// Turn a 'virtual' path into a real one, as determined by `MAPPINGS`.
///
/// `path` should be a canonical path. This is in part because `Path`'s `PartialEq` does a component-wise comparison
///  and because the path resolving algorithm shouldn't have to deal with relative path components.
pub(crate) fn resolve_path(path: &Path) -> Result<Cow<Path>, PathResolveError> {
    let mappings = MAPPINGS.lock().unwrap();

    let path = _resolve_path(path, &mappings);

    path
}

//

/// The *real* path resolution function.
///
/// This is separtated out for the sake of testability.
pub fn _resolve_path<'a>(
    path: &'a Path,
    mappings: &Mappings,
) -> Result<Cow<'a, Path>, PathResolveError> {
    let mut current_path = Cow::Borrowed(path);

    for mapping in mappings.iter() {
        match mapping.kind {
            MappingKind::Redirect => {
                if current_path.starts_with(&mapping.from) {
                    current_path = mapping
                        .to
                        // Unwrapping here is fine since `mapping.from` is a prefix of `current_path` in this case.
                        .join(current_path.strip_prefix(&mapping.from).unwrap())
                        .into();
                }
            }
            MappingKind::Mount => {
                if current_path.parent() == Some(&mapping.to) {
                    current_path = mapping.from.clone().into();
                } else if current_path
                    .ancestors()
                    .skip(1)
                    .any(|curr_anc| curr_anc.parent() == Some(&mapping.to))
                {
                    // Unwrapping here is fine since `mapping.to` is a prefix of `current_path` in this case.
                    let relative = current_path.strip_prefix(&mapping.to).unwrap();
                    if let Some(mounted_dir_name) = mapping.from.file_name() {
                        if relative.starts_with(mounted_dir_name) {
                            // Unwrapping here is fine since `mounted_dir_name` is a prefix of `relative` in this case.
                            let relative = relative.strip_prefix(mounted_dir_name).unwrap();
                            current_path = mapping.from.join(relative).into();
                        }
                    } else {
                        // This branch should only be run when `mapping.from` only consists of a
                        // `std::path::Component::Prefix` and/or a `std::path::Component::RootDir`.
                    }
                }
            }
        }
    }

    Ok(current_path)
}

#[derive(Debug)]
pub enum PathResolveError {
    Detour(detour::Error),
    Io(io::Error),
}

impl From<detour::Error> for PathResolveError {
    fn from(from: detour::Error) -> Self {
        Self::Detour(from)
    }
}

impl From<io::Error> for PathResolveError {
    fn from(from: io::Error) -> Self {
        Self::Io(from)
    }
}
