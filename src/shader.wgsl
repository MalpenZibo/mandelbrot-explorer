struct Viewport {
    half_viewport_x: f32,
    half_viewport_y: f32,
    ratio: f32,
};

struct Coordinates {
    real: f32,
    imag: f32,
    zoom: f32,
};

struct Iterations {
    value: i32
};

struct ColorParams {
    hue: f32,
    saturation: f32,
    lightness: f32,
    hue_linked: i32,
    saturation_linked: i32,
    lightness_linked: i32,
};

@group(0) @binding(0)
var<uniform> viewport: Viewport;

@group(1) @binding(0)
var<uniform> coordinates: Coordinates;

@group(2) @binding(0)
var<uniform> max_iterations: Iterations;

@group(3) @binding(0)
var<uniform> color_params: ColorParams;

@vertex
fn vs_main(@builtin(vertex_index) vertex_idx: u32) -> @builtin(position) vec4<f32> {
    var pos = vec2<f32>(0.0, 0.0);
    switch vertex_idx {
        case 0u: {
            pos.x = 1.0;
            pos.y = 1.0;
        }
        case 1u, 5u: {
            pos.x = -1.0;
            pos.y = 1.0;
        }
        case 2u, 3u: {
            pos.x = 1.0;
            pos.y = -1.0;
        }
        case 4u: {
            pos.x = -1.0;
            pos.y = -1.0;
        }
        default: {}
    }

    return vec4<f32>(pos, 0.0, 1.0);
}

fn transform_position(in: vec2<f32>) -> vec2<f32> {
    var x = (in.x - viewport.half_viewport_x) / viewport.half_viewport_x * viewport.ratio;
    var y = (in.y - viewport.half_viewport_y) / viewport.half_viewport_y * -1.0;

    let center_x = coordinates.real - 0.4;

    x = x * coordinates.zoom + center_x;
    y = y * coordinates.zoom - coordinates.imag;

    return vec2(x, y);
}

fn get_iterations(c: vec2<f32>) -> i32 {
    var real = c.x;
    var imag = c.y;

    var iterations = 0;
    var const_real = real;
    var const_imag = imag;

    while iterations < max_iterations.value {
        var tmp_real = real;
        real = (real * real - imag * imag) + const_real;
        imag = (2.0 * tmp_real * imag) + const_imag;

        var dist = real * real + imag * imag;

        if dist > 4.0 {
            break;
        }

        iterations += 1;
    }

    return iterations;
}

fn hsl_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(hsv.xxx + K.xyz) * 6.0 - K.www);

    let clamped = clamp(p - K.xxx, vec3(0.0), vec3(1.0));

    return hsv.z * mix(K.xxx, clamped, hsv.y);
}

fn get_color(iterations: i32) -> vec4<f32> {
    var iterations_color = f32(iterations) / f32(max_iterations.value);

    var hue = select(color_params.hue, iterations_color + color_params.hue - 1.0, color_params.hue_linked > 0);
    var saturation = select(color_params.saturation, color_params.saturation * iterations_color, color_params.saturation_linked > 0);
    var lightness = select(0.0, select(color_params.lightness, color_params.lightness * iterations_color, color_params.lightness_linked > 0), iterations_color < 1.0);

    return vec4(hsl_to_rgb(vec3(hue, saturation, lightness)), 1.0);
}

@fragment
fn fs_main(@builtin(position) in: vec4<f32>) -> @location(0) vec4<f32> {
    let position = transform_position(in.xy);

    let iterations = get_iterations(position);

    return get_color(iterations);
}