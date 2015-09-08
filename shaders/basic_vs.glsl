#version 150

in vec3 position;
in vec2 texcoord;

out vec2 Texcoord;

uniform mat4 transform;

void main() {
  Texcoord = texcoord;
  gl_Position = transform * vec4(position, 1.0);
}
