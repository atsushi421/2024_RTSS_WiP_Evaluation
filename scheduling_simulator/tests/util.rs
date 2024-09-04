use std::collections::BTreeMap;

use petgraph::Graph;
use scheduling_simulator::task::dag::{Node, DAG};

// http://retis.sssup.it/~giorgio/paps/2005/rtsj05-rmedf.pdf
// Figure 1
pub fn create_sequential_dag_set0() -> Vec<Graph<Node, i32>> {
    let mut dag_set = Vec::new();

    let mut dag = Graph::<Node, i32>::new();
    let params = BTreeMap::from([
        ("period".to_string(), 5),
        ("relative_deadline".to_string(), 5),
        ("execution_time".to_string(), 2),
    ]);
    dag.add_node(Node::new(0, params));
    dag.set_param_to_all_nodes("dag_id", 0);
    dag_set.push(dag);

    let mut dag = Graph::<Node, i32>::new();
    let params = BTreeMap::from([
        ("period".to_string(), 7),
        ("relative_deadline".to_string(), 7),
        ("execution_time".to_string(), 4),
    ]);
    dag.add_node(Node::new(0, params));
    dag.set_param_to_all_nodes("dag_id", 1);
    dag_set.push(dag);

    dag_set
}

pub fn create_sequential_dag_set1() -> Vec<Graph<Node, i32>> {
    let mut dag_set = Vec::new();

    let mut dag = Graph::<Node, i32>::new();
    let params = BTreeMap::from([
        ("period".to_string(), 5),
        ("relative_deadline".to_string(), 5),
        ("execution_time".to_string(), 1),
    ]);
    dag.add_node(Node::new(0, params));
    dag.set_param_to_all_nodes("dag_id", 0);
    dag_set.push(dag);

    let mut dag = Graph::<Node, i32>::new();
    let params = BTreeMap::from([
        ("period".to_string(), 7),
        ("relative_deadline".to_string(), 7),
        ("execution_time".to_string(), 4),
    ]);
    dag.add_node(Node::new(0, params));
    dag.set_param_to_all_nodes("dag_id", 1);
    dag_set.push(dag);

    dag_set
}
