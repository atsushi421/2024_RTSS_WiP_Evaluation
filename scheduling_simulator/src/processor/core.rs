//! This module contains the definition of the core and the process result enum
use crate::{
    processor::core::ProcessResult::{Done, Idle, InProgress},
    task::dag::Node,
};
use core::panic;

use getset::{CopyGetters, Getters};

#[derive(Debug, PartialEq, Clone)]
pub enum ProcessResult {
    Idle,
    InProgress,
    Done(Node),
}

#[derive(Clone, CopyGetters, Getters, Debug)]
pub struct Core {
    #[get_copy = "pub with_prefix"]
    pub is_idle: bool,
    #[get = "pub with_prefix"]
    pub processing_node: Option<Node>,
    pub remain_proc_time: i32,
}

impl Default for Core {
    fn default() -> Self {
        Self {
            is_idle: true,
            processing_node: None,
            remain_proc_time: 0,
        }
    }
}

impl Core {
    pub fn allocate(&mut self, node: &Node) {
        if !self.is_idle {
            panic!("The node is already allocated");
        }

        self.is_idle = false;
        self.processing_node = Some(node.clone());
        if let Some(exec_time) = node.params.get("execution_time") {
            self.remain_proc_time = *exec_time;
        } else {
            panic!("Node {} does not have execution_time", node.id);
        }
    }

    pub fn process(&mut self) -> ProcessResult {
        if self.is_idle {
            return Idle;
        }

        self.remain_proc_time -= 1;
        if self.remain_proc_time == 0 {
            self.is_idle = true;
            let finish_node_data = self.processing_node.take().unwrap();
            return Done(finish_node_data);
        }

        InProgress
    }

    pub fn preempt(&mut self) -> Node {
        if self.is_idle {
            panic!("Although the core is idle, preempt is called");
        }

        let mut node_data = self.processing_node.take().unwrap();
        node_data
            .params
            .insert("execution_time".to_string(), self.remain_proc_time);
        node_data.params.insert("is_preempted".to_string(), 1);
        self.is_idle = true;
        self.remain_proc_time = 0;
        node_data
    }
}

#[cfg(test)]
mod tests_core {
    use super::*;
    use std::collections::BTreeMap;

    fn create_node(key: &str, value: Option<i32>) -> Node {
        let mut params = BTreeMap::new();
        params.insert(key.to_string(), value.unwrap_or_default());
        Node { id: 0, params }
    }

    #[test]
    fn test_allocate_normal() {
        const DUMMY_ET: i32 = 10;
        let dummy_node = create_node("execution_time", Some(DUMMY_ET));
        let mut core = Core::default();

        core.allocate(&dummy_node);
        assert!(!core.is_idle);
        assert_eq!(core.processing_node, Some(dummy_node));
        assert_eq!(core.remain_proc_time, DUMMY_ET);
    }

    #[test]
    #[should_panic]
    fn test_allocate_already_allocated() {
        let mut core = Core::default();
        core.allocate(&create_node("execution_time", None));
        core.allocate(&create_node("execution_time", None));
    }

    #[test]
    #[should_panic]
    fn test_allocate_node_no_has_execution_time() {
        let mut core = Core::default();
        core.allocate(&create_node("no_execution_time", None));
    }

    #[test]
    fn test_process_in_progress() {
        const DUMMY_ET: i32 = 10;
        let mut core = Core::default();
        core.allocate(&create_node("execution_time", Some(DUMMY_ET)));
        assert_eq!(core.process(), InProgress);
        assert_eq!(core.remain_proc_time, DUMMY_ET - 1);
    }

    #[test]
    fn test_process_idle() {
        let mut core = Core::default();
        assert_eq!(core.process(), Idle);
    }

    #[test]
    fn test_process_done() {
        let dummy_node = create_node("execution_time", Some(1));
        let mut core = Core::default();
        core.allocate(&dummy_node);
        assert_eq!(core.process(), Done(dummy_node));
        assert!(core.is_idle);
        assert_eq!(core.processing_node, None);
        assert_eq!(core.remain_proc_time, 0);
    }

    #[test]
    fn test_preempt() {
        const DUMMY_ET: i32 = 10;
        let dummy_node = create_node("execution_time", Some(DUMMY_ET));
        let mut core = Core::default();

        core.allocate(&dummy_node);
        core.process();

        let preempted_node = core.preempt();
        assert_eq!(
            preempted_node.get_value("execution_time"),
            DUMMY_ET - 1
        );
        assert_eq!(preempted_node.get_value("is_preempted"), 1);
        assert!(core.is_idle);
        assert_eq!(core.processing_node, None);
        assert_eq!(core.remain_proc_time, 0);
    }
}
