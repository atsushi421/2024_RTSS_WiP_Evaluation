//! Generate a petgraph DAG object from a yaml file

use petgraph::{graph::Graph, prelude::*};
use rand::seq::SliceRandom;
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::PathBuf,
};
use yaml_rust::{Yaml, YamlLoader};

use super::dag::{Node, DAG};

fn load_yaml(path: &str) -> Vec<Yaml> {
    if !path.ends_with(".yaml") && !path.ends_with(".yml") {
        panic!("Invalid file type: {}", path);
    }
    YamlLoader::load_from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

const AUTOWARE_ET_DIR: &str = "/home/atsushi/2024_RTSS_WiP_Evaluation/autoware_execution_times";

fn choice_execution_time_us(execution_time_file: &str) -> i32 {
    let file = File::open(PathBuf::from(AUTOWARE_ET_DIR).join(execution_time_file)).unwrap();
    let reader = BufReader::new(file);
    let mut execution_times: Vec<i32> = Vec::new();

    for line in reader.lines() {
        execution_times.push(line.unwrap().trim().parse::<i32>().unwrap());
    }

    // ns -> us
    (*execution_times.choose(&mut rand::thread_rng()).unwrap() as f32 / 1000.0).ceil() as i32
}

fn create_dag_from_yaml(path: &str) -> Graph<Node, i32> {
    let content = &load_yaml(path)[0];
    let mut dag = Graph::<Node, i32>::new();

    // Load nodes
    let nodes = content["nodes"]
        .as_vec()
        .expect("`nodes` field does not exist.");
    for node in nodes {
        let mut params = BTreeMap::new();
        let id = node["id"].as_i64().expect("`id` field does not exist.") as i32;
        params.insert(
            "execution_time".to_owned(),
            choice_execution_time_us(node["execution_time_file"].as_str().unwrap()),
        );

        // Load node parameters
        for (key, value) in node.as_hash().unwrap() {
            let key_str = key.as_str().unwrap();
            if key_str == "id" || key_str == "execution_time_file" {
                continue;
            }

            match value {
                Yaml::Integer(_) => {
                    params.insert(key_str.to_owned(), (value.as_i64().unwrap()) as i32);
                }
                _ => {
                    println!("Non-integer type parameter found: {:?}", value);
                    panic!(
                        "Non-integer type parameter found: {}",
                        std::any::type_name::<Yaml>()
                    );
                }
            }
        }

        dag.add_node(Node { id, params });
    }

    // Load edges
    let links = content["links"]
        .as_vec()
        .expect("`links` field does not exist.");
    for link in links {
        let source = link["source"]
            .as_i64()
            .expect("`source` field does not exist()") as usize;
        let target = link["target"]
            .as_i64()
            .expect("`target` field does not exist()") as usize;

        dag.add_edge(NodeIndex::new(source), NodeIndex::new(target), 0);
    }

    dag
}

fn get_yaml_paths_from_dir(dir_path: &str) -> Vec<String> {
    if !std::fs::metadata(dir_path).unwrap().is_dir() {
        panic!("Not a directory");
    }

    let mut yaml_paths = Vec::new();
    for dir_entry in PathBuf::from(dir_path).read_dir().unwrap() {
        let path = dir_entry.unwrap().path();
        let extension = path.extension().unwrap();
        if extension == "yaml" || extension == "yml" {
            yaml_paths.push(path.to_str().unwrap().to_string());
        }
    }

    if yaml_paths.is_empty() {
        panic!("No YAML file found in {}", dir_path);
    }

    yaml_paths
}

pub fn create_dag_set_from_dir(dir_path: &str) -> Vec<Graph<Node, i32>> {
    let mut yaml_paths = get_yaml_paths_from_dir(dir_path);
    yaml_paths.sort();

    let mut dag_set: Vec<Graph<Node, i32>> = Vec::new();
    for (dag_id, path) in yaml_paths.iter().enumerate() {
        let mut dag = create_dag_from_yaml(path);
        dag.set_param_to_all_nodes("dag_id", dag_id as i32);
        dag_set.push(dag);
    }

    dag_set
}
