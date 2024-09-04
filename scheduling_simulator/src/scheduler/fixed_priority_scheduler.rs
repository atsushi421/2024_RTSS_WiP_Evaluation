use crate::getset_dag_set_scheduler;
use crate::processor::homogeneous::HomogeneousProcessor;
use crate::processor::processor_interface::Processor;
use crate::{graph_extension::NodeData, log::DAGSetSchedulerLog};
use petgraph::graph::Graph;

use super::dag_set_scheduler::DAGSetSchedulerBase;

pub struct FixedPriorityScheduler {
    dag_set: Vec<Graph<NodeData, i32>>,
    processor: HomogeneousProcessor,
    log: DAGSetSchedulerLog,
    current_time: i32,
}

impl DAGSetSchedulerBase<HomogeneousProcessor> for FixedPriorityScheduler {
    fn new(dag_set: &[Graph<NodeData, i32>], processor: &HomogeneousProcessor) -> Self {
        Self {
            dag_set: dag_set.to_vec(),
            processor: processor.clone(),
            log: DAGSetSchedulerLog::new(dag_set, processor.get_num_cores()),
            current_time: 0,
        }
    }

    fn update_params_when_release(_dag: &mut Graph<NodeData, i32>, _job_id: i32) {
        // Do nothing.
    }

    fn sort_ready_queue(&self, ready_queue: &mut std::collections::VecDeque<NodeData>) {
        ready_queue.make_contiguous().sort_by(|a, b| {
            a.get_params_value("priority")
                .cmp(&b.get_params_value("priority"))
        });
    }

    getset_dag_set_scheduler!(HomogeneousProcessor);
}
