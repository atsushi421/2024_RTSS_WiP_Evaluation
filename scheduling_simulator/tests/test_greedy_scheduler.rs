use scheduling_simulator::{
    processor::{homogeneous::HomogeneousProcessor, processor_interface::Processor},
    scheduler::{
        dag_set_scheduler::DAGSetSchedulerBase, fixed_priority_scheduler::FixedPriorityScheduler,
    },
};
use util::create_simple_dag;

mod util;

#[test]
fn test_work_conserving_scheduler_once_3core() {
    let mut dag = create_simple_dag();
    for node in dag.node_weights_mut() {
        node.params.insert("priority".to_string(), 0);
    }
    let dag_set = vec![dag];

    let processor = HomogeneousProcessor::new(3);
    let mut scheduler = FixedPriorityScheduler::new(&dag_set, &processor);
    scheduler.schedule(
        scheduling_simulator::scheduler::dag_set_scheduler::PreemptiveType::NonPreemptive,
        8,
    );

    let log = scheduler.get_log();
    assert!(!log.deadline_missed);
    assert_eq!(log.dag_set_log[0].response_times.clone(), vec![8]);
}

#[test]
fn test_work_conserving_scheduler_once_2core() {
    let mut dag = create_simple_dag();
    for node in dag.node_weights_mut() {
        node.params.insert("priority".to_string(), 0);
    }
    let dag_set = vec![dag];

    let processor = HomogeneousProcessor::new(2);
    let mut scheduler = FixedPriorityScheduler::new(&dag_set, &processor);
    scheduler.schedule(
        scheduling_simulator::scheduler::dag_set_scheduler::PreemptiveType::NonPreemptive,
        11,
    );

    let log = scheduler.get_log();
    assert!(!log.deadline_missed);
    assert_eq!(log.dag_set_log[0].response_times.clone(), vec![11]);
}

#[test]
fn test_work_conserving_scheduler_duration_2core() {
    let mut dag = create_simple_dag();
    for node in dag.node_weights_mut() {
        node.params.insert("priority".to_string(), 0);
    }
    let dag_set = vec![dag];

    let processor = HomogeneousProcessor::new(2);
    let mut scheduler = FixedPriorityScheduler::new(&dag_set, &processor);
    scheduler.schedule(
        scheduling_simulator::scheduler::dag_set_scheduler::PreemptiveType::NonPreemptive,
        22,
    );

    let log = scheduler.get_log();
    assert!(!log.deadline_missed);
    assert_eq!(log.dag_set_log[0].response_times.clone(), vec![11, 11]);
}
