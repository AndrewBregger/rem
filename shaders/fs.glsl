#version 410

uniform sampler2D atlas;
uniform int bg_pass;

in vec2 Texcoords;
in vec3 Color;
in vec3 Bg;

layout (location = 0, index = 0) out vec4 color0;
layout (location = 0, index = 1) out vec4 color1;

void main() {

    if(bg_pass == 1) {
        color0 = vec4(Bg, 1);
    }
    else {
        vec3 color = texture(atlas, Texcoords).rgb;
        color0 = vec4(Color, color.r);
        color1 = vec4(color, color.r);
    }
}
