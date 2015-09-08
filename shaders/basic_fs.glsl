#version 150

in vec2 Texcoord;

out vec4 out_color;

uniform sampler2D diffuse;

void main() {
   out_color = texture(diffuse, Texcoord);
}
