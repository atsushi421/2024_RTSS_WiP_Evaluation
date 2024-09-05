use petgraph::{
    graph::{Graph, NodeIndex},
    visit::EdgeRef,
    Direction::{Incoming, Outgoing},
};
use std::cmp::Ord;
use std::collections::{BTreeMap, VecDeque};

/// custom node data structure for dag nodes (petgraph)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Node {
    pub id: i32,
    pub params: BTreeMap<String, i32>,
}

impl Node {
    pub fn new(id: i32, params: BTreeMap<String, i32>) -> Node {
        Node { id, params }
    }

    pub fn get_id(&self) -> NodeIndex {
        NodeIndex::new(self.id as usize)
    }

    pub fn get_value(&self, key: &str) -> i32 {
        *self
            .params
            .get(key)
            .unwrap_or_else(|| panic!("The key {} not found.", key))
    }
}

pub trait DAG {
    fn add_param(&mut self, node_i: NodeIndex, key: &str, value: i32);
    fn update_param(&mut self, node_i: NodeIndex, key: &str, value: i32);
    fn set_param(&mut self, node_i: NodeIndex, key: &str, value: i32);
    fn get_source(&self) -> NodeIndex;
    fn get_sink(&self) -> Vec<NodeIndex>;
    fn get_pre(&self, node_i: NodeIndex) -> Vec<NodeIndex>;
    fn get_suc(&self, node_i: NodeIndex) -> Vec<NodeIndex>;
    fn get_anc(&self, node_i: NodeIndex) -> Vec<NodeIndex>;
    fn get_des(&self, node_i: NodeIndex) -> Vec<NodeIndex>;
    fn get_dag_param(&self, key: &str) -> i32;
    fn set_param_to_all_nodes(&mut self, key: &str, value: i32);
    fn is_node_ready(&self, node_i: NodeIndex) -> bool;
}

impl DAG for Graph<Node, i32> {
    fn add_param(&mut self, node_i: NodeIndex, key: &str, value: i32) {
        let target_node = self.node_weight_mut(node_i).unwrap();
        if target_node.params.contains_key(key) {
            panic!("The key {} already exists.", key);
        } else {
            target_node.params.insert(key.to_string(), value);
        }
    }

    fn update_param(&mut self, node_i: NodeIndex, key: &str, value: i32) {
        let target_node = self.node_weight_mut(node_i).unwrap();
        if !target_node.params.contains_key(key) {
            panic!("The key {} not found.", key);
        } else {
            target_node.params.insert(key.to_string(), value);
        }
    }

    fn set_param(&mut self, node_i: NodeIndex, key: &str, value: i32) {
        let target_node = self.node_weight_mut(node_i).unwrap();
        target_node.params.insert(key.to_string(), value);
    }

    fn get_source(&self) -> NodeIndex {
        let source = self
            .node_indices()
            .filter(|&i| self.edges_directed(i, Incoming).next().is_none())
            .collect::<Vec<_>>();
        if source.len() > 1 {
            panic!("Multiple source nodes found.");
        }

        source[0]
    }

    fn get_sink(&self) -> Vec<NodeIndex> {
        self.node_indices()
            .filter(|&i| self.edges_directed(i, Outgoing).next().is_none())
            .collect::<Vec<_>>()
    }

    fn get_pre(&self, node_i: NodeIndex) -> Vec<NodeIndex> {
        if node_i.index() >= self.node_count() {
            panic!("Node {:?} not found.", node_i);
        }

        let pre_nodes = self
            .edges_directed(node_i, Incoming)
            .map(|edge| edge.source())
            .collect::<Vec<_>>();

        pre_nodes
    }

    fn get_suc(&self, node_i: NodeIndex) -> Vec<NodeIndex> {
        if node_i.index() >= self.node_count() {
            panic!("Node {:?} not found.", node_i);
        }

        let suc_nodes = self
            .edges_directed(node_i, Outgoing)
            .map(|edge| edge.target())
            .collect::<Vec<_>>();

        suc_nodes
    }

    fn get_anc(&self, node_i: NodeIndex) -> Vec<NodeIndex> {
        let mut anc_nodes = Vec::new();
        let mut search_queue = VecDeque::new();
        search_queue.push_back(node_i);

        while let Some(node_i) = search_queue.pop_front() {
            for pre_node in self.get_pre(node_i) {
                if !anc_nodes.contains(&pre_node) {
                    anc_nodes.push(pre_node);
                    search_queue.push_back(pre_node);
                }
            }
        }

        anc_nodes
    }

    fn get_des(&self, node_i: NodeIndex) -> Vec<NodeIndex> {
        let mut des_nodes = Vec::new();
        let mut search_queue = VecDeque::new();
        search_queue.push_back(node_i);

        while let Some(node_i) = search_queue.pop_front() {
            for suc_node in self.get_suc(node_i) {
                if !des_nodes.contains(&suc_node) {
                    des_nodes.push(suc_node);
                    search_queue.push_back(suc_node);
                }
            }
        }

        des_nodes
    }

    fn get_dag_param(&self, key: &str) -> i32 {
        self[self.get_source()].get_value(key)
    }

    fn set_param_to_all_nodes(&mut self, key: &str, value: i32) {
        if self.node_indices().count() == 0 {
            panic!("No node exists.");
        }

        for node_i in self.node_indices() {
            if self[node_i].params.contains_key(key) {
                self.update_param(node_i, key, value);
            } else {
                self.add_param(node_i, key, value);
            }
        }
    }

    fn is_node_ready(&self, node_i: NodeIndex) -> bool {
        let pre_nodes_count = self.get_pre(node_i).len() as i32;
        let pre_done_count = self[node_i].params.get("pre_done_count").unwrap_or(&0);
        pre_nodes_count == *pre_done_count
    }
}

#[cfg(test)]
mod tests_dag {
    use super::*;

    fn create_node(key: &str, value: Option<i32>) -> Node {
        let mut params = BTreeMap::new();
        params.insert(key.to_string(), value.unwrap_or_default());
        Node { id: 0, params }
    }

    #[test]
    fn test_add_param_normal() {
        let mut dag = Graph::<Node, i32>::new();
        let n0 = dag.add_node(create_node("execution_time", None));
        const TEST_VALUE: i32 = 1;
        dag.add_param(n0, "test", TEST_VALUE);
        assert_eq!(dag[n0].get_value("test"), TEST_VALUE);
    }

    #[test]
    fn test_update_param_normal() {
        let mut dag = Graph::<Node, i32>::new();
        let n0 = dag.add_node(create_node("execution_time", None));
        const TEST_VALUE: i32 = 1;
        dag.update_param(n0, "execution_time", TEST_VALUE);
        assert_eq!(dag[n0].get_value("execution_time"), TEST_VALUE);
    }

    #[test]
    fn test_get_source_normal() {
        let mut dag = Graph::<Node, i32>::new();
        let n0 = dag.add_node(create_node("execution_time", None));
        let n1 = dag.add_node(create_node("execution_time", None));
        let n2 = dag.add_node(create_node("execution_time", None));
        dag.add_edge(n0, n1, 1);
        dag.add_edge(n0, n2, 1);
        assert_eq!(dag.get_source(), NodeIndex::new(0));
    }

    #[test]
    fn test_get_sink_normal() {
        let mut dag = Graph::<Node, i32>::new();
        let n0 = dag.add_node(create_node("execution_time", None));
        let n1 = dag.add_node(create_node("execution_time", None));
        let n2 = dag.add_node(create_node("execution_time", None));
        assert_eq!(
            dag.get_sink(),
            vec![NodeIndex::new(0), NodeIndex::new(1), NodeIndex::new(2)]
        );
        dag.add_edge(n0, n1, 0);
        dag.add_edge(n0, n2, 0);
        assert_eq!(dag.get_sink(), vec![NodeIndex::new(1), NodeIndex::new(2)]);
    }

    #[test]
    fn test_get_pre_normal() {
        let mut dag = Graph::<Node, i32>::new();
        let n0 = dag.add_node(create_node("execution_time", None));
        let n1 = dag.add_node(create_node("execution_time", None));
        let n2 = dag.add_node(create_node("execution_time", None));
        dag.add_edge(n1, n2, 0);
        dag.add_edge(n0, n2, 0);

        assert_eq!(dag.get_pre(n2), vec![n0, n1]);
    }

    #[test]
    fn test_get_suc_normal() {
        let mut dag = Graph::<Node, i32>::new();
        let n0 = dag.add_node(create_node("execution_time", None));
        let n1 = dag.add_node(create_node("execution_time", None));
        let n2 = dag.add_node(create_node("execution_time", None));
        dag.add_edge(n0, n1, 1);
        dag.add_edge(n0, n2, 1);

        assert_eq!(dag.get_suc(n0), vec![n2, n1]);
    }

    #[test]
    fn test_get_anc_nodes_normal() {
        let mut dag = Graph::<Node, i32>::new();
        let n0 = dag.add_node(create_node("execution_time", None));
        let n1 = dag.add_node(create_node("execution_time", None));
        let n2 = dag.add_node(create_node("execution_time", None));
        let n3 = dag.add_node(create_node("execution_time", None));
        dag.add_edge(n0, n1, 0);
        dag.add_edge(n2, n3, 0);
        dag.add_edge(n1, n3, 0);

        assert_eq!(dag.get_anc(n3), vec![n1, n2, n0]);
    }

    #[test]
    fn test_get_des_nodes_normal() {
        let mut dag = Graph::<Node, i32>::new();
        let n0 = dag.add_node(create_node("execution_time", None));
        let n1 = dag.add_node(create_node("execution_time", None));
        let n2 = dag.add_node(create_node("execution_time", None));
        let n3 = dag.add_node(create_node("execution_time", None));
        dag.add_edge(n0, n1, 0);
        dag.add_edge(n0, n2, 0);
        dag.add_edge(n1, n3, 0);

        assert_eq!(dag.get_des(n0), vec![n2, n1, n3]);
    }

    #[test]
    fn test_set_dag_param_normal() {
        let mut dag = Graph::<Node, i32>::new();
        dag.add_node(create_node("execution_time", None));
        dag.add_node(create_node("execution_time", None));
        const DAG_ID: i32 = 0;
        dag.set_param_to_all_nodes("dag_id", DAG_ID);

        for node_i in dag.node_indices() {
            assert_eq!(dag[node_i].get_value("dag_id"), DAG_ID);
        }
    }

    #[test]
    fn test_is_node_ready_normal() {
        let mut dag = Graph::<Node, i32>::new();
        let n0 = dag.add_node(create_node("execution_time", None));
        let n1 = dag.add_node(create_node("execution_time", None));
        dag.add_edge(n0, n1, 0);

        assert!(dag.is_node_ready(n0));
        assert!(!dag.is_node_ready(n1));
        dag.add_param(n1, "pre_done_count", 1);
        assert!(dag.is_node_ready(n1));
    }
}
