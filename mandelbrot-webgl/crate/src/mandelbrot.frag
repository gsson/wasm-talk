precision highp float;

uniform vec2 offset;
uniform vec2 size;
uniform float zoom;

int iterate(vec2 p) {
    vec2 z = vec2(0, 0);
    for (int i = 0; i < 150; i++) {
        vec2 zsquared = z * z;
        if(zsquared.x + zsquared.y >= 4.0) {
            return i;
        }

        z = vec2(
            (zsquared.x) - (zsquared.y) + p.x,
            (2.0 * z.x * z.y) + p.y
        );
    }
    return 150;
}


void main(void) {
    vec2 t = (gl_FragCoord.xy - size) / zoom - offset;
    int i = iterate(t);
    float intensity = float(i * i) / (150.0 * 150.0);

    gl_FragColor = vec4(intensity, intensity, intensity, 1.0);
}