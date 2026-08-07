use omniversa_core::*;

#[tokio::test]
async fn test_engine_initialization() {
    let config = EngineConfig::default();
    let engine = OmniversaEngine::new(1000, config).await.unwrap();
    let snapshot = engine.fragility_snapshot().await;
    assert!(snapshot.fragility_score >= 0.0);
    assert!(snapshot.fragility_score <= 1.0);
}

#[tokio::test]
async fn test_manifold_operations() {
    let mut manifold = PhaseSpaceManifold::new(6);
    let point = ManifoldPoint::new(0);
    manifold.ingest(point);
    let pcs = manifold.principal_components();
    assert!(!pcs.is_empty());
}
