#version 330 core

// inflused by: Jwilm's Alacritty font shader.
layout (location = 0) in vec2 cell;
layout (location = 1) in vec3 gcolor;
layout (location = 2) in vec4 glyph;
layout (location = 3) in vec4 uv;

uniform mat4 projection;
uniform vec2 cell_size;

out vec2 Texcoords;
out vec3 Color;

 void main() {
    vec2 cell_position = cell_size * cell;
    vec2 glyphSize = glyph.xy;

    vec2 glyphOffset;

    glyphOffset.x = glyph.z;
    glyphOffset.y = cell_size.y - glyph.w;

    vec2 position;
    position.x = (gl_VertexID == 0 || gl_VertexID == 1) ? 1.0 : 0.0;
    position.y = (gl_VertexID == 0 || gl_VertexID == 3) ? 0.0 : 1.0;

    vec2 final_position = cell_position + glyphSize * position + glyphOffset;

    gl_Position = projection * vec4(final_position, 0, 1.0);

    Texcoords = uv.xy + position * uv.zw;
    Color = gcolor;
}
