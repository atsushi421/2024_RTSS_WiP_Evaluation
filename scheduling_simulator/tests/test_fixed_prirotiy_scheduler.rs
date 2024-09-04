mod util;
use scheduling_simulator::{
    processor::{homogeneous::HomogeneousProcessor, processor_interface::Processor},
    scheduler::{
        dag_set_scheduler::{DAGSetSchedulerBase, PreemptiveType},
        fixed_priority_scheduler::FixedPriorityScheduler,
    },
    task::dag::DAG,
};
use util::create_sequential_dag_set;

#[test]
fn test_sequential_rm_scheduler_missed() {
    let mut dag_set = create_sequential_dag_set();
    for dag in dag_set.iter_mut() {
        let dag_period = dag.get_dag_period();
        for node in dag.node_weights_mut() {
            node.params.insert("priority".to_string(), dag_period);
        }
    }

    let processor = HomogeneousProcessor::new(1);
    let mut scheduler = FixedPriorityScheduler::new(&dag_set, &processor);
    scheduler.schedule(
        PreemptiveType::Preemptive {
            key: "priority".to_string(),
        },
        28,
    );

    let log = scheduler.get_log();

    let rt0 = log.dag_set_log[0].response_times.clone();
    assert_eq!(rt0, vec![2, 2]);
    let rt1 = log.dag_set_log[1].response_times.clone();
    assert_eq!(rt1, vec![8]);
}
