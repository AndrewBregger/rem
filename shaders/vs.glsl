#version 410

attribute vec4 position;
attribute vec2 texcoords;

uniform mat4 per;

out vec2 coords;

void main() {
    gl_Position = per * position;
    coords = texcoords;
}
