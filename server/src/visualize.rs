use std::{
    fmt,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use rbx_tree::RbxId;

use crate::{
    imfs::{Imfs, ImfsItem},
    rbx_session::RbxSession,
};

static GRAPHVIZ_HEADER: &str = r#"
digraph RojoTree {
    rankdir = "LR";
    graph [
        ranksep = "0.7",
        nodesep = "0.5",
    ];
    node [
        fontname = "Hack",
        shape = "record",
    ];
"#;

pub fn graphviz_to_svg(source: &str) -> String {
    let mut child = Command::new("dot")
        .arg("-Tsvg")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn GraphViz process -- make sure it's installed in order to use /api/visualize");

    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(source.as_bytes()).expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to read stdout");
    String::from_utf8(output.stdout).expect("Failed to parse stdout as UTF-8")
}

pub struct VisualizeRbxSession<'a>(pub &'a RbxSession);

impl<'a> fmt::Display for VisualizeRbxSession<'a> {
    fn fmt(&self, output: &mut fmt::Formatter) -> fmt::Result {
        writeln!(output, "{}", GRAPHVIZ_HEADER)?;

        visualize_rbx_node(self.0, self.0.get_tree().get_root_id(), output)?;

        writeln!(output, "}}")?;

        Ok(())
    }
}

fn visualize_rbx_node(session: &RbxSession, id: RbxId, output: &mut fmt::Formatter) -> fmt::Result {
    let node = session.get_tree().get_instance(id).unwrap();

    let mut node_label = format!("{}|{}|{}", node.name, node.class_name, id);

    if let Some(metadata) = session.get_instance_metadata(id) {
        node_label.push('|');
        node_label.push_str(&serde_json::to_string(metadata).unwrap());
    }

    node_label = node_label
        .replace("\"", "&quot;")
        .replace("{", "\\{")
        .replace("}", "\\}");

    writeln!(output, "    \"{}\" [label=\"{}\"]", id, node_label)?;

    for &child_id in node.get_children_ids() {
        writeln!(output, "    \"{}\" -> \"{}\"", id, child_id)?;
        visualize_rbx_node(session, child_id, output)?;
    }

    Ok(())
}

pub struct VisualizeImfs<'a>(pub &'a Imfs);

impl<'a> fmt::Display for VisualizeImfs<'a> {
    fn fmt(&self, output: &mut fmt::Formatter) -> fmt::Result {
        writeln!(output, "{}", GRAPHVIZ_HEADER)?;

        for root_path in self.0.get_roots() {
            visualize_root_path(self.0, root_path, output)?;
        }

        writeln!(output, "}}")?;

        Ok(())
    }
}

fn normalize_name(path: &Path) -> String {
    path.to_str().unwrap().replace("\\", "/")
}

fn visualize_root_path(imfs: &Imfs, path: &Path, output: &mut fmt::Formatter) -> fmt::Result {
    let normalized_name = normalize_name(path);
    let item = imfs.get(path).unwrap();

    writeln!(output, "    \"{}\"", normalized_name)?;

    match item {
        ImfsItem::File(_) => {},
        ImfsItem::Directory(directory) => {
            for child_path in &directory.children {
                writeln!(output, "    \"{}\" -> \"{}\"", normalized_name, normalize_name(child_path))?;
                visualize_path(imfs, child_path, output)?;
            }
        },
    }

    Ok(())
}

fn visualize_path(imfs: &Imfs, path: &Path, output: &mut fmt::Formatter) -> fmt::Result {
    let normalized_name = normalize_name(path);
    let short_name = path.file_name().unwrap().to_string_lossy();
    let item = imfs.get(path).unwrap();

    writeln!(output, "    \"{}\" [label = \"{}\"]", normalized_name, short_name)?;

    match item {
        ImfsItem::File(_) => {},
        ImfsItem::Directory(directory) => {
            for child_path in &directory.children {
                writeln!(output, "    \"{}\" -> \"{}\"", normalized_name, normalize_name(child_path))?;
                visualize_path(imfs, child_path, output)?;
            }
        },
    }

    Ok(())
}