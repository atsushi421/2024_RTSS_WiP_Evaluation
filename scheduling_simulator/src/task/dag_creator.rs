//! Generate a petgraph DAG object from a yaml file
use crate::util::load_yaml;

use petgraph::{graph::Graph, prelude::*};
use std::{collections::BTreeMap, path::PathBuf};
use yaml_rust::Yaml;

use super::dag::{DAG, NodeData};

pub fn create_dag_from_yaml(file_path: &str) -> Graph<NodeData, i32> {
    let yaml_docs = load_yaml(file_path);
    let yaml_doc = &yaml_docs[0];

    // Check if nodes and links fields exist
    if let (Some(nodes), Some(links)) = (yaml_doc["nodes"].as_vec(), yaml_doc["links"].as_vec()) {
        let mut dag = Graph::<NodeData, i32>::new();

        // add nodes to dag
        for node in nodes {
            let mut params = BTreeMap::new();
            let id = node["id"].as_i64().unwrap() as i32;

            // add node parameters to BTreeMap
            for (key, value) in node.as_hash().unwrap() {
                let key_str = key.as_str().unwrap();
                if key_str != "id" {
                    match value {
                        Yaml::Integer(_i) => {
                            params.insert(key_str.to_owned(), (value.as_i64().unwrap()) as i32);
                        }
                        _ => {
                            panic!("Unknown type: {}", std::any::type_name::<Yaml>());
                        }
                    }
                }
            }
            dag.add_node(NodeData { id, params });
        }

        // add edges to dag
        for link in links {
            let source = link["source"].as_i64().unwrap() as usize;
            let target = link["target"].as_i64().unwrap() as usize;
            let mut communication_time = 0;

            dag.add_edge(
                NodeIndex::new(source),
                NodeIndex::new(target),
                communication_time,
            );
        }
        dag
    } else {
        panic!("YAML files are not DAG structures.");
    }
}

fn get_yaml_paths_from_dir(dir_path: &str) -> Vec<String> {
    if !std::fs::metadata(dir_path).unwrap().is_dir() {
        panic!("Not a directory");
    }

    let mut file_path_list = Vec::new();
    for dir_entry_result in PathBuf::from(dir_path).read_dir().unwrap() {
        let path = dir_entry_result.unwrap().path();
        let extension = path.extension().unwrap();
        if extension == "yaml" || extension == "yml" {
            file_path_list.push(path.to_str().unwrap().to_string());
        }
    }

    if file_path_list.is_empty() {
        panic!("No YAML file found in {}", dir_path);
    }

    file_path_list
}

pub fn create_dag_set_from_dir(dir_path: &str) -> Vec<Graph<NodeData, i32>> {
    let mut file_path_list = get_yaml_paths_from_dir(dir_path);
    file_path_list.sort();
    let mut dag_set: Vec<Graph<NodeData, i32>> = Vec::new();
    for (dag_id, file_path) in file_path_list.iter().enumerate() {
        let mut dag = create_dag_from_yaml(file_path);
        dag.set_dag_param("dag_id", dag_id as i32);
        dag_set.push(dag);
    }
    dag_set
}
