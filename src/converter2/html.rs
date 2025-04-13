use std::path::Path;

use norg_rs::export::{ExportCtx, ExportMeta, ExportTarget, Exporter};

pub fn convert(doc_path: &Path, document: &[u8], _root_url: &str) -> (String, ExportMeta) {
    let ast = norg_rs::parser::parse(document);
    let ctx = ExportCtx {
        path: doc_path.to_path_buf(),
    };

    let mut exporter = Exporter::new();
    let res = exporter.export(ExportTarget::Html, ast, Some(ctx)).unwrap();

    res
}
