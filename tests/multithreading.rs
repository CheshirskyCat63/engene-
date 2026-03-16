#[test]
fn worker_pool_creation() {
    use engene::core::jobs::WorkerPool;

    let pool = WorkerPool::new();
    assert!(pool.worker_count() >= 2);
}

#[test]
fn job_graph_empty() {
    use engene::core::job_graph::JobGraph;

    let graph = JobGraph::new();
    let _ = graph.pool();
}

#[test]
fn job_graph_add_and_execute() {
    use engene::core::job_graph::JobGraph;

    let graph = JobGraph::new();
    let (a, b) = graph.par_join(|| 10 + 5, || 20 - 3);
    assert_eq!(a, 15);
    assert_eq!(b, 17);
}

#[test]
fn ownership_map_registration() {
    use engene::core::ownership_map::{OwnershipMap, ResourceOwnership, ThreadSafety};

    let mut map = OwnershipMap::new();
    map.register_resource(ResourceOwnership {
        resource_name: "TestResource".to_string(),
        type_id: std::any::TypeId::of::<u32>(),
        owner_system: "SystemA".to_string(),
        readers: vec!["SystemB".to_string()],
        writers: vec!["SystemA".to_string()],
        mutation_timing: vec![],
        thread_safety: ThreadSafety::MainThreadOnly,
        notes: String::new(),
    });

    let owned = map.resources_owned_by("SystemA");
    assert_eq!(owned.len(), 1);
    assert_eq!(owned[0].resource_name, "TestResource");

    let readers = map.systems_reading("TestResource");
    assert_eq!(readers, vec!["SystemB"]);
}

#[test]
fn parallel_validation_disjoint() {
    use engene::core::parallel_validation::validate_systems;
    use engene::core::system_descriptor::SystemDescriptor;

    struct ResA;
    struct ResB;

    let a = SystemDescriptor::new("SystemA")
        .with_parallel(true)
        .writes_resource::<ResA>();
    let b = SystemDescriptor::new("SystemB")
        .with_parallel(true)
        .writes_resource::<ResB>();

    let report = validate_systems(&[a, b], false, false);
    assert!(!report.has_errors());
}
