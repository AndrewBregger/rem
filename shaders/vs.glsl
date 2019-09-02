#version 330 core

// inflused by: Jwilm's Alacritty font shader.
layout (location = 0) in vec2 cell;
layout (location = 1) in vec4 glyph;
layout (location = 2) in vec4 uv;
layout (location = 3) in vec3 tc; // text color
layout (location = 4) in vec4 bg; // background color

uniform mat4 projection;
uniform vec2 cell_size;
uniform int bg_pass;

out vec2 Texcoords;
flat out vec3 Fg;
flat out vec4 Bg;

 void main() {
    vec2 cell_position = cell_size * cell;
    vec2 glyphSize = glyph.xy;

    vec2 glyphOffset;

    glyphOffset.x = glyph.z;
    glyphOffset.y = cell_size.y - glyph.w;

    vec2 position;
    position.x = (gl_VertexID == 0 || gl_VertexID == 1) ? 1.0 : 0.0;
    position.y = (gl_VertexID == 0 || gl_VertexID == 3) ? 0.0 : 1.0;


    if(bg_pass == 1) {
        vec2 final_position = cell_position + cell_size * position;
        gl_Position = projection * vec4(final_position, 0, 1.0);
        Texcoords = vec2(0, 0);
    }
    else {
        vec final_position = cell_position + glyphSize * position + glyphOffset;
        gl_Position = projection * vec4(final_position, 0, 1.0);
        Texcoords = uv.xy + position * uv.zw;
    }

    Bg = bg;
    Fg = tc;
}
