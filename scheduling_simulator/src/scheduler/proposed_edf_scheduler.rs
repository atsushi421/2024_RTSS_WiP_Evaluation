use crate::dag_set_scheduler_common;
use crate::log::DAGSetSchedulerLog;
use crate::processor::homogeneous::HomogeneousProcessor;
use crate::processor::processor_interface::Processor;
use crate::task::dag::{Node, DAG};
use petgraph::graph::Graph;
use std::cmp::Ordering;
use std::collections::VecDeque;

use super::dag_set_scheduler::DAGSetSchedulerBase;

pub struct GlobalEDFScheduler {
    dag_set: Vec<Graph<Node, i32>>,
    processor: HomogeneousProcessor,
    log: DAGSetSchedulerLog,
    current_time: i32,
}

impl DAGSetSchedulerBase<HomogeneousProcessor> for GlobalEDFScheduler {
    fn update_params_when_release(dag: &mut Graph<Node, i32>, job_id: i32) {
        let sink_nodes = dag.get_sink();

        // Assign ref_absolute_deadline to sink nodes.
        for sink_i in sink_nodes.iter() {
            dag.set_param(
                *sink_i,
                "ref_absolute_deadline",
                dag[*sink_i].get_value("relative_deadline") + job_id * dag.get_dag_period(),
            )
        }

        // Assign ref_absolute_deadline to non-sink nodes.
        let non_sink_nodes = dag
            .node_indices()
            .filter(|node_i| !sink_nodes.contains(node_i))
            .collect::<Vec<_>>();
        for non_sink_i in non_sink_nodes {
            let ref_absolute_deadline = dag
                .get_des(non_sink_i)
                .iter()
                .filter(|x| sink_nodes.contains(x))
                .map(|x| dag[*x].get_value("ref_absolute_deadline"))
                .min()
                .unwrap();
            dag.set_param(non_sink_i, "ref_absolute_deadline", ref_absolute_deadline);
        }
    }

    fn sort_ready_queue(&self, ready_queue: &mut VecDeque<Node>) {
        ready_queue.make_contiguous().sort_by(|a, b| {
            match a
                .get_value("ref_absolute_deadline")
                .cmp(&b.get_value("ref_absolute_deadline"))
            {
                // If the keys are equal, compare by id
                Ordering::Equal => match a.id.partial_cmp(&b.id) {
                    // If the ids are also equal, compare by dag_id
                    Some(Ordering::Equal) => a.get_value("dag_id").cmp(&b.get_value("dag_id")),
                    other => other.unwrap(),
                },
                other => other,
            }
        });
    }

    dag_set_scheduler_common!(HomogeneousProcessor);
}
