//! Ответ PROPFIND: `207 Multi-Status` со списком ресурсов.
//!
//! XML собирается руками: форма ответа фиксированная, а тащить ради неё
//! генератор незачем. Экранируется всё, что уходит внутрь тегов, — имена
//! файлов приходят от админа и могут содержать `&` или `<`.

use super::tree::Node;

/// Один ресурс в ответе.
pub struct Resource {
    /// Путь от корня сборки, без ведущего слэша.
    pub path: String,
    pub is_dir: bool,
    pub size: i64,
    pub etag: String,
}

impl Resource {
    pub fn from_node(parent: &str, node: Node) -> Self {
        let path = if parent.is_empty() {
            node.name
        } else {
            format!("{parent}/{}", node.name)
        };
        Self {
            path,
            is_dir: node.is_dir,
            size: node.size,
            etag: node.sha1,
        }
    }
}

/// `base` — префикс URL вида `/dav/{build_id}`.
pub fn multistatus(base: &str, resources: &[Resource]) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="utf-8"?><D:multistatus xmlns:D="DAV:">"#,
    );
    for res in resources {
        xml.push_str(&response(base, res));
    }
    xml.push_str("</D:multistatus>");
    xml
}

fn response(base: &str, res: &Resource) -> String {
    let href = if res.path.is_empty() {
        format!("{base}/")
    } else if res.is_dir {
        format!("{base}/{}/", encode_path(&res.path))
    } else {
        format!("{base}/{}", encode_path(&res.path))
    };

    // Для каталога отдаём resourcetype=collection и не отдаём длину: Finder
    // иначе считает папку файлом нулевого размера.
    let props = if res.is_dir {
        "<D:resourcetype><D:collection/></D:resourcetype>".to_string()
    } else {
        format!(
            "<D:resourcetype/><D:getcontentlength>{}</D:getcontentlength>\
             <D:getetag>&quot;{}&quot;</D:getetag>\
             <D:getcontenttype>application/octet-stream</D:getcontenttype>",
            res.size,
            escape(&res.etag)
        )
    };

    format!(
        "<D:response><D:href>{}</D:href><D:propstat><D:prop>{}</D:prop>\
         <D:status>HTTP/1.1 200 OK</D:status></D:propstat></D:response>",
        escape(&href),
        props
    )
}

/// Percent-encoding для сегментов пути; слэши остаются разделителями.
fn encode_path(path: &str) -> String {
    path.split('/')
        .map(|segment| urlencoding::encode(segment).into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn escape(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
#[path = "propfind_tests.rs"]
mod tests;
