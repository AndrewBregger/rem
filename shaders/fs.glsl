#version 410

uniform sampler2D atlas;

in vec2 Texcoords;
in vec3 Color;

layout (location = 0, index = 0) out vec4 color0;
layout (location = 0, index = 1) out vec4 color1;

void main() {
    vec3 color = texture(atlas, Texcoords).rgb;
    color0 = vec4(Color, color.r);
    color1 = vec4(color, color.r);
}
