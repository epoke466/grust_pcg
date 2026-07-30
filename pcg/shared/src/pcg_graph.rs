pub mod graph {
    use std::{error::Error, fs, io::Result as IOResult, path::PathBuf};

    use crate::PCGNode;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PCGGraph {
        pub nodes: Vec<PCGNode>,
        pub version: i64,
    }

    /// Save
    pub fn save_graph_file(path: &str, graph: &PCGGraph) -> Result<(), Box<dyn Error>> {
        let contents = ron::ser::to_string_pretty(graph, ron::ser::PrettyConfig::default())?;
        fs::write(path, contents)?;
        Ok(())
    }

    fn trim_deleted_folder(path: &PathBuf) -> IOResult<()> {
        let mut files: Vec<_> = fs::read_dir(path)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let metadata = entry.metadata().ok()?;

                if metadata.is_file() {
                    Some((entry.path(), metadata.modified().ok()?))
                } else {
                    None
                }
            })
            .collect();

        // Oldest first
        files.sort_by_key(|(_, modified)| *modified);

        while files.len() > 5 {
            let (oldest, _) = files.remove(0);
            fs::remove_file(oldest)?;
        }

        Ok(())
    }

    ///Delete
    pub fn delete_graph_file(path: &PathBuf, graph_name: &str) -> Result<(), Box<dyn Error>> {
        let deleted_path = path.join("deleted");
        fs::create_dir_all(&deleted_path)?;

        let graph_path = path.join(format!("{graph_name}"));
        let deleted_graph_path = deleted_path.join(format!("{graph_name}"));

        match fs::rename(&graph_path, &deleted_graph_path) {
            Ok(_) => {
                trim_deleted_folder(&deleted_path).unwrap();
            }
            Err(e) => {
                println!("Source: {}", graph_path.display());
                println!("Dest:   {}", deleted_graph_path.display());
                panic!("{e}");
            }
        }

        Ok(())
    }

    // Load
    pub fn load_graph_file(path: &str) -> Option<PCGGraph> {
        let contents = fs::read_to_string(path).ok()?;
        ron::from_str(&contents).ok()
    }

    impl Default for PCGGraph {
        fn default() -> Self {
            Self {
                nodes: Vec::new(),
                version: 0,
            }
        }
    }
}
