use icon_studio_lib::engine::builder::RenderCache;
use icon_studio_lib::model::shapes::ShapeType;
use icon_studio_lib::model::*;

fn default_project() -> IconProject {
    IconProject::default()
}

fn project_with_shape() -> IconProject {
    let mut project = IconProject::default();
    project.elements.push(Element::Shape(ShapeElement {
        common: CommonProps {
            id: "shape-1".into(),
            x: 50.0,
            y: 50.0,
            width: 100.0,
            height: 100.0,
            opacity: 1.0,
            rotation: 0.0,
            shadows: vec![],
                animation: None,
                blend_mode: None,
        clip_element_id: None,
        mask_element_id: None, locked: false, visible: true, svg_filter: None,
        overlay: None,
        },
        shape_type: ShapeType::Circle,
        fill: "#FF0000".into(),
        stroke: None,
        stroke_width: 0.0,
        border_radius: 0.0,
        stroke_dasharray: None,
        gradient: None,
    }));
    project
}

#[test]
fn cache_returns_same_svg_for_same_version() {
    let project = project_with_shape();
    let mut cache = RenderCache::default();

    let svg1 = cache.build(&project).expect("first build");
    let svg2 = cache.build(&project).expect("second build");
    assert_eq!(svg1, svg2, "Cache should return identical SVG for same version");
}

#[test]
fn cache_populated_after_first_build() {
    let project = default_project();
    let mut cache = RenderCache::default();

    let svg1 = cache.build(&project).expect("first build");
    let svg2 = cache.build(&project).expect("cached build");
    assert_eq!(svg1, svg2, "Second call should return same result from cache");
}

#[test]
fn version_change_triggers_rebuild() {
    let mut project = project_with_shape();
    let mut cache = RenderCache::default();

    let svg_v0 = cache.build(&project).expect("build v0");
    assert!(svg_v0.contains("<circle"), "Should contain circle, got: {}", svg_v0);

    project.bump_version();
    project.elements.clear();

    let svg_v1 = cache.build(&project).expect("build v1");
    assert!(!svg_v1.contains("<circle"), "Should NOT contain circle after clear, got: {}", svg_v1);
    assert_ne!(svg_v0, svg_v1, "SVG should differ after version change");
}

#[test]
fn invalidate_cache_forces_rebuild() {
    let project = default_project();
    let mut cache = RenderCache::default();

    let svg1 = cache.build(&project).expect("first build");
    cache.invalidate_cache();

    let svg2 = cache.build(&project).expect("rebuild after invalidation");
    assert_eq!(svg1, svg2, "Rebuilt SVG should match original");
}

#[test]
fn default_cache_handles_version_zero() {
    let project = default_project();
    assert_eq!(project.version, 0, "Default project should have version 0");

    let mut cache = RenderCache::default();
    let svg = cache.build(&project).expect("build");
    assert!(svg.contains("<svg"), "Should produce valid SVG, got: {}", svg);
}

#[test]
fn cache_multiple_version_changes() {
    let mut project = project_with_shape();
    let mut cache = RenderCache::default();

    let _svg_v0 = cache.build(&project).expect("build v0");

    project.bump_version();
    project.elements[0] = Element::Shape(ShapeElement {
        fill: "#00FF00".into(),
        ..match &project.elements[0] {
            Element::Shape(s) => s.clone(),
            _ => unreachable!(),
        }
    });
    let svg_v1 = cache.build(&project).expect("build v1");
    assert!(svg_v1.contains("#00FF00"), "Should contain new color, got: {}", svg_v1);

    project.bump_version();
    project.elements.clear();
    let svg_v2 = cache.build(&project).expect("build v2");
    assert!(!svg_v2.contains("<circle"), "Should not contain circle after clear");
}
