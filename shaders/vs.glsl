#version 410

attribute vec4 position;
attribute vec2 texcoords;

out vec2 coords;

void main() {
    gl_Position = position;
    coords = texcoords;
}
