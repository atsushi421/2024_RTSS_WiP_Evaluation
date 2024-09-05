use num_integer::lcm;
use petgraph::Graph;

use super::dag::{Node, DAG};

pub fn get_hyper_period(dag_set: &[Graph<Node, i32>]) -> i32 {
    let mut hyper_period = 1;
    for dag in dag_set {
        let dag_period = dag.get_dag_param("period");
        hyper_period = lcm(hyper_period, dag_period);
    }
    hyper_period
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn create_dag_with_period(period: i32) -> Graph<Node, i32> {
        let mut dag = Graph::<Node, i32>::new();
        let mut params = BTreeMap::new();
        params.insert("period".to_string(), period);
        dag.add_node(Node { id: 0, params });

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
