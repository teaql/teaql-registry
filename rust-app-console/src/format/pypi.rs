use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyPiFileEntry {
    pub filename: String,
    pub url: String,
    pub sha256: String,
}

pub fn generate_pypi_simple_package_html(project_name: &str, files: &[PyPiFileEntry]) -> String {
    let mut links = String::new();
    for f in files {
        links.push_str(&format!(
            "    <a href=\"{}\" data-requires-python=\"\" data-sha256=\"{}\">{}</a><br/>\n",
            f.url, f.sha256, f.filename
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html>
  <head>
    <title>Links for {}</title>
  </head>
  <body>
    <h1>Links for {}</h1>
{}  </body>
</html>
"#,
        project_name, project_name, links
    )
}

pub fn generate_pypi_simple_root_html(projects: &[String]) -> String {
    let mut links = String::new();
    for p in projects {
        links.push_str(&format!("    <a href=\"{}/\">{}</a><br/>\n", p, p));
    }

    format!(
        r#"<!DOCTYPE html>
<html>
  <head>
    <title>Simple Index</title>
  </head>
  <body>
    <h1>Simple Index</h1>
{}  </body>
</html>
"#,
        links
    )
}
