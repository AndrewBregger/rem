#version 330 core

layout (location = 0) in vec2 cell;
layout (location = 1) in vec3 color;

uniform mat4 projection;
uniform vec2 cell_size;

out vec3 Color;

void main() {
    vec2 cell_position = cell_size * cell;

    vec2 position;
    position.x = (gl_VertexID == 0 || gl_VertexID == 1) ? 1.0 : 0.0;
    position.y = (gl_VertexID == 0 || gl_VertexID == 3) ? 0.0 : 1.0;
    
    vec2 final_position = cell_position + cell_size * position;

    gl_Position = projection * vec4(final_position, 0.0, 1.0);
    
    Color = color;
}
