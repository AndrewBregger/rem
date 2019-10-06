#version 330 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec2 texCoords;
layout (location = 2) in vec3 color;

// uniform vec4 tColor;

uniform mat4 projection;

flat out vec3 Color;
out vec2 TexCoords;

void main() {
    gl_Position = projection * vec4(position, 1.0);
    TexCoords = texCoords;
    Color = color;
}