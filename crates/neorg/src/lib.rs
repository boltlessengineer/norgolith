use std::path::{Path, PathBuf};

use norg_rs::target::NorgLinkAppTarget;
use workspace::NeorgWorkspaceManifest;

pub mod workspace;

pub fn get_workspace(path: &Path) -> Option<NeorgWorkspaceManifest> {
    // 1. find for workspace manifest file (`root.toml`) from given path
    // 2. read manifest file and create `NeorgWorkspaceManifest` object
    // 3. return `Option<NeorgWorkspaceManifest>`
    fn find_in_parent_dirs(path: &Path, target_file_name: &str) -> Option<PathBuf> {
        if path.file_name().unwrap_or_default() == target_file_name {
            return Some(path.parent().unwrap().to_path_buf());
        }

        let mut curr = Some(path);

        while let Some(path) = curr {
            let candidate = path.join(target_file_name);
            if std::fs::metadata(&candidate).is_ok() {
                return std::path::absolute(path).ok();
            }
            curr = path.parent();
        }

        None
    }
    if let Some(path) = find_in_parent_dirs(&path, "root.toml") {
        return Some(NeorgWorkspaceManifest::from(path));
    }
    if let Some(path) = find_in_parent_dirs(&path, ".root.toml") {
        return Some(NeorgWorkspaceManifest::from(path));
    }
    return None;
}

// TODO: rename this to glob_docs
// TODO: treat path as relative to current path
pub fn query_docs(self_path: &Path, query: &str) -> Option<Vec<PathBuf>> {
    // TODO: remove these asserts and use AbsPath type instead to ensure path is absolute
    assert!(self_path.is_absolute());
    let workspace = get_workspace(self_path)?;
    let root = workspace.path;
    let query = root.join(query);
    Some(
        glob::glob(query.to_str().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|path| path.extension().is_some_and(|ext| ext == "norg"))
            .filter(|path| *path != self_path)
            .collect(),
    )
}

pub struct Location {
    pub path: PathBuf,
    // pub range: Range,
}

#[derive(Debug)]
pub enum ResolveAppTargetError {
    NoWorkspaceAt(PathBuf),
    NoExternalWorkspace(String),
}

/// resolve `NorgLinkAppTarget` to `Location`
pub fn resolve_target(
    origin_path: &Path,
    target: &NorgLinkAppTarget,
) -> Result<Location, ResolveAppTargetError> {
    assert!(origin_path.is_absolute());
    let path = match &target.workspace {
        // {:$name:foo}
        Some(name) => {
            let origin_workspace = get_workspace(&origin_path)
                .ok_or(ResolveAppTargetError::NoWorkspaceAt(origin_path.into()))?;
            let target_workspace = origin_workspace
                .get_external_workspace_by_name(&name)
                .ok_or(ResolveAppTargetError::NoExternalWorkspace(name.to_string()))?;
            let stripped_path = target.path.strip_prefix("/").unwrap();
            target_workspace.path.join(stripped_path)
        }
        // {:/foo}
        None if target.path.is_absolute() => {
            let origin_workspace = get_workspace(&origin_path)
                .ok_or(ResolveAppTargetError::NoWorkspaceAt(origin_path.into()))?;
            let stripped_path = target.path.strip_prefix("/").unwrap();
            origin_workspace.path.join(stripped_path)
        }
        // {:foo}
        None => origin_path.parent().unwrap().join(&target.path),
    };
    // let range =
    Ok(Location { path })
}

pub fn export_linkable_href(origin_path: &Path, target: NorgLinkAppTarget) -> String {
    assert!(origin_path.is_absolute());
    let workspace = get_workspace(&origin_path).unwrap().path;
    let location = resolve_target(origin_path, &target).unwrap();
    let is_index = location
        .path
        .file_stem()
        .is_some_and(|name| name == "index");
    let target_path = if is_index {
        location.path.parent().unwrap().to_path_buf()
    } else {
        location.path
    };
    target_path
        .strip_prefix(workspace)
        .map(|p| PathBuf::from("/").join(p))
        .unwrap_or(target_path)
        .to_string_lossy()
        .to_string()
}

pub fn create_app_target(origin_path: &Path, path: &Path) -> NorgLinkAppTarget {
    assert!(origin_path.is_absolute());
    assert!(path.is_absolute());
    let cwd = origin_path.parent().unwrap();
    let path = path.with_extension("");
    if let Ok(path) = path.strip_prefix(cwd) {
        return NorgLinkAppTarget {
            workspace: None,
            path: path.to_path_buf(),
            scopes: vec![],
        };
    }
    let workspace = get_workspace(origin_path).unwrap();
    if let Ok(path) = path.strip_prefix(workspace.path) {
        return NorgLinkAppTarget {
            workspace: None,
            path: PathBuf::from("/").join(path),
            scopes: vec![],
        };
    }
    // TODO: find for external workspace from workspace manifest
    let _ext_workspace = get_workspace(&path).unwrap();
    todo!("path is outside of current workspace")
}

pub fn parse_file(path: &Path) -> norg_rs::parser::NorgAST {
    let bytes = std::fs::read(path).unwrap();
    norg_rs::parser::parse(&bytes)
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_query_docs() {
        let query = "posts/*";
        let path = std::path::absolute("../../my-site/content/posts/index.norg").unwrap();
        let docs = query_docs(&path, query);
        assert_eq!(
            docs,
            Some(vec![
                PathBuf::from("posts/building-a-cool-note-app.norg"),
                PathBuf::from("posts/desk-setup-2025.norg"),
            ])
        );
    }

    #[test]
    fn test_export_linkables_href() {
        let href = export_linkable_href(
            &std::path::absolute("../../my-site/content/posts/index.norg").unwrap(),
            NorgLinkAppTarget {
                workspace: None,
                path: PathBuf::from("desk-setup-2025"),
                scopes: vec![],
            },
        );
        assert_eq!(&href, "/posts/desk-setup-2025",);
        let href = export_linkable_href(
            &std::path::absolute("../../my-site/content/posts/index.norg").unwrap(),
            NorgLinkAppTarget {
                workspace: None,
                path: PathBuf::from("desk-setup-2025"),
                scopes: vec![],
            },
        );
        assert_eq!(&href, "/posts/desk-setup-2025",);
    }

    #[test]
    fn test_create_app_target() {
        let target = create_app_target(
            &std::path::absolute("../../my-site/content/posts/index.norg").unwrap(),
            &std::path::absolute("../../my-site/content/posts/desk-setup-2025.norg").unwrap(),
        );
        assert_eq!(
            target,
            NorgLinkAppTarget {
                workspace: None,
                path: PathBuf::from("desk-setup-2025"),
                scopes: vec![],
            }
        );
    }
}
