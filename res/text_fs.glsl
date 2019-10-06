#version 410

uniform sampler2D atlas;

flat in vec3 Color;
in vec2 TexCoords;

layout (location = 0, index = 0) out vec4 color;
layout (location = 0, index = 1) out vec4 character;

void main() {
    vec3 value = texture(atlas, TexCoords).rgb;
    character = vec4(value, value.r);

    // float value = texture(atlas, TexCoords).a;
    // character = vec4(value);

    color = vec4(Color, value.r);
}