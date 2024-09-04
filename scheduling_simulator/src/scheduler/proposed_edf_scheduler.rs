use crate::getset_dag_set_scheduler;
use crate::graph_extension::GraphExtension;
use crate::processor::homogeneous::HomogeneousProcessor;
use crate::processor::processor_interface::Processor;
use crate::{graph_extension::NodeData, log::DAGSetSchedulerLog};
use petgraph::graph::Graph;
use std::cmp::Ordering;
use std::collections::VecDeque;

use super::dag_set_scheduler::DAGSetSchedulerBase;

pub struct GlobalEDFScheduler {
    dag_set: Vec<Graph<NodeData, i32>>,
    processor: HomogeneousProcessor,
    log: DAGSetSchedulerLog,
    current_time: i32,
}

impl DAGSetSchedulerBase<HomogeneousProcessor> for GlobalEDFScheduler {
    fn new(dag_set: &[Graph<NodeData, i32>], processor: &HomogeneousProcessor) -> Self {
        Self {
            dag_set: dag_set.to_vec(),
            processor: processor.clone(),
            log: DAGSetSchedulerLog::new(dag_set, processor.get_num_cores()),
            current_time: 0,
        }
    }

    fn update_params_when_release(dag: &mut Graph<NodeData, i32>, job_id: i32) {
        let sink_nodes = dag.get_sink_nodes();

        // Assign ref_absolute_deadline to sink nodes.
        for sink_i in sink_nodes.iter() {
            let sink_node = &mut dag[*sink_i];
            sink_node.params.insert(
                "ref_absolute_deadline".to_string(),
                sink_node.get_params_value("relative_deadline")
                    + job_id * sink_node.get_params_value("period"),
            );
        }

        // Assign ref_absolute_deadline to non-sink nodes.
        let non_sink_nodes = dag
            .node_indices()
            .filter(|node_i| !sink_nodes.contains(node_i))
            .collect::<Vec<_>>();
        for non_sink_i in non_sink_nodes {
            let ref_absolute_deadline = dag
                .get_des_nodes(non_sink_i)
                .unwrap()
                .iter()
                .filter(|x| sink_nodes.contains(x))
                .map(|x| dag[*x].get_params_value("ref_absolute_deadline"))
                .min()
                .unwrap();
            dag[non_sink_i]
                .params
                .insert("ref_absolute_deadline".to_string(), ref_absolute_deadline);
        }
    }

    fn sort_ready_queue(&self, ready_queue: &mut VecDeque<NodeData>) {
        ready_queue.make_contiguous().sort_by(|a, b| {
            match a
                .get_params_value("ref_absolute_deadline")
                .cmp(&b.get_params_value("ref_absolute_deadline"))
            {
                // If the keys are equal, compare by id
                Ordering::Equal => match a.id.partial_cmp(&b.id) {
                    // If the ids are also equal, compare by dag_id
                    Some(Ordering::Equal) => a
                        .get_params_value("dag_id")
                        .cmp(&b.get_params_value("dag_id")),
                    other => other.unwrap(),
                },
                other => other,
            }
        });
    }

    getset_dag_set_scheduler!(HomogeneousProcessor);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_extension::GraphExtension;
    use crate::scheduler::dag_set_scheduler::PreemptiveType;
    use crate::util::approx_eq;
    use crate::util::load_yaml;
    use std::{collections::BTreeMap, fs::remove_file};

    fn create_node(id: i32, key: &str, value: i32) -> NodeData {
        let mut params = BTreeMap::new();
        params.insert(key.to_string(), value);
        NodeData { id, params }
    }

    fn create_sample_dag() -> Graph<NodeData, i32> {
        let mut dag = Graph::<NodeData, i32>::new();
        // cX is the Xth critical node.
        let c0 = dag.add_node(create_node(0, "execution_time", 10));
        let c1 = dag.add_node(create_node(1, "execution_time", 20));
        let c2 = dag.add_node(create_node(2, "execution_time", 20));
        dag.add_param(c0, "period", 150);
        dag.add_param(c2, "end_to_end_deadline", 50);
        // nY_X is the Yth suc node of cX.
        let n0_0 = dag.add_node(create_node(3, "execution_time", 10));
        let n1_0 = dag.add_node(create_node(4, "execution_time", 10));

        // Create critical path edges
        dag.add_edge(c0, c1, 1);
        dag.add_edge(c1, c2, 1);

        // Create non-critical path edges
        dag.add_edge(c0, n0_0, 1);
        dag.add_edge(c0, n1_0, 1);
        dag.add_edge(n0_0, c2, 1);
        dag.add_edge(n1_0, c2, 1);

        dag
    }

    fn create_sample_dag2() -> Graph<NodeData, i32> {
        let mut dag = Graph::<NodeData, i32>::new();
        // cX is the Xth critical node.
        let c0 = dag.add_node(create_node(0, "execution_time", 10));
        let c1 = dag.add_node(create_node(1, "execution_time", 20));
        let c2 = dag.add_node(create_node(2, "execution_time", 20));
        dag.add_param(c0, "period", 100);
        dag.add_param(c2, "end_to_end_deadline", 60);
        // nY_X is the Yth suc node of cX.
        let n0_0 = dag.add_node(create_node(3, "execution_time", 10));

        // Create critical path edges
        dag.add_edge(c0, c1, 1);
        dag.add_edge(c1, c2, 1);

        // Create non-critical path edges
        dag.add_edge(c0, n0_0, 1);
        dag.add_edge(n0_0, c2, 1);

        dag
    }

    fn create_sample_dag3() -> Graph<NodeData, i32> {
        let mut dag = Graph::<NodeData, i32>::new();
        // cX is the Xth critical node.
        let c0 = dag.add_node(create_node(0, "execution_time", 5));
        let c1 = dag.add_node(create_node(1, "execution_time", 5));
        let c2 = dag.add_node(create_node(2, "execution_time", 5));
        dag.add_param(c0, "period", 25);
        dag.add_param(c1, "end_to_end_deadline", 15);
        // nY_X is the Yth suc node of cX.
        let n0_0 = dag.add_node(create_node(3, "execution_time", 5));
        let n1_0 = dag.add_node(create_node(4, "execution_time", 5));

        // Create critical path edges
        dag.add_edge(c0, c1, 1);
        dag.add_edge(c1, c2, 1);

        // Create non-critical path edges
        dag.add_edge(c0, n0_0, 1);
        dag.add_edge(c0, n1_0, 1);
        dag.add_edge(n0_0, c2, 1);
        dag.add_edge(n1_0, c2, 1);

        dag
    }

    #[test]
    fn test_global_edf_normal() {
        let mut dag = create_sample_dag();
        let mut dag2 = create_sample_dag2();
        dag.set_dag_param("dag_id", 0);
        dag2.set_dag_param("dag_id", 1);
        let dag_set = vec![dag, dag2];

        let processor = HomogeneousProcessor::new(4);

        let mut global_edf_scheduler = GlobalEDFScheduler::new(&dag_set, &processor);
        let time = global_edf_scheduler.schedule(PreemptiveType::NonPreemptive);

        assert_eq!(time, 300);

        let file_path = global_edf_scheduler.dump_log("../lib/tests", "edf_normal_test");
        let yaml_docs = load_yaml(&file_path);
        let yaml_doc = &yaml_docs[0];

        // Check the value of processor_info
        assert_eq!(
            yaml_doc["processor_info"]["number_of_cores"]
                .as_i64()
                .unwrap(),
            4
        );

        // Check the value of dag_set_log
        let dag_set_log = &yaml_doc["dag_set_log"][0];
        assert_eq!(dag_set_log["dag_id"].as_i64().unwrap(), 0);
        let release_time = &dag_set_log["release_time"];
        assert_eq!(release_time[0].as_i64().unwrap(), 0);
        assert_eq!(release_time[1].as_i64().unwrap(), 150);
        let finish_time = &dag_set_log["finish_time"];
        assert_eq!(finish_time[0].as_i64().unwrap(), 50);
        assert_eq!(finish_time[1].as_i64().unwrap(), 200);

        // Check the value of node_set_logs
        let node_set_logs = &yaml_doc["node_set_logs"][0];
        assert_eq!(node_set_logs[0]["core_id"].as_i64().unwrap(), 0);
        assert_eq!(node_set_logs[0]["dag_id"].as_i64().unwrap(), 0);
        assert_eq!(node_set_logs[0]["node_id"].as_i64().unwrap(), 0);
        // start_time
        assert_eq!(node_set_logs[0]["event_time"].as_str().unwrap(), "0");
        // finish_time
        assert_eq!(node_set_logs[1]["event_time"].as_str().unwrap(), "10");

        // Check the value of processor_log
        let processor_log = &yaml_doc["processor_log"];
        assert!(approx_eq(
            processor_log["average_utilization"].as_f64().unwrap(),
            0.2666667
        ));
        assert!(approx_eq(
            processor_log["variance_utilization"].as_f64().unwrap(),
            0.0605555
        ));

        // Check the value of core_logs
        let core_logs = &processor_log["core_logs"][0];
        assert_eq!(core_logs["core_id"].as_i64().unwrap(), 0);
        assert_eq!(core_logs["total_proc_time"].as_i64().unwrap(), 200);
        assert_eq!(core_logs["utilization"].as_f64().unwrap(), 0.6666667);

        remove_file(file_path).unwrap();
    }

    #[test]
    fn test_global_edf_preemptive() {
        let mut dag = create_sample_dag();
        let mut dag3 = create_sample_dag3();
        dag.set_dag_param("dag_id", 0);
        dag3.set_dag_param("dag_id", 1);
        let dag_set = vec![dag, dag3];

        let processor = HomogeneousProcessor::new(2);

        let mut global_edf_scheduler = GlobalEDFScheduler::new(&dag_set, &processor);
        let time = global_edf_scheduler.schedule(PreemptiveType::Preemptive {
            key: "node_absolute_deadline".to_string(),
        });

        assert_eq!(time, 150);

        let file_path = global_edf_scheduler.dump_log("../lib/tests", "edf_preemptive_test");
        let yaml_docs = load_yaml(&file_path);
        let yaml_doc = &yaml_docs[0];

        // Check the value of processor_info
        assert_eq!(
            yaml_doc["processor_info"]["number_of_cores"]
                .as_i64()
                .unwrap(),
            2
        );

        // Check the value of dag_set_log
        let dag_set_log = &yaml_doc["dag_set_log"][0];
        assert_eq!(dag_set_log["dag_id"].as_i64().unwrap(), 0);
        assert_eq!(dag_set_log["release_time"][0].as_i64().unwrap(), 0);
        assert_eq!(dag_set_log["finish_time"][0].as_i64().unwrap(), 80);
        assert_eq!(dag_set_log["response_time"][0].as_i64().unwrap(), 80);

        // Check the value of node_set_logs
        let node_set_logs = &yaml_doc["node_set_logs"][0];
        assert_eq!(node_set_logs[0]["core_id"].as_i64().unwrap(), 1);
        assert_eq!(node_set_logs[0]["dag_id"].as_i64().unwrap(), 0);
        assert_eq!(node_set_logs[0]["node_id"].as_i64().unwrap(), 0);
        // start_time
        assert_eq!(node_set_logs[0]["event_time"].as_str().unwrap(), "0");
        // preempt_time
        assert_eq!(node_set_logs[1]["event_time"].as_str().unwrap(), "5");
        // resume_time
        assert_eq!(node_set_logs[2]["event_time"].as_str().unwrap(), "10");
        // finish_time
        assert_eq!(node_set_logs[3]["event_time"].as_str().unwrap(), "15");

        // Check the value of processor_log
        let processor_log = &yaml_doc["processor_log"];
        assert_eq!(
            processor_log["average_utilization"].as_f64().unwrap(),
            0.73333335
        );
        assert_eq!(
            processor_log["variance_utilization"].as_f64().unwrap(),
            0.017777776
        );

        // Check the value of core_logs
        let core_logs = &processor_log["core_logs"][0];
        assert_eq!(core_logs["core_id"].as_i64().unwrap(), 0);
        assert_eq!(core_logs["total_proc_time"].as_i64().unwrap(), 130);
        assert_eq!(core_logs["utilization"].as_f64().unwrap(), 0.8666667);

        remove_file(file_path).unwrap();
    }
}
