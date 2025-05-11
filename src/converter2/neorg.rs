use norg_rs::{export::Exporter, target::NorgLinkAppTarget};

pub(crate) fn create_exporter() -> Exporter {
    use janetrs::Janet;

    /// (neorg/get-workspace path)
    /// test api
    #[janetrs::janet_fn]
    fn neorg_get_workspace(args: &mut [Janet]) -> Janet {
        use janetrs::JanetArgs as _;
        use janetrs::{JanetType, TaggedJanet};
        use std::path::PathBuf;

        let path = match args.get_tagged_matches(0, &[JanetType::Buffer, JanetType::String]) {
            TaggedJanet::Buffer(b) => PathBuf::from(&b.to_os_str_lossy()),
            TaggedJanet::String(s) => PathBuf::from(&s.to_os_str_lossy()),
            _ => unreachable!("Already checked to be a buffer|string"),
        };
        let Some(workspace) = neorg::get_workspace(&path) else {
            return Janet::nil();
        };

        workspace.into()
    }

    /// (neorg/glob-docs path query)
    #[janetrs::janet_fn]
    fn neorg_glob_docs(args: &mut [Janet]) -> Janet {
        use janetrs::JanetArgs as _;
        use janetrs::{JanetType, TaggedJanet};
        use std::path::PathBuf;

        let path = match args.get_tagged_matches(0, &[JanetType::Buffer, JanetType::String]) {
            TaggedJanet::Buffer(b) => PathBuf::from(&b.to_os_str_lossy()),
            TaggedJanet::String(s) => PathBuf::from(&s.to_os_str_lossy()),
            _ => unreachable!("Already checked to be a buffer|string"),
        };
        let query = match args.get_tagged_matches(1, &[JanetType::Buffer, JanetType::String]) {
            TaggedJanet::Buffer(b) => b.to_string(),
            TaggedJanet::String(s) => s.to_string(),
            _ => unreachable!("Already checked to be a buffer|string"),
        };
        let Some(docs) = neorg::glob_docs(&path, &query) else {
            println!("something went wrong. can't find workspace");
            return Janet::nil();
        };
        Janet::tuple(
            docs.into_iter()
                .map(|path| janetrs::JanetString::new(path.to_str().unwrap()))
                .collect(),
        )
    }

    /// (_neorg/export/linkable-href path app-target)
    #[janetrs::janet_fn]
    fn neorg_export_linkable_href(args: &mut [Janet]) -> Janet {
        use janetrs::JanetArgs as _;
        use janetrs::{JanetType, TaggedJanet};
        use std::path::PathBuf;

        let path = match args.get_tagged_matches(0, &[JanetType::Buffer, JanetType::String]) {
            TaggedJanet::Buffer(b) => PathBuf::from(&b.to_os_str_lossy()),
            TaggedJanet::String(s) => PathBuf::from(&s.to_os_str_lossy()),
            _ => unreachable!("Already checked to be a buffer|string"),
        };
        let target = NorgLinkAppTarget::try_from(args[1]).unwrap();
        Janet::string(neorg::export_linkable_href(&path, target).into())
    }

    #[janetrs::janet_fn]
    fn neorg_create_app_target(args: &mut [Janet]) -> Janet {
        use janetrs::JanetArgs as _;
        use janetrs::{JanetType, TaggedJanet};
        use std::path::PathBuf;

        let self_path = match args.get_tagged_matches(0, &[JanetType::Buffer, JanetType::String]) {
            TaggedJanet::Buffer(b) => PathBuf::from(&b.to_os_str_lossy()),
            TaggedJanet::String(s) => PathBuf::from(&s.to_os_str_lossy()),
            _ => unreachable!("Already checked to be a buffer|string"),
        };
        let path = match args.get_tagged_matches(1, &[JanetType::Buffer, JanetType::String]) {
            TaggedJanet::Buffer(b) => PathBuf::from(&b.to_os_str_lossy()),
            TaggedJanet::String(s) => PathBuf::from(&s.to_os_str_lossy()),
            _ => unreachable!("Already checked to be a buffer|string"),
        };
        let target = neorg::create_app_target(&self_path, &path);
        Janet::structs(target.into())
    }

    #[janetrs::janet_fn]
    fn neorg_parse_file(args: &mut [Janet]) -> Janet {
        use janetrs::JanetArgs as _;
        use janetrs::{JanetType, TaggedJanet};
        use std::path::PathBuf;

        let path = match args.get_tagged_matches(0, &[JanetType::Buffer, JanetType::String]) {
            TaggedJanet::Buffer(b) => PathBuf::from(&b.to_os_str_lossy()),
            TaggedJanet::String(s) => PathBuf::from(&s.to_os_str_lossy()),
            _ => unreachable!("Already checked to be a buffer|string"),
        };
        let ast = neorg::parse_file(&path);
        Janet::structs(ast.into())
    }

    let mut exporter = Exporter::new();

    #[rustfmt::skip]
    exporter.with_janet(|janet| {
        use janetrs::env::CFunOptions;

        janet.add_c_fn(CFunOptions::new(c"neorg/get-workspace", neorg_get_workspace_c));
        janet.add_c_fn(CFunOptions::new(c"neorg/glob-docs", neorg_glob_docs_c));
        janet.add_c_fn(CFunOptions::new(c"neorg/create-app-target", neorg_create_app_target_c));
        janet.add_c_fn(CFunOptions::new(c"_neorg/export/linkable-href", neorg_export_linkable_href_c));
        janet.add_c_fn(CFunOptions::new(c"neorg/parse-file", neorg_parse_file_c));

        janet
            .run_bytes(include_bytes!("../resources/janet/neorg.janet"))
            .unwrap();
    });

    exporter
}

#[cfg(test)]
mod test {
    use janetrs::env::DefOptions;

    use super::*;

    #[test]
    fn test_neorg_env() {
        let mut exporter = create_exporter();
        let val = exporter.with_janet(|janet| {
            let path = std::path::absolute("my-site/content/posts/index.norg").unwrap();
            janet.add_def(DefOptions::new("path", path.to_str().unwrap()));
            janet.run(r#" (neorg/get-workspace path) "#)
        });
        assert_eq!(
            val,
            Ok(neorg::workspace::NeorgWorkspaceManifest::from(
                std::path::absolute("my-site/content").unwrap(),
            )
            .into())
        );
    }

    // #[test]
    // fn test_ul_docs_macro() {
    //     let (html, _ctx) = convert(
    //         &std::path::absolute("my-site/content/posts/index.norg").unwrap(),
    //         ".ul-docs posts/*\n".as_bytes(),
    //         "example.com",
    //     );
    //     dbg!(html);
    // }
}
