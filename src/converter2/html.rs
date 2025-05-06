use std::path::Path;

use norg_rs::export::{ExportCtx, ExportMeta, ExportTarget};

pub fn convert(doc_path: &Path, doc_content: &[u8], _root_url: &str) -> (String, ExportMeta) {
    let ast = norg_rs::parser::parse(doc_content);
    let ctx = ExportCtx {
        path: doc_path.to_path_buf(),
    };

    let mut exporter = crate::converter2::neorg::create_exporter();
    let res = exporter.export(ExportTarget::Html, ast, Some(ctx)).unwrap();

    res
}
