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
            // .map(|path| {
            //     path.strip_prefix(&root)
            //         .map(Path::to_path_buf)
            //         .unwrap_or(path)
            // })
            .collect(),
    )
}

pub fn export_linkable_href(self_path: &Path, target: NorgLinkAppTarget) -> String {
    assert!(self_path.is_absolute());
    let cwd = self_path.parent().unwrap();
    let workspace = if let Some(_name) = target.workspace {
        todo!("handle external workspace")
    } else if let Some(workspace) = get_workspace(&self_path) {
        workspace.path
    } else {
        std::env::current_dir().unwrap()
    };
    let target_path = cwd.join(&target.path);
    let target_path = match target_path.strip_prefix(workspace) {
        Ok(p) => PathBuf::from("/").join(p),
        Err(_) => target_path,
    };
    target_path.to_string_lossy().to_string()
}

pub fn create_app_target(self_path: &Path, path: &Path) -> NorgLinkAppTarget {
    assert!(self_path.is_absolute());
    assert!(path.is_absolute());
    let cwd = self_path.parent().unwrap();
    let path = path.with_extension("");
    if let Ok(path) = path.strip_prefix(cwd) {
        return NorgLinkAppTarget {
            workspace: None,
            path: path.to_path_buf(),
            scopes: vec![],
        };
    }
    let workspace = get_workspace(self_path).unwrap();
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
        assert_eq!(target, NorgLinkAppTarget {
            workspace: None,
            path: PathBuf::from("desk-setup-2025"),
            scopes: vec![],
        });
    }
}
