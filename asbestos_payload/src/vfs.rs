use std::{
    borrow::Cow,
    ffi::OsString,
    fmt,
    fmt::Write,
    io,
    path::{Component, Path, PathBuf},
};

use asbestos_shared::{
    log_trace,
    protocol::{MappingFrom, MappingKind, MappingTo, Mappings},
};

use super::{PipeConnection, MAPPINGS};

/// Turn a 'virtual' path into a real one, as determined by `MAPPINGS`.
///
/// `path` should be a canonical path. This is in part because `Path`'s `PartialEq` does a component-wise comparison
///  and because the path resolving algorithm shouldn't have to deal with relative path components.
pub(crate) fn resolve_path<'a>(
    conn: Option<&mut PipeConnection>,
    path: &'a Path,
) -> Result<Cow<'a, Path>, PathResolveError> {
    let mappings = MAPPINGS.lock().unwrap();

    let path = _resolve_path(conn, path, &mappings);

    path
}

/// The *real* path resolution function.
///
/// This is separtated out for the sake of testability.
pub fn _resolve_path<'a>(
    conn: Option<&mut PipeConnection>,
    path: &'a Path,
    mappings: &Mappings,
) -> Result<Cow<'a, Path>, PathResolveError> {
    let mut is_nt_wierd = false;
    let mut is_simplified = false;
    let simplified_path = {
        let mut components = path.components();
        let nt_wierd_components = [Component::RootDir, Component::Normal("??".as_ref())];
        let mut nt_wierd_components = nt_wierd_components.iter().copied();
        let nt_wierd_stripped = {
            if components.next() == nt_wierd_components.next() {
                if components.next() == nt_wierd_components.next() {
                    components.as_path()
                } else {
                    path
                }
            } else {
                path
            }
        };
        if nt_wierd_stripped != path {
            is_nt_wierd = true;
            nt_wierd_stripped
        } else {
            let post_dunce = dunce::simplified(nt_wierd_stripped);
            is_simplified = post_dunce != path;
            post_dunce
        }
    };

    let mut current_path = Cow::Borrowed(simplified_path);

    let mut trace = String::new();

    if conn.is_some() {
        write!(
            trace,
            r#"Determining redirect for "{}""#,
            current_path.display()
        )
        .ok();
    }

    for mapping in mappings.iter() {
        match mapping.kind {
            MappingKind::Redirect => match (&mapping.from, &mapping.to) {
                (MappingFrom::File(from), MappingTo::File(to)) => {
                    if &current_path == from {
                        current_path = to.to_owned().into();
                    }
                }
                (MappingFrom::File(from), MappingTo::Folder(to)) => {
                    if current_path == *from {
                        if let Some(name) = current_path.file_name() {
                            current_path = to.join(name).into();
                        } else {
                            return Err(PathResolveError::InvalidMapping);
                        }
                    }
                }
                (MappingFrom::Folder(from), MappingTo::Folder(to)) => {
                    if current_path.ancestors().any(|anc| anc == from) {
                        let relative = current_path.strip_prefix(&from).unwrap();
                        current_path = to.join(relative).into();
                    }
                }
                (MappingFrom::Folder(_), MappingTo::File(_)) => {
                    return Err(PathResolveError::InvalidMapping);
                }
            },
            MappingKind::Mount => match (&mapping.from, &mapping.to) {
                (MappingFrom::File(from), MappingTo::Folder(to)) => {
                    if current_path.file_name() == from.file_name()
                        && current_path.parent() == Some(to)
                    {
                        current_path = from.to_owned().into();
                    }
                }
                (MappingFrom::Folder(from), MappingTo::Folder(to)) => {
                    if current_path.ancestors().any(|anc| anc == to) {
                        let relative = current_path.strip_prefix(&to).unwrap();
                        current_path = from.join(relative).into();
                    }
                }
                (MappingFrom::File(_), MappingTo::File(_))
                | (MappingFrom::Folder(_), MappingTo::File(_)) => {
                    return Err(PathResolveError::InvalidMapping);
                }
            },
        }

        if conn.is_some() {
            write!(
                trace,
                r#"{}current_path = "{}""#,
                "\n",
                current_path.display()
            )
            .ok();
        }
    }

    if let Some(conn) = conn {
        log_trace!(conn, "{}", trace).ok();
    }

    if is_nt_wierd {
        let mut out = OsString::from(r"\??\");
        out.push(current_path.as_ref());
        Ok(PathBuf::from(out).into())
    } else if is_simplified {
        let mut out = OsString::from(r"\\?\");
        out.push(current_path.as_ref());
        Ok(PathBuf::from(out).into())
    } else {
        Ok(current_path)
    }
}

#[derive(Debug)]
pub enum PathResolveError {
    Detour(detour::Error),
    Io(io::Error),
    InvalidMapping,
}

impl fmt::Display for PathResolveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Detour(err) => err.fmt(f),
            Self::Io(err) => err.fmt(f),
            Self::InvalidMapping => write!(f, "Incalid VFS mapping"),
        }
    }
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
