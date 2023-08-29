precision mediump float;

attribute vec2 vertPosition;

uniform mat4 mWorld;
uniform mat4 mView;
uniform mat4 mProj;

void main()
{
    gl_Position = mProj * mView * mWorld * vec4(vertPosition, 0.0, 1.0);
    // gl_Position = vec4(vertPosition, 0.0, 1.0);
}