//! Homogeneous processor module. This module uses Core struct.

use super::core::{Core, ProcessResult};
use crate::{processor::processor_interface::Processor, task::graph_extension::NodeData};

#[derive(Clone, Debug)]
pub struct HomogeneousProcessor {
    pub cores: Vec<Core>,
}

impl Processor for HomogeneousProcessor {
    fn new(num_cores: usize) -> Self {
        Self {
            cores: vec![Core::default(); num_cores],
        }
    }

    fn allocate(&mut self, core_id: usize, node: &NodeData) {
        self.cores[core_id].allocate(node)
    }

    fn process(&mut self) -> Vec<ProcessResult> {
        self.cores.iter_mut().map(|core| core.process()).collect()
    }

    fn get_num_cores(&self) -> usize {
        self.cores.len()
    }

    fn get_num_idle_cores(&self) -> usize {
        self.cores.iter().filter(|core| core.is_idle).count()
    }

    fn get_idle_core_i(&self) -> Option<usize> {
        for (index, core) in self.cores.iter().enumerate() {
            if core.is_idle {
                return Some(index);
            }
        }
        None
    }

    fn preempt(&mut self, core_id: usize) -> NodeData {
        self.cores[core_id].preempt()
    }

    fn get_max_and_index(&self, key: &str) -> Option<(i32, usize)> {
        self.cores
            .iter()
            .enumerate()
            .filter_map(|(i, core)| {
                let node_data = core.processing_node.as_ref()?;
                let value = node_data.get_params_value(key);
                Some((value, i))
            })
            .max_by_key(|&(value, _)| value)
    }
}

#[cfg(test)]
mod tests_homogeneous_processor {
    use super::*;
    use std::collections::BTreeMap;

    fn create_node(key: &str, value: Option<i32>) -> NodeData {
        let mut params = BTreeMap::new();
        params.insert(key.to_string(), value.unwrap_or_default());
        NodeData { id: 0, params }
    }

    #[test]
    fn test_get_max_and_index() {
        let mut processor = HomogeneousProcessor::new(2);
        const NODE0_ET: i32 = 10;
        const NODE1_ET: i32 = 11;
        processor.allocate(0, &create_node("execution_time", Some(NODE0_ET)));
        processor.allocate(1, &create_node("execution_time", Some(NODE1_ET)));

        assert_eq!(
            processor.get_max_and_index("execution_time"),
            Some((NODE1_ET, 1))
        );
    }
}
