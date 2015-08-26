#version 150
out vec4 out_color;
in vec2 Texcoord;
uniform sampler2D tex;
void main() {
   out_color = texture(tex, Texcoord);
}
