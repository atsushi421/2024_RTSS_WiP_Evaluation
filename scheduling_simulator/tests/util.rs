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

fn create_node_with_et(id: i32, et: i32) -> Node {
    let params = BTreeMap::from([("execution_time".to_string(), et)]);
    Node::new(id, params)
}

// A New Federated Scheduling Algorithm for Arbitrary-Deadline DAG Tasks (TC '23)
// Figure 1
pub fn create_simple_dag() -> Graph<Node, i32> {
    let mut dag = Graph::<Node, i32>::new();
    let params = BTreeMap::from([
        ("period".to_string(), 10),
        ("execution_time".to_string(), 1),
    ]);
    let node_0 = dag.add_node(Node::new(0, params));
    let node_1 = dag.add_node(create_node_with_et(1, 3));
    let node_2 = dag.add_node(create_node_with_et(2, 3));
    let node_3 = dag.add_node(create_node_with_et(3, 4));
    let node_4 = dag.add_node(create_node_with_et(4, 3));
    let node_5 = dag.add_node(create_node_with_et(5, 3));
    let node_6 = dag.add_node(create_node_with_et(6, 1));

    dag.add_edge(node_0, node_1, 0);
    dag.add_edge(node_0, node_2, 0);
    dag.add_edge(node_0, node_3, 0);
    dag.add_edge(node_0, node_4, 0);
    dag.add_edge(node_0, node_5, 0);
    dag.add_edge(node_1, node_6, 0);
    dag.add_edge(node_2, node_6, 0);
    dag.add_edge(node_3, node_6, 0);
    dag.add_edge(node_4, node_6, 0);
    dag.add_edge(node_5, node_6, 0);

    dag.set_param_to_all_nodes("dag_id", 0);
    dag.set_param_to_all_nodes("relative_deadline", 12);

    dag
}
