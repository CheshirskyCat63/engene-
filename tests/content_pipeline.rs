#[test]
fn import_pipeline_register_and_track() {
    use engene::content::import::asset_pipeline::{ImportPipeline, AssetType};
    use std::path::Path;

    let mut pipeline = ImportPipeline::new(4);
    let id = pipeline.register_source(Path::new("test_asset.png"), AssetType::Texture, "default");
    assert!(id > 0);
    assert_eq!(pipeline.asset_count(), 1);

    let id2 = pipeline.register_source(Path::new("test_asset.png"), AssetType::Texture, "default");
    assert_eq!(id, id2, "same path should return same ID");
}

#[test]
fn import_pipeline_dependencies() {
    use engene::content::import::asset_pipeline::{ImportPipeline, AssetType};
    use std::path::Path;

    let mut pipeline = ImportPipeline::new(2);
    let model = pipeline.register_source(Path::new("model.obj"), AssetType::Model, "");
    let tex = pipeline.register_source(Path::new("diffuse.png"), AssetType::Texture, "");
    pipeline.add_dependency(model, tex);

    let record = pipeline.get_record(model).unwrap();
    assert!(record.dependencies.contains(&tex));
}

#[test]
fn cook_pipeline_variants() {
    use engene::content::cooking::cook_pipeline::{CookPipeline, CookVariant};
    use std::path::Path;

    let mut pipeline = CookPipeline::new(Path::new("out/"));
    let data = b"cooked_data";

    pipeline.cook_asset(1, CookVariant::Dev, data);
    pipeline.cook_asset(1, CookVariant::Shipping, data);
    pipeline.cook_asset(1, CookVariant::LowSpec, data);

    assert_eq!(pipeline.artifact_count(), 3);
    assert!(pipeline.get_artifact(1, &CookVariant::Dev).is_some());
    assert!(pipeline.get_artifact(1, &CookVariant::LowSpec).is_some());
}

#[test]
fn dependency_graph_invalidation() {
    use engene::content::cooking::dependency_graph::ContentDependencyGraph;

    let mut graph = ContentDependencyGraph::new();
    graph.add_dependency(2, 1);
    graph.add_dependency(3, 1);
    graph.add_dependency(4, 2);

    let affected = graph.invalidate(1);
    assert!(affected.contains(&1));
    assert!(affected.contains(&2));
    assert!(affected.contains(&3));
    assert!(affected.contains(&4));
}

#[test]
fn dependency_graph_dirty_tracking() {
    use engene::content::cooking::dependency_graph::ContentDependencyGraph;

    let mut graph = ContentDependencyGraph::new();
    graph.add_dependency(2, 1);
    graph.add_dependency(3, 2);

    assert!(!graph.is_dirty(1));
    graph.invalidate_and_mark_dirty(1);
    assert!(graph.is_dirty(1));
    assert!(graph.is_dirty(2));
    assert!(graph.is_dirty(3));

    graph.clear_dirty(1);
    assert!(!graph.is_dirty(1));
    assert!(graph.is_dirty(2));
}

#[test]
fn prefab_registry_inheritance() {
    use engene::content::prefabs::prefab::{PrefabDescriptor, PrefabEntity, SpecVariant};
    use engene::content::prefabs::prefab_registry::PrefabRegistry;

    let mut registry = PrefabRegistry::new();

    let base = PrefabDescriptor {
        name: "wall_base".to_string(),
        base: None,
        spec_variant: Some(SpecVariant::Full),
        root: PrefabEntity {
            name: Some("Wall".to_string()),
            components: vec![],
            children: vec![],
        },
        tags: vec!["structure".to_string()],
    };

    let derived = PrefabDescriptor {
        name: "wall_brick".to_string(),
        base: Some("wall_base".to_string()),
        spec_variant: None,
        root: PrefabEntity {
            name: Some("BrickWall".to_string()),
            components: vec![],
            children: vec![],
        },
        tags: vec!["brick".to_string()],
    };

    registry.register(base);
    registry.register(derived);

    let resolved = registry.resolve_inheritance("wall_brick").unwrap();
    assert_eq!(resolved.name, "wall_brick");
    assert!(resolved.tags.contains(&"structure".to_string()));
    assert!(resolved.tags.contains(&"brick".to_string()));
}

#[test]
fn content_validator_missing_base() {
    use engene::content::prefabs::prefab::{PrefabDescriptor, PrefabEntity};
    use engene::content::prefabs::prefab_registry::PrefabRegistry;
    use engene::content::validation::content_validator::validate_prefabs;

    let mut registry = PrefabRegistry::new();
    registry.register(PrefabDescriptor {
        name: "orphan".to_string(),
        base: Some("nonexistent".to_string()),
        spec_variant: None,
        root: PrefabEntity { name: None, components: vec![], children: vec![] },
        tags: vec![],
    });

    let result = validate_prefabs(&registry);
    assert!(!result.is_valid());
    assert!(result.errors.iter().any(|e| e.contains("nonexistent")));
}

#[test]
fn chunk_package_priority_ordering() {
    use engene::world::chunk_package::*;
    use engene::world::streaming::ChunkCoord;

    let mut pkg = ChunkPackage::new(ChunkCoord { x: 0, z: 0 }, 1);
    pkg.add_bundle(Bundle {
        kind: BundleKind::Vegetation,
        size_bytes: 100,
        compressed_size: 50,
        asset_refs: vec![],
    });
    pkg.add_bundle(Bundle {
        kind: BundleKind::Collision,
        size_bytes: 200,
        compressed_size: 100,
        asset_refs: vec![],
    });

    let sorted = pkg.bundles_by_priority();
    assert_eq!(sorted[0].kind, BundleKind::Collision);
    assert_eq!(sorted[1].kind, BundleKind::Vegetation);
}

#[test]
fn io_budget_streamer_respects_limits() {
    use engene::world::io_budget::*;
    use engene::world::chunk_package::{StreamPriority, BundleKind};
    use engene::world::streaming::ChunkCoord;

    let budget = IoBudget {
        read_bytes_per_frame: 100,
        decompress_bytes_per_frame: 200,
        upload_bytes_per_frame: 100,
    };
    let mut streamer = BudgetedStreamer::new(budget);

    streamer.enqueue(StreamRequest {
        coord: ChunkCoord { x: 0, z: 0 },
        bundle_kind: BundleKind::Collision,
        priority: StreamPriority::Critical,
        size_bytes: 80,
        compressed_bytes: 60,
        distance_sq: 100.0,
    });
    streamer.enqueue(StreamRequest {
        coord: ChunkCoord { x: 1, z: 0 },
        bundle_kind: BundleKind::Vegetation,
        priority: StreamPriority::Cosmetic,
        size_bytes: 80,
        compressed_bytes: 60,
        distance_sq: 200.0,
    });

    let result = streamer.process_frame();
    assert_eq!(result.loaded.len(), 1, "only one fits in budget");
    assert_eq!(result.loaded[0].1, BundleKind::Collision, "critical first");
    assert_eq!(result.deferred, 1);
}
