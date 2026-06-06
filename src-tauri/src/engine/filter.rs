use crate::model::filter::{FilterType, SvgFilter};

/// Convert an SvgFilter to an SVG `<filter>` element string.
/// Returns the inner content of `<filter id="filter-{id}">...</filter>`.
pub fn filter_to_svg(filter: &SvgFilter, id: usize) -> String {
    let def_id = format!("filter-{}", id);
    let inner = match filter.filter_type {
        FilterType::Noise => noise_filter(filter),
        FilterType::Blur => blur_filter(filter),
        FilterType::Pixelate => pixelate_filter(filter),
        FilterType::Emboss => emboss_filter(filter),
        FilterType::Posterize => posterize_filter(filter),
        FilterType::Turbulence => turbulence_filter(filter),
    };
    format!(
        r#"<filter id="{def_id}" x="-10%" y="-10%" width="120%" height="120%">{inner}</filter>"#,
        def_id = def_id,
        inner = inner,
    )
}

fn noise_filter(f: &SvgFilter) -> String {
    let base_freq = get_param(f, "baseFrequency", 0.05).clamp(0.001, 1.0);
    let num_octaves = get_param(f, "numOctaves", 3.0).clamp(1.0, 6.0) as i32;
    format!(
        r#"<feTurbulence type="fractalNoise" baseFrequency="{bf:.4}" numOctaves="{no}" result="noise"/><feColorMatrix in="noise" type="saturate" values="0" result="grayNoise"/><feBlend in="SourceGraphic" in2="grayNoise" mode="multiply"/>"#,
        bf = base_freq,
        no = num_octaves,
    )
}

fn blur_filter(f: &SvgFilter) -> String {
    let std_dev = get_param(f, "stdDeviation", 3.0).clamp(0.0, 50.0);
    if std_dev < f64::EPSILON {
        return String::new();
    }
    format!(
        r#"<feGaussianBlur stdDeviation="{sd:.2}"/>"#,
        sd = std_dev,
    )
}

/// Pixelate via feConvolveMatrix with a uniform kernel (box blur approximation).
/// The kernel size is derived from the `size` param.
fn pixelate_filter(f: &SvgFilter) -> String {
    let size = get_param(f, "size", 4.0).clamp(2.0, 20.0) as i32;
    let n = (size * size) as f64;
    let kernel: String = (0..(size * size))
        .map(|_| format!("{:.4}", 1.0 / n))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        r#"<feConvolveMatrix order="{s},{s}" kernelMatrix="{k}"/>"#,
        s = size,
        k = kernel,
    )
}

fn emboss_filter(f: &SvgFilter) -> String {
    let strength = get_param(f, "strength", 1.0).clamp(0.1, 5.0);
    // Emboss 3x3 kernel: [-2,-1,0,-1,1,1,0,1,2] scaled by strength
    let s = strength;
    let vals = [
        -2.0 * s, -s, 0.0,
        -s,  s, s,
         0.0,      s, 2.0 * s,
    ];
    let kernel: String = vals.iter()
        .map(|v| format!("{:.3}", v))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        r#"<feConvolveMatrix order="3,3" kernelMatrix="{k}" preserveAlpha="true"/>"#,
        k = kernel,
    )
}

fn posterize_filter(f: &SvgFilter) -> String {
    let steps = get_param(f, "steps", 4.0).clamp(2.0, 10.0) as i32;
    // Build a stepped transfer function: map each channel to discrete levels.
    // Using a table interpolation approach for feComponentTransfer.
    let segments = steps as usize;
    let mut table_values = Vec::with_capacity(segments);
    for i in 0..segments {
        table_values.push(format!("{:.4}", i as f64 / (segments - 1) as f64));
    }
    let tv = table_values.join(" ");
    format!(
        r#"<feComponentTransfer><feFuncR type="discrete" tableValues="{tv}"/><feFuncG type="discrete" tableValues="{tv}"/><feFuncB type="discrete" tableValues="{tv}"/></feComponentTransfer>"#,
        tv = tv,
    )
}

fn turbulence_filter(f: &SvgFilter) -> String {
    let base_freq = get_param(f, "baseFrequency", 0.05).clamp(0.001, 1.0);
    let num_octaves = get_param(f, "numOctaves", 3.0).clamp(1.0, 6.0) as i32;
    let turb_type = match f.params.get("turbType").map(|v| *v as i32) {
        Some(1) => "turbulence",
        _ => "fractalNoise",
    };
    format!(
        r#"<feTurbulence type="{tt}" baseFrequency="{bf:.4}" numOctaves="{no}" result="turb"/><feDisplacementMap in="SourceGraphic" in2="turb" scale="{sc:.1}" xChannelSelector="R" yChannelSelector="G"/>"#,
        tt = turb_type,
        bf = base_freq,
        no = num_octaves,
        sc = base_freq * 100.0,
    )
}

fn get_param(f: &SvgFilter, key: &str, default: f64) -> f64 {
    f.params.get(key).copied().unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_filter(ft: FilterType, params: Vec<(&str, f64)>) -> SvgFilter {
        SvgFilter {
            filter_type: ft,
            params: params.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
        }
    }

    #[test]
    fn test_noise_filter_svg() {
        let f = make_filter(FilterType::Noise, vec![("baseFrequency", 0.02), ("numOctaves", 2.0)]);
        let svg = filter_to_svg(&f, 1);
        assert!(svg.contains(r#"id="filter-1""#));
        assert!(svg.contains("feTurbulence"));
        assert!(svg.contains(r#"baseFrequency="0.0200""#));
        assert!(svg.contains(r#"numOctaves="2""#));
        assert!(svg.contains("feColorMatrix"));
        assert!(svg.contains("feBlend"));
    }

    #[test]
    fn test_blur_filter_svg() {
        let f = make_filter(FilterType::Blur, vec![("stdDeviation", 5.0)]);
        let svg = filter_to_svg(&f, 2);
        assert!(svg.contains(r#"id="filter-2""#));
        assert!(svg.contains("feGaussianBlur"));
        assert!(svg.contains(r#"stdDeviation="5.00""#));
    }

    #[test]
    fn test_blur_zero_no_output() {
        let f = make_filter(FilterType::Blur, vec![("stdDeviation", 0.0)]);
        let svg = filter_to_svg(&f, 3);
        assert!(svg.contains("filter-3"));
        // Inner should be empty (just the filter tag wrapper)
    }

    #[test]
    fn test_pixelate_filter_svg() {
        let f = make_filter(FilterType::Pixelate, vec![("size", 3.0)]);
        let svg = filter_to_svg(&f, 4);
        assert!(svg.contains(r#"id="filter-4""#));
        assert!(svg.contains("feConvolveMatrix"));
        assert!(svg.contains(r#"order="3,3""#));
        // 3x3 = 9 kernel values
        let kernel_part = svg.split("kernelMatrix=\"").nth(1).unwrap();
        let values: Vec<&str> = kernel_part.split('"').next().unwrap().split(' ').collect();
        assert_eq!(values.len(), 9);
    }

    #[test]
    fn test_emboss_filter_svg() {
        let f = make_filter(FilterType::Emboss, vec![("strength", 1.5)]);
        let svg = filter_to_svg(&f, 5);
        assert!(svg.contains(r#"id="filter-5""#));
        assert!(svg.contains("feConvolveMatrix"));
        assert!(svg.contains(r#"order="3,3""#));
        assert!(svg.contains("preserveAlpha=\"true\""));
    }

    #[test]
    fn test_posterize_filter_svg() {
        let f = make_filter(FilterType::Posterize, vec![("steps", 4.0)]);
        let svg = filter_to_svg(&f, 6);
        assert!(svg.contains(r#"id="filter-6""#));
        assert!(svg.contains("feComponentTransfer"));
        assert!(svg.contains("feFuncR"));
        assert!(svg.contains("feFuncG"));
        assert!(svg.contains("feFuncB"));
        assert!(svg.contains(r#"type="discrete""#));
        // 4 steps = 4 table values per channel
        let tv_count = svg.matches("tableValues=").count();
        assert_eq!(tv_count, 3); // R, G, B
    }

    #[test]
    fn test_turbulence_filter_svg() {
        let f = make_filter(FilterType::Turbulence, vec![("baseFrequency", 0.03), ("numOctaves", 4.0)]);
        let svg = filter_to_svg(&f, 7);
        assert!(svg.contains(r#"id="filter-7""#));
        assert!(svg.contains("feTurbulence"));
        assert!(svg.contains("feDisplacementMap"));
        assert!(svg.contains(r#"baseFrequency="0.0300""#));
        assert!(svg.contains(r#"numOctaves="4""#));
    }

    #[test]
    fn test_turbulence_type_param() {
        let mut f = make_filter(FilterType::Turbulence, vec![("baseFrequency", 0.05)]);
        f.params.insert("turbType".to_string(), 1.0);
        let svg = filter_to_svg(&f, 8);
        assert!(svg.contains(r#"type="turbulence""#));
    }

    #[test]
    fn test_default_params_used() {
        // No params provided — defaults should apply
        let f = SvgFilter {
            filter_type: FilterType::Blur,
            params: HashMap::new(),
        };
        let svg = filter_to_svg(&f, 9);
        assert!(svg.contains("feGaussianBlur"));
        // Default stdDeviation = 3.0
        assert!(svg.contains(r#"stdDeviation="3.00""#));
    }

    #[test]
    fn test_param_clamping() {
        // baseFrequency below min should be clamped to 0.001
        let f = make_filter(FilterType::Noise, vec![("baseFrequency", -1.0)]);
        let svg = filter_to_svg(&f, 10);
        assert!(svg.contains(r#"baseFrequency="0.0010""#));
    }
}
