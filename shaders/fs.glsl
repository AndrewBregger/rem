#version 410

uniform sampler2D atlas;
uniform int bg_pass;

in vec2 Texcoords;
flat in vec3 Fg;
flat in vec4 Bg;

layout (location = 0, index = 0) out vec4 color;
layout (location = 0, index = 1) out vec4 alpha_mask;

void main() {

    if(bg_pass == 1) {
        if(Bg.a == 0.0)
            discard;

        color = Bg;
        alpha_mask = vec4(1);
    }
    else {
        vec3 character = texture(atlas, Texcoords).rgb;
        color = vec4(Fg, character.r);
        alpha_mask = vec4(character, character.r);
    }
}
