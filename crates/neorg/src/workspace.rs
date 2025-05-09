use std::path::PathBuf;

#[derive(Debug)]
pub struct NeorgWorkspaceManifest {
    // TODO: force to use absolute path instead
    pub path: PathBuf,
}

impl NeorgWorkspaceManifest {
    pub fn get_external_workspace_by_name(&self, _name: &str) -> Option<Self> {
        todo!("get external workspace from given name")
    }
}

impl From<PathBuf> for NeorgWorkspaceManifest {
    fn from(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Into<janetrs::JanetStruct<'_>> for NeorgWorkspaceManifest {
    fn into(self) -> janetrs::JanetStruct<'static> {
        janetrs::JanetStruct::builder(1)
            .put(
                janetrs::JanetKeyword::new("path"),
                janetrs::JanetString::new(self.path.to_str().unwrap()),
            )
            .finalize()
    }
}
impl Into<janetrs::Janet> for NeorgWorkspaceManifest {
    fn into(self) -> janetrs::Janet {
        let s: janetrs::JanetStruct = self.into();
        s.into()
    }
}
