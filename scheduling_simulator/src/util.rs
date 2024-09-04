use crate::{
    processor::core::ProcessResult,
    task::dag::{Node, DAG},
};
use chrono::{DateTime, Utc};
use log::{info, warn};
use num_integer::lcm;
use petgraph::graph::Graph;
use std::{
    fs::{self, OpenOptions},
    io::Write,
};

pub fn get_hyper_period(dag_set: &[Graph<Node, i32>]) -> i32 {
    let mut hyper_period = 1;
    for dag in dag_set {
        let dag_period = dag.get_dag_period();
        hyper_period = lcm(hyper_period, dag_period);
    }
    hyper_period
}

pub fn append_info_to_yaml(file_path: &str, info: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(file_path)
    {
        if let Err(err) = file.write_all(info.as_bytes()) {
            eprintln!("Failed to write to file: {}", err);
        }
    } else {
        eprintln!("Failed to open file: {}", file_path);
    }
}

pub fn create_yaml(folder_path: &str, file_name: &str) -> String {
    if fs::metadata(folder_path).is_err() {
        let _ = fs::create_dir_all(folder_path);
        info!("Created folder: {}", folder_path);
    }
    let file_path = format!("{}/{}.yaml", folder_path, file_name);
    if let Err(err) = fs::File::create(&file_path) {
        warn!("Failed to create file: {}", err);
    }
    file_path
}

pub fn create_scheduler_log_yaml(dir_path: &str, alg_name: &str) -> String {
    let now: DateTime<Utc> = Utc::now();
    let date = now.format("%Y-%m-%d-%H-%M-%S-%3f").to_string();
    let file_name = format!("{}-{}-log", date, alg_name);
    create_yaml(dir_path, &file_name)
}

pub fn approx_eq(a: f64, b: f64) -> bool {
    const EPSILON: f64 = 1e-6;
    (a - b).abs() < EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn create_dag() -> Graph<Node, i32> {
        let mut dag = Graph::<Node, i32>::new();
        let mut params = BTreeMap::new();
        params.insert("execution_time".to_owned(), 4);
        dag.add_node(Node { id: 0, params });

        dag
    }

    fn create_dag_with_period(period: i32) -> Graph<Node, i32> {
        let mut dag = Graph::<Node, i32>::new();
        let mut params = BTreeMap::new();
        params.insert("execution_time".to_owned(), 4);
        params.insert("period".to_owned(), period);
        let n0 = dag.add_node(Node { id: 0, params });

        params = BTreeMap::new();
        params.insert("execution_time".to_owned(), 4);
        let n1 = dag.add_node(Node { id: 1, params });

        dag.add_edge(n0, n1, 0);

        dag
    }

    fn create_dag_with_deadline(deadline: i32) -> Graph<Node, i32> {
        let mut dag = Graph::<Node, i32>::new();
        let mut params = BTreeMap::new();
        params.insert("execution_time".to_owned(), 4);
        let n0 = dag.add_node(Node { id: 0, params });

        params = BTreeMap::new();
        params.insert("execution_time".to_owned(), 4);
        params.insert("end_to_end_deadline".to_owned(), deadline);
        let n1 = dag.add_node(Node { id: 1, params });

        dag.add_edge(n0, n1, 0);

        dag
    }

    fn create_dag_with_period_and_deadline(period: i32, deadline: i32) -> Graph<Node, i32> {
        let mut dag = Graph::<Node, i32>::new();
        let mut params = BTreeMap::new();
        params.insert("execution_time".to_owned(), 4);
        params.insert("period".to_owned(), period);
        let n0 = dag.add_node(Node { id: 0, params });

        params = BTreeMap::new();
        params.insert("execution_time".to_owned(), 4);
        params.insert("end_to_end_deadline".to_owned(), deadline);
        let n1 = dag.add_node(Node { id: 1, params });

        dag.add_edge(n0, n1, 0);

        dag
    }

    #[test]
    fn test_get_hyper_period_normal() {
        let dag_set = vec![
            create_dag_with_period(10),
            create_dag_with_period(20),
            create_dag_with_period(30),
            create_dag_with_period(40),
        ];
        assert_eq!(get_hyper_period(&dag_set), 120);
    }
}
