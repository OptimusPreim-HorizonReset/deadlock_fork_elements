#version 450

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 fragment_color;

void main() {
    float radial_alpha = clamp(1.0 - dot(uv, uv), 0.0, 1.0);
    float density_alpha = clamp(0.65 * radial_alpha, 0.0, 0.9);
    vec3 gas_color = mix(vec3(0.25, 0.72, 1.0), vec3(0.95, 0.42, 0.22), radial_alpha);
    fragment_color = vec4(gas_color, density_alpha);
}