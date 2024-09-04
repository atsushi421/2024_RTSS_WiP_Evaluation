use std::collections::VecDeque;

use crate::dag_set_scheduler_common;
use crate::log::DAGSetSchedulerLog;
use crate::processor::homogeneous::HomogeneousProcessor;
use crate::processor::processor_interface::Processor;
use crate::task::dag::Node;
use petgraph::graph::Graph;

use super::dag_set_scheduler::DAGSetSchedulerBase;

pub struct FixedPriorityScheduler {
    dag_set: Vec<Graph<Node, i32>>,
    processor: HomogeneousProcessor,
    log: DAGSetSchedulerLog,
    current_time: i32,
}

impl DAGSetSchedulerBase<HomogeneousProcessor> for FixedPriorityScheduler {
    dag_set_scheduler_common!(HomogeneousProcessor);

    fn update_params_when_release(_dag: &mut Graph<Node, i32>, _job_id: i32) {
        // Do nothing.
    }

    fn sort_ready_queue(&self, ready_queue: &mut VecDeque<Node>) {
        ready_queue
            .make_contiguous()
            .sort_by_key(|a| a.get_value("priority"));
    }
}
