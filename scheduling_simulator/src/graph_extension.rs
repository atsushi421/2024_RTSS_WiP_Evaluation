use log::warn;
use petgraph::{
    graph::{Graph, NodeIndex},
    visit::EdgeRef,
    Direction::{Incoming, Outgoing},
};
use std::cmp::Ord;
use std::collections::{BTreeMap, VecDeque};

/// custom node data structure for dag nodes (petgraph)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeData {
    pub id: i32,
    pub params: BTreeMap<String, i32>,
}

impl NodeData {
    pub fn new(id: i32, params: BTreeMap<String, i32>) -> NodeData {
        NodeData { id, params }
    }

    pub fn get_id(&self) -> NodeIndex {
        NodeIndex::new(self.id as usize)
    }

    pub fn get_params_value(&self, key: &str) -> i32 {
        *self
            .params
            .get(key)
            .unwrap_or_else(|| panic!("The key does not exist. key: {}", key))
    }
}

pub trait GraphExtension {
    fn add_param(&mut self, node_i: NodeIndex, key: &str, value: i32);
    fn update_param(&mut self, node_i: NodeIndex, key: &str, value: i32);
    fn remove_nodes(&mut self, node_indices: &[NodeIndex]);
    fn get_source_nodes(&self) -> Vec<NodeIndex>;
    fn get_sink_nodes(&self) -> Vec<NodeIndex>;
    fn get_head_period(&self) -> Option<i32>;
    fn get_pre_nodes(&self, node_i: NodeIndex) -> Option<Vec<NodeIndex>>;
    fn get_suc_nodes(&self, node_i: NodeIndex) -> Option<Vec<NodeIndex>>;
    fn get_anc_nodes(&self, node_i: NodeIndex) -> Option<Vec<NodeIndex>>;
    fn get_des_nodes(&self, node_i: NodeIndex) -> Option<Vec<NodeIndex>>;
    fn get_dag_param(&self, key: &str) -> i32;
    fn set_dag_param(&mut self, key: &str, value: i32);
    fn add_node_with_id_consistency(&mut self, node: NodeData) -> NodeIndex;
    fn is_node_ready(&self, node_i: NodeIndex) -> bool;
}

impl GraphExtension for Graph<NodeData, i32> {
    fn add_param(&mut self, node_i: NodeIndex, key: &str, value: i32) {
        let target_node = self.node_weight_mut(node_i).unwrap();
        if target_node.params.contains_key(key) {
            warn!("The key already exists. key: {}", key);
        } else {
            target_node.params.insert(key.to_string(), value);
        }
    }

    fn update_param(&mut self, node_i: NodeIndex, key: &str, value: i32) {
        let target_node = self.node_weight_mut(node_i).unwrap();
        if !target_node.params.contains_key(key) {
            warn!("The key no exists. key: {}", key);
        } else {
            target_node.params.insert(key.to_string(), value);
        }
    }

    fn remove_nodes(&mut self, node_indices: &[NodeIndex]) {
        for node_i in node_indices.iter().rev() {
            self.remove_node(*node_i);
        }
    }

    fn get_source_nodes(&self) -> Vec<NodeIndex> {
        self.node_indices()
            .filter(|&i| self.edges_directed(i, Incoming).next().is_none())
            .collect::<Vec<_>>()
    }

    fn get_sink_nodes(&self) -> Vec<NodeIndex> {
        self.node_indices()
            .filter(|&i| self.edges_directed(i, Outgoing).next().is_none())
            .collect::<Vec<_>>()
    }

    fn get_head_period(&self) -> Option<i32> {
        let source_nodes = self.get_source_nodes();
        let periods: Vec<&i32> = source_nodes
            .iter()
            .filter_map(|&node_i| self[node_i].params.get("period"))
            .collect();

        if source_nodes.len() > 1 {
            warn!("Multiple source nodes found.");
        }
        if periods.len() > 1 {
            warn!("Multiple periods found. The first period is used.");
        }
        if periods.is_empty() {
            warn!("No period found.");
            return None;
        }
        Some(*periods[0])
    }

    fn get_pre_nodes(&self, node_i: NodeIndex) -> Option<Vec<NodeIndex>> {
        //Since node indices are sequentially numbered, this is used to determine whether a node exists or not.
        if node_i.index() < self.node_count() {
            let pre_nodes = self
                .edges_directed(node_i, Incoming)
                .map(|edge| edge.source())
                .collect::<Vec<_>>();

            if pre_nodes.is_empty() {
                None
            } else {
                Some(pre_nodes)
            }
        } else {
            panic!("Node {:?} does not exist!", node_i);
        }
    }

    fn get_suc_nodes(&self, node_i: NodeIndex) -> Option<Vec<NodeIndex>> {
        //Since node indices are sequentially numbered, this is used to determine whether a node exists or not.
        if node_i.index() < self.node_count() {
            let suc_nodes = self
                .edges_directed(node_i, Outgoing)
                .map(|edge| edge.target())
                .collect::<Vec<_>>();

            if suc_nodes.is_empty() {
                None
            } else {
                Some(suc_nodes)
            }
        } else {
            panic!("Node {:?} does not exist!", node_i);
        }
    }

    fn get_anc_nodes(&self, node_i: NodeIndex) -> Option<Vec<NodeIndex>> {
        let mut anc_nodes = Vec::new();
        let mut search_queue = VecDeque::new();
        search_queue.push_back(node_i);

        while let Some(node) = search_queue.pop_front() {
            //If the target node does not exist, get_pre_node causes panic!
            for pre_node in self.get_pre_nodes(node).unwrap_or_default() {
                if !anc_nodes.contains(&pre_node) {
                    anc_nodes.push(pre_node);
                    search_queue.push_back(pre_node);
                }
            }
        }
        Some(anc_nodes).filter(|anc| !anc.is_empty())
    }

    fn get_des_nodes(&self, node_i: NodeIndex) -> Option<Vec<NodeIndex>> {
        let mut des_nodes = Vec::new();
        let mut search_queue = VecDeque::new();
        search_queue.push_back(node_i);

        while let Some(node) = search_queue.pop_front() {
            //If the target node does not exist, get_suc_node causes panic!
            for suc_node in self.get_suc_nodes(node).unwrap_or_default() {
                if !des_nodes.contains(&suc_node) {
                    des_nodes.push(suc_node);
                    search_queue.push_back(suc_node);
                }
            }
        }
        Some(des_nodes).filter(|des| !des.is_empty())
    }

    fn get_dag_param(&self, key: &str) -> i32 {
        if self.node_indices().count() == 0 {
            panic!(
                "Error: {} does not exist. Please use set_dag_param({}, value)",
                key, key
            );
        }
        self[NodeIndex::new(0)].params[key]
    }

    fn set_dag_param(&mut self, key: &str, value: i32) {
        if self.node_indices().count() == 0 {
            panic!("No node found.");
        }
        for node_i in self.node_indices() {
            if self[node_i].params.contains_key(key) {
                self.update_param(node_i, key, value);
            } else {
                self.add_param(node_i, key, value);
            }
        }
    }

    fn add_node_with_id_consistency(&mut self, node: NodeData) -> NodeIndex {
        let node_index = self.add_node(node);

        assert_eq!(
            node_index.index() as i32,
            self[node_index].id,
            "The add node id is different from NodeIndex."
        );

        node_index
    }

    fn is_node_ready(&self, node_i: NodeIndex) -> bool {
        let pre_nodes_count = self.get_pre_nodes(node_i).unwrap_or_default().len() as i32;
        let pre_done_nodes_count = self[node_i].params.get("pre_done_count").unwrap_or(&0);
        pre_nodes_count == *pre_done_nodes_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_node(id: i32, key: &str, value: i32) -> NodeData {
        let mut params = BTreeMap::new();
        params.insert(key.to_string(), value);
        NodeData { id, params }
    }

    #[test]
    fn test_add_param_normal() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "execution_time", 0));
        dag.add_param(n0, "test", 1);
        assert_eq!(dag[n0].params.get("test").unwrap(), &1);
        assert_eq!(dag[n0].params.get("execution_time").unwrap(), &0);
    }

    #[test]
    fn test_add_param_duplicate() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "execution_time", 0));
        assert_eq!(dag[n0].params.get("execution_time").unwrap(), &0);
        dag.add_param(n0, "execution_time", 1);
        assert_eq!(dag[n0].params.get("execution_time").unwrap(), &0);
    }

    #[test]
    fn test_update_param_normal() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "execution_time", 0));
        assert_eq!(dag[n0].params.get("execution_time").unwrap(), &0);
        dag.update_param(n0, "execution_time", 1);
        assert_eq!(dag[n0].params.get("execution_time").unwrap(), &1);
    }
    #[test]
    fn test_update_param_no_exist_params() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "execution_time", 0));
        dag.update_param(n0, "test", 1);
        assert_eq!(dag[n0].params.get("test"), None);
        assert_eq!(dag[n0].params.get("execution_time").unwrap(), &0);
    }

    #[test]
    fn test_remove_nodes_normal() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "execution_time", 3));
        let n1 = dag.add_node(create_node(1, "execution_time", 6));
        let n2 = dag.add_node(create_node(2, "execution_time", 45));
        dag.add_edge(n0, n1, 1);
        dag.add_edge(n0, n2, 1);

        dag.remove_nodes(&[n1, n2]);
        assert_eq!(dag.node_count(), 1);
        assert_eq!(dag.edge_count(), 0);
        assert_eq!(dag[n0].id, 0);

        fn contains(dag: &Graph<NodeData, i32>, node: NodeIndex) -> bool {
            dag.node_indices().any(|i| i == node)
        }

        assert!(!contains(&dag, n1));
        assert!(!contains(&dag, n2));
    }

    #[test]
    fn test_get_source_nodes_normal() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "execution_time", 0));
        let n1 = dag.add_node(create_node(1, "execution_time", 0));
        let n2 = dag.add_node(create_node(2, "execution_time", 0));
        assert_eq!(
            dag.get_source_nodes(),
            vec![NodeIndex::new(0), NodeIndex::new(1), NodeIndex::new(2),]
        );
        dag.add_edge(n0, n1, 1);
        dag.add_edge(n0, n2, 1);
        assert_eq!(dag.get_source_nodes(), vec![NodeIndex::new(0)]);
    }

    #[test]
    fn test_get_sink_nodes_normal() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "execution_time", 0));
        let n1 = dag.add_node(create_node(1, "execution_time", 0));
        let n2 = dag.add_node(create_node(2, "execution_time", 0));
        assert_eq!(
            dag.get_sink_nodes(),
            vec![NodeIndex::new(0), NodeIndex::new(1), NodeIndex::new(2)]
        );
        dag.add_edge(n0, n1, 1);
        dag.add_edge(n0, n2, 1);
        assert_eq!(
            dag.get_sink_nodes(),
            vec![NodeIndex::new(1), NodeIndex::new(2)]
        );
    }

    #[test]
    fn test_get_head_period_normal() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "period", 3));
        let n1 = dag.add_node(create_node(0, "period", 4));

        dag.add_edge(n0, n1, 1);

        assert_eq!(dag.get_head_period(), Some(3));
    }

    #[test]
    fn test_get_head_period_node_no_includes_period() {
        let mut dag = Graph::<NodeData, i32>::new();
        dag.add_node(create_node(0, "weight", 3));

        assert_eq!(dag.get_head_period(), None);
    }

    #[test]
    fn test_get_pre_nodes_normal() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "execution_time", 0));
        let n1 = dag.add_node(create_node(1, "execution_time", 0));
        let n2 = dag.add_node(create_node(2, "execution_time", 0));
        dag.add_edge(n1, n2, 1);
        dag.add_edge(n0, n2, 1);

        assert_eq!(dag.get_pre_nodes(n2), Some(vec![n0, n1]));
    }

    #[test]
    fn test_get_pre_nodes_single() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "execution_time", 0));
        let n1 = dag.add_node(create_node(1, "execution_time", 0));
        dag.add_edge(n0, n1, 1);

        assert_eq!(dag.get_pre_nodes(n1), Some(vec![n0]));
    }

    #[test]
    fn test_get_pre_nodes_no_exist_pre_nodes() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "execution_time", 0));

        assert_eq!(dag.get_pre_nodes(n0), None);
    }

    #[test]
    #[should_panic]
    fn test_get_pre_nodes_no_exist_target_node() {
        let dag = Graph::<NodeData, i32>::new();
        let invalid_node = NodeIndex::new(999);

        assert_eq!(dag.get_pre_nodes(invalid_node), None);
    }

    #[test]
    fn test_get_suc_nodes_normal() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "execution_time", 0));
        let n1 = dag.add_node(create_node(1, "execution_time", 0));
        let n2 = dag.add_node(create_node(2, "execution_time", 0));
        dag.add_edge(n0, n1, 1);
        dag.add_edge(n0, n2, 1);

        assert_eq!(dag.get_suc_nodes(n0), Some(vec![n2, n1]));
    }

    #[test]
    fn test_get_suc_nodes_single() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "execution_time", 0));
        let n1 = dag.add_node(create_node(1, "execution_time", 0));
        dag.add_edge(n0, n1, 1);

        assert_eq!(dag.get_suc_nodes(n0), Some(vec![n1]));
    }

    #[test]
    fn test_get_suc_nodes_no_exist_suc_nodes() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "execution_time", 0));

        assert_eq!(dag.get_suc_nodes(n0), None);
    }

    #[test]
    #[should_panic]
    fn test_get_suc_nodes_no_exist_target_node() {
        let dag = Graph::<NodeData, i32>::new();
        let invalid_node = NodeIndex::new(999);

        assert_eq!(dag.get_suc_nodes(invalid_node), None);
    }

    #[test]
    fn test_get_anc_nodes_normal() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "execution_time", 0));
        let n1 = dag.add_node(create_node(1, "execution_time", 0));
        let n2 = dag.add_node(create_node(2, "execution_time", 0));
        let n3 = dag.add_node(create_node(3, "execution_time", 0));
        dag.add_edge(n0, n1, 1);
        dag.add_edge(n2, n3, 1);
        dag.add_edge(n1, n3, 1);

        assert_eq!(dag.get_anc_nodes(n3), Some(vec![n1, n2, n0]));
    }

    #[test]
    fn test_get_anc_nodes_single() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "execution_time", 0));
        let n1 = dag.add_node(create_node(1, "execution_time", 0));
        dag.add_edge(n0, n1, 1);

        assert_eq!(dag.get_anc_nodes(n1), Some(vec![n0]));
    }

    #[test]
    fn test_get_anc_nodes_no_exist_anc_nodes() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "execution_time", 0));

        assert_eq!(dag.get_anc_nodes(n0), None);
    }

    #[test]
    #[should_panic]
    fn test_get_anc_nodes_no_exist_target_node() {
        let dag = Graph::<NodeData, i32>::new();
        let invalid_node = NodeIndex::new(999);

        assert_eq!(dag.get_anc_nodes(invalid_node), None);
    }

    #[test]
    fn test_get_des_nodes_normal() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "execution_time", 0));
        let n1 = dag.add_node(create_node(1, "execution_time", 0));
        let n2 = dag.add_node(create_node(2, "execution_time", 0));
        let n3 = dag.add_node(create_node(3, "execution_time", 0));
        dag.add_edge(n0, n1, 1);
        dag.add_edge(n0, n2, 1);
        dag.add_edge(n1, n3, 1);

        assert_eq!(dag.get_des_nodes(n0), Some(vec![n2, n1, n3]));
    }

    #[test]
    fn test_get_des_nodes_single() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "execution_time", 0));
        let n1 = dag.add_node(create_node(1, "execution_time", 0));
        dag.add_edge(n0, n1, 1);

        assert_eq!(dag.get_des_nodes(n0), Some(vec![n1]));
    }

    #[test]
    fn test_get_des_nodes_no_exist_des_nodes() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "execution_time", 0));

        assert_eq!(dag.get_des_nodes(n0), None);
    }

    #[test]
    #[should_panic]
    fn test_get_des_nodes_no_exist_target_node() {
        let dag = Graph::<NodeData, i32>::new();
        let invalid_node = NodeIndex::new(999);

        assert_eq!(dag.get_des_nodes(invalid_node), None);
    }

    #[test]
    fn test_get_dag_id_normal() {
        let mut dag = Graph::<NodeData, i32>::new();
        dag.add_node(create_node(0, "dag_id", 0));
        assert_eq!(dag.get_dag_param("dag_id"), 0);
    }

    #[test]
    #[should_panic]
    fn test_get_dag_id_no_exist_node() {
        let dag = Graph::<NodeData, i32>::new();
        dag.get_dag_param("dag_id");
    }

    #[test]
    fn test_set_dag_param_normal() {
        let mut dag = Graph::<NodeData, i32>::new();
        dag.add_node(create_node(0, "execution_time", 0));
        dag.add_node(create_node(1, "execution_time", 0));
        dag.set_dag_param("dag_id", 0);

        for node_i in dag.node_indices() {
            assert_eq!(dag[node_i].params["dag_id"], 0);
        }
    }

    #[test]
    #[should_panic]
    fn test_set_dag_param_no_exist_node() {
        let mut dag = Graph::<NodeData, i32>::new();
        dag.set_dag_param("dag_id", 0);
    }

    #[test]
    fn test_add_node_with_id_consistency_normal() {
        let mut dag = Graph::<NodeData, i32>::new();

        let n0 = dag.add_node_with_id_consistency(create_node(0, "execution_time", 3));
        let n1 = dag.add_node_with_id_consistency(create_node(1, "execution_time", 3));

        assert_eq!(dag[n0].id, 0);
        assert_eq!(dag[n1].id, 1);
    }

    #[test]
    #[should_panic]
    fn test_add_node_with_id_consistency_id_duplication() {
        let mut dag = Graph::<NodeData, i32>::new();
        dag.add_node_with_id_consistency(create_node(0, "execution_time", 3));
        dag.add_node_with_id_consistency(create_node(0, "execution_time", 3));
    }

    #[test]
    fn test_is_node_ready_normal() {
        let mut dag = Graph::<NodeData, i32>::new();
        let n0 = dag.add_node(create_node(0, "execution_time", 0));
        let n1 = dag.add_node(create_node(1, "execution_time", 0));
        dag.add_edge(n0, n1, 1);

        assert!(dag.is_node_ready(n0));
        assert!(!dag.is_node_ready(n1));
        dag.add_param(n1, "pre_done_count", 1);
        assert!(dag.is_node_ready(n1));
    }
}
