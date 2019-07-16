#version 410

uniform sampler2D text;
// uniform vec4 background;

in vec2 coords;

layout (location = 0, index = 0) out vec4 color0;
layout (location = 0, index = 1) out vec4 color1;

void main() {
    vec3 color = texture(text, coords).rgb;
    color0 = vec4(0.3, 0.4, 0.3, 1.0);
    color1 = vec4(color, color.r);
}
