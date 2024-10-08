use crate::task::dag::Node;

use super::core::ProcessResult;

pub trait Processor {
    fn new(num_cores: usize) -> Self;
    fn allocate(&mut self, core_id: usize, node_data: &Node);
    fn process(&mut self) -> Vec<ProcessResult>;
    fn get_num_cores(&self) -> usize;
    fn get_idle_core_i(&self) -> Option<usize>;
    fn get_num_idle_cores(&self) -> usize;
    fn preempt(&mut self, core_id: usize) -> Node;
    fn get_max_and_index(&self, key: &str) -> Option<(i32, usize)>;
}
