#version 410

uniform sampler2D text;

in vec2 coords;
out vec4 color;

void main() {
    color = texture(text, coords); // + vec4(0.4, 0.5, 0.4, 1.0);
}
