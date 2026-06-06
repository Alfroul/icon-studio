use criterion::{black_box, criterion_group, criterion_main, Criterion};
use icon_studio_lib::engine::{builder, renderer};
use icon_studio_lib::engine::builder::RenderCache;
use icon_studio_lib::engine::text_measure;
use icon_studio_lib::model::history::{AddElementCommand, CommandHistory};
use icon_studio_lib::model::shapes::ShapeType;
use icon_studio_lib::model::{
    Canvas, CommonProps, Element, Gradient, GradientKind, IconProject, Shadow, ShapeElement, TextElement,
    IconElement, ExportConfig,
};
use std::collections::HashMap;

fn make_small_project() -> IconProject {
    IconProject {
        schema_version: "1.0".into(),
        canvas: Canvas::default(),
        elements: vec![Element::Shape(ShapeElement {
            common: CommonProps {
                id: "shape-1".into(),
                x: 156.0,
                y: 156.0,
                width: 200.0,
                height: 200.0,
                opacity: 1.0,
                rotation: 0.0,
                shadows: vec![],
                animation: None,
                blend_mode: None,
                clip_element_id: None,
                mask_element_id: None,
                locked: false,
                visible: true,
                svg_filter: None,
            },
            shape_type: ShapeType::Circle,
            fill: "#4A90D9".into(),
            stroke: None,
            stroke_width: 0.0,
            border_radius: 0.0,
            stroke_dasharray: None,
            gradient: None,
        })],
        exports: ExportConfig::default(),
        templates: HashMap::new(),
        next_element_id: 2,
        version: 0,
        pages: vec![],
        symbols: HashMap::new(),
    }
}

fn make_medium_project() -> IconProject {
    IconProject {
        schema_version: "1.0".into(),
        canvas: Canvas::default(),
        elements: vec![
            Element::Shape(ShapeElement {
                common: CommonProps {
                    id: "shape-1".into(),
                    x: 50.0, y: 50.0, width: 150.0, height: 150.0,
                    opacity: 0.9, rotation: 0.0,
                    shadows: vec![Shadow { color: "#00000040".into(), blur: 8.0, offset_x: 0.0, offset_y: 4.0, inset: false }],
                    animation: None, blend_mode: None, clip_element_id: None,
                    mask_element_id: None, locked: false, visible: true, svg_filter: None,
                },
                shape_type: ShapeType::Circle,
                fill: "#FF5733".into(), stroke: None, stroke_width: 0.0,
                border_radius: 0.0, stroke_dasharray: None,
                gradient: Some(Gradient {
                    gradient_type: GradientKind::Linear,
                    colors: vec!["#FF5733".into(), "#33C1FF".into()],
                    angle: 45.0, stops: vec![],
                }),
            }),
            Element::Shape(ShapeElement {
                common: CommonProps {
                    id: "shape-2".into(),
                    x: 250.0, y: 50.0, width: 120.0, height: 80.0,
                    opacity: 1.0, rotation: 15.0, shadows: vec![],
                    animation: None, blend_mode: None, clip_element_id: None,
                    mask_element_id: None, locked: false, visible: true, svg_filter: None,
                },
                shape_type: ShapeType::Rect,
                fill: "#27AE60".into(), stroke: Some("#1E8449".into()), stroke_width: 2.0,
                border_radius: 0.0, stroke_dasharray: None, gradient: None,
            }),
            Element::Shape(ShapeElement {
                common: CommonProps {
                    id: "shape-3".into(),
                    x: 100.0, y: 250.0, width: 100.0, height: 100.0,
                    opacity: 1.0, rotation: 0.0,
                    shadows: vec![Shadow { color: "#00000060".into(), blur: 12.0, offset_x: 2.0, offset_y: 6.0, inset: false }],
                    animation: None, blend_mode: None, clip_element_id: None,
                    mask_element_id: None, locked: false, visible: true, svg_filter: None,
                },
                shape_type: ShapeType::Star,
                fill: "#F1C40F".into(), stroke: None, stroke_width: 0.0,
                border_radius: 0.0, stroke_dasharray: None,
                gradient: Some(Gradient {
                    gradient_type: GradientKind::Radial,
                    colors: vec!["#F1C40F".into(), "#E74C3C".into(), "#8E44AD".into()],
                    angle: 0.0, stops: vec![0.0, 0.5, 1.0],
                }),
            }),
            Element::Shape(ShapeElement {
                common: CommonProps {
                    id: "shape-4".into(),
                    x: 280.0, y: 200.0, width: 130.0, height: 130.0,
                    opacity: 0.85, rotation: 30.0, shadows: vec![],
                    animation: None, blend_mode: None, clip_element_id: None,
                    mask_element_id: None, locked: false, visible: true, svg_filter: None,
                },
                shape_type: ShapeType::Hexagon,
                fill: "#8E44AD".into(), stroke: None, stroke_width: 0.0,
                border_radius: 0.0, stroke_dasharray: None, gradient: None,
            }),
            Element::Shape(ShapeElement {
                common: CommonProps {
                    id: "shape-5".into(),
                    x: 380.0, y: 100.0, width: 80.0, height: 80.0,
                    opacity: 1.0, rotation: 0.0,
                    shadows: vec![Shadow { color: "#00000030".into(), blur: 6.0, offset_x: 1.0, offset_y: 3.0, inset: false }],
                    animation: None, blend_mode: None, clip_element_id: None,
                    mask_element_id: None, locked: false, visible: true, svg_filter: None,
                },
                shape_type: ShapeType::Diamond,
                fill: "#2ECC71".into(), stroke: None, stroke_width: 0.0,
                border_radius: 0.0, stroke_dasharray: None, gradient: None,
            }),
            Element::Text(TextElement {
                common: CommonProps {
                    id: "text-1".into(),
                    x: 100.0, y: 400.0, width: 300.0, height: 60.0,
                    opacity: 1.0, rotation: 0.0, shadows: vec![],
                    animation: None, blend_mode: None, clip_element_id: None,
                    mask_element_id: None, locked: false, visible: true, svg_filter: None,
                },
                content: "IconStudio".into(),
                fill: "#2C3E50".into(), font_family: "Microsoft YaHei".into(),
                font_size: 48.0, font_weight: "bold".into(), letter_spacing: 2.0,
                stroke: None, stroke_width: 0.0,
                gradient: None,
            }),
            Element::Icon(IconElement {
                common: CommonProps {
                    id: "icon-1".into(),
                    x: 350.0, y: 350.0, width: 80.0, height: 80.0,
                    opacity: 1.0, rotation: 0.0, shadows: vec![],
                    animation: None, blend_mode: None, clip_element_id: None,
                    mask_element_id: None, locked: false, visible: true, svg_filter: None,
                },
                name: "heart".into(),
                fill: "#E74C3C".into(), stroke: None, stroke_width: 0.0,
                gradient: None,
            }),
            Element::Shape(ShapeElement {
                common: CommonProps {
                    id: "shape-6".into(),
                    x: 20.0, y: 20.0, width: 472.0, height: 472.0,
                    opacity: 0.3, rotation: 0.0, shadows: vec![],
                    animation: None, blend_mode: None, clip_element_id: None,
                    mask_element_id: None, locked: false, visible: true, svg_filter: None,
                },
                shape_type: ShapeType::RoundedRect,
                fill: "#ECF0F1".into(), stroke: None, stroke_width: 0.0,
                border_radius: 0.0, stroke_dasharray: None,
                gradient: Some(Gradient {
                    gradient_type: GradientKind::Linear,
                    colors: vec!["#ECF0F1".into(), "#BDC3C7".into()],
                    angle: 135.0, stops: vec![],
                }),
            }),
            Element::Shape(ShapeElement {
                common: CommonProps {
                    id: "shape-7".into(),
                    x: 200.0, y: 200.0, width: 100.0, height: 120.0,
                    opacity: 1.0, rotation: 0.0,
                    shadows: vec![Shadow { color: "#00000050".into(), blur: 10.0, offset_x: 0.0, offset_y: 5.0, inset: false }],
                    animation: None, blend_mode: None, clip_element_id: None,
                    mask_element_id: None, locked: false, visible: true, svg_filter: None,
                },
                shape_type: ShapeType::Shield,
                fill: "#3498DB".into(), stroke: None, stroke_width: 0.0,
                border_radius: 0.0, stroke_dasharray: None, gradient: None,
            }),
            Element::Text(TextElement {
                common: CommonProps {
                    id: "text-2".into(),
                    x: 400.0, y: 450.0, width: 80.0, height: 30.0,
                    opacity: 0.7, rotation: 0.0, shadows: vec![],
                    animation: None, blend_mode: None, clip_element_id: None,
                    mask_element_id: None, locked: false, visible: true, svg_filter: None,
                },
                content: "v1.0".into(),
                fill: "#7F8C8D".into(), font_family: "Microsoft YaHei".into(),
                font_size: 18.0, font_weight: "normal".into(), letter_spacing: 0.0,
                stroke: None, stroke_width: 0.0,
                gradient: None,
            }),
        ],
        exports: ExportConfig::default(),
        templates: HashMap::new(),
        next_element_id: 11,
        version: 0,
        pages: vec![],
        symbols: HashMap::new(),
    }
}

fn bench_build_small(c: &mut Criterion) {
    let project = make_small_project();
    c.bench_function("bench_build_small", |b| {
        b.iter(|| builder::build(black_box(&project)).unwrap())
    });
}

fn bench_build_medium(c: &mut Criterion) {
    let project = make_medium_project();
    c.bench_function("bench_build_medium", |b| {
        b.iter(|| builder::build(black_box(&project)).unwrap())
    });
}

fn bench_render_512(c: &mut Criterion) {
    let project = make_medium_project();
    let svg = builder::build(&project).unwrap();
    c.bench_function("bench_render_512", |b| {
        b.iter(|| renderer::render(black_box(&svg), 512).unwrap())
    });
}

fn bench_render_1024(c: &mut Criterion) {
    let project = make_medium_project();
    let svg = builder::build(&project).unwrap();
    c.bench_function("bench_render_1024", |b| {
        b.iter(|| renderer::render(black_box(&svg), 1024).unwrap())
    });
}

fn bench_undo_redo(c: &mut Criterion) {
    let mut group = c.benchmark_group("undo_redo");
    group.sample_size(20);

    group.bench_function("50_undo_redo_cycles", |b| {
        b.iter(|| {
            let mut project = make_small_project();
            let mut history = CommandHistory::new(50);

            for _ in 0..50 {
                let elem = ShapeElement {
                    common: CommonProps {
                        id: "shape-temp".into(),
                        x: 0.0,
                        y: 0.0,
                        width: 50.0,
                        height: 50.0,
                        opacity: 1.0,
                        rotation: 0.0,
                        shadows: vec![],
                        animation: None,
                        blend_mode: None,
                        clip_element_id: None,
                        mask_element_id: None,
                        locked: false,
                        visible: true,
                        svg_filter: None,
                    },
                    shape_type: ShapeType::Circle,
                    fill: "#FF0000".into(),
                    stroke: None,
                    stroke_width: 0.0,
                    border_radius: 0.0,
                    stroke_dasharray: None,
                    gradient: None,
                };
                let cmd = Box::new(AddElementCommand::new(Element::Shape(elem)));
                history.push_and_execute(cmd, &mut project).unwrap();
            }

            for _ in 0..50 {
                history.undo(&mut project).unwrap();
            }

            for _ in 0..50 {
                history.redo(&mut project).unwrap();
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_build_small,
    bench_build_medium,
    bench_render_512,
    bench_render_1024,
    bench_undo_redo,
    bench_cache_hit,
    bench_cache_miss,
    bench_text_measure,
);

fn bench_cache_hit(c: &mut Criterion) {
    let project = make_medium_project();
    let mut cache = RenderCache::default();
    cache.build(&project).unwrap();

    let mut group = c.benchmark_group("cache");
    group.bench_function("cache_hit_same_version", |b| {
        b.iter(|| {
            cache.build(black_box(&project)).unwrap()
        })
    });
    group.finish();
}

fn bench_cache_miss(c: &mut Criterion) {
    let mut project = make_medium_project();
    let mut cache = RenderCache::default();
    cache.build(&project).unwrap();

    let mut group = c.benchmark_group("cache");
    group.bench_function("cache_miss_version_change", |b| {
        b.iter(|| {
            project.version += 1;
            cache.build(black_box(&project)).unwrap()
        })
    });
    group.finish();
}

fn bench_text_measure(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_measure");
    group.bench_function("measure_english_text", |b| {
        b.iter(|| {
            text_measure::measure_text_width(
                black_box("IconStudio"),
                black_box("Microsoft YaHei"),
                black_box(24.0),
                black_box(400),
            )
        })
    });
    group.bench_function("measure_chinese_text", |b| {
        b.iter(|| {
            text_measure::measure_text_width(
                black_box("图表工作室"),
                black_box("Microsoft YaHei"),
                black_box(32.0),
                black_box(700),
            )
        })
    });
    group.finish();
}

criterion_main!(benches);
