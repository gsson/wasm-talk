precision highp float;

uniform float ONE;

vec2 quickTwoSum(float a, float b) {
    float sum = (a + b) * ONE;
    float err = b - (sum - a) * ONE;
    return vec2(sum, err);
}

vec2 twoSum(float a, float b) {
    float s = (a + b);
    float v = (s * ONE - a) * ONE;
    float err = (a - (s - v) * ONE) * ONE * ONE * ONE + (b - v);
    return vec2(s, err);
}

vec2 twoSub(float a, float b) {
    float s = (a - b);
    float v = (s * ONE - a) * ONE;
    float err = (a - (s - v) * ONE) * ONE * ONE * ONE - (b + v);
    return vec2(s, err);
}

vec2 split(float a) {
    const float SPLIT = 4097.0;
    float t = a * SPLIT;
    float a_hi = t * ONE - (t - a);
    float a_lo = a * ONE - a_hi;
    return vec2(a_hi, a_lo);
}

vec2 twoProd(float a, float b) {
    float prod = a * b;
    vec2 a_fp64 = split(a);
    vec2 b_fp64 = split(b);
    float err = (
        (a_fp64.x * b_fp64.x - prod) +
        a_fp64.x * b_fp64.y + a_fp64.y * b_fp64.x)
    + a_fp64.y * b_fp64.y;
    return vec2(prod, err);
}

vec2 mul_fp64(vec2 a, vec2 b) {
    vec2 prod = twoProd(a.x, b.x);
    // y component is for the error
    prod.y += a.x * b.y;
    prod = quickTwoSum(prod.x, prod.y);
    prod.y += a.y * b.x;
    prod = quickTwoSum(prod.x, prod.y);
    return prod;
}

vec2 sum_fp64(vec2 a, vec2 b) {
    vec2 s, t;
    s = twoSum(a.x, b.x);
    t = twoSum(a.y, b.y);
    s.y += t.x;
    s = quickTwoSum(s.x, s.y);
    s.y += t.y;
    s = quickTwoSum(s.x, s.y);
    return s;
}

vec2 sub_fp64(vec2 a, vec2 b) {
    vec2 s, t;
    s = twoSub(a.x, b.x);
    t = twoSub(a.y, b.y);
    s.y += t.x;
    s = quickTwoSum(s.x, s.y);
    s.y += t.y;
    s = quickTwoSum(s.x, s.y);
    return s;
}

vec2 div_fp64(vec2 a, vec2 b) {
    float xn = 1.0 / b.x;
    vec2 yn = a * xn;
    float diff = (sub_fp64(a, mul_fp64(b, yn))).x;
    vec2 prod = twoProd(xn, diff);
    return sum_fp64(yn, prod);
}


uniform vec2 offset_x;
uniform vec2 offset_y;
uniform vec2 size;
uniform vec2 zoom;

const vec2 TWO = vec2(2.0, 0);
const int MAX_ITER = 128;

int iterate(vec2 px, vec2 py) {
    vec2 x = vec2(0, 0);
    vec2 y = vec2(0, 0);

    for (int i = 0; i < MAX_ITER; i++) {
        vec2 xsquared = mul_fp64(x, x);
        vec2 ysquared = mul_fp64(y, y);
        if (xsquared.x + ysquared.x >= 4.0) {
            return i;
        }

        vec2 x_temp = sum_fp64(sub_fp64(xsquared, ysquared), px);

        y = sum_fp64(mul_fp64(x * 2.0, y), py);
        x = x_temp;
    }
    return MAX_ITER;
}


void main(void) {
    vec2 px = sub_fp64(div_fp64(vec2(gl_FragCoord.x - size.x, 0), zoom), offset_x);
    vec2 py = sub_fp64(div_fp64(vec2(gl_FragCoord.y - size.y, 0), zoom), offset_y);

    int i = iterate(px, py);
    lowp float n = float(i) / 128.0;

    lowp float deg1 = abs(sin(3.14159 / 1.0 * n * n));
    lowp float deg2 = sin(3.14159 / 1.0 * n);
    lowp float deg3 = abs(sin(3.14159 / 0.25 * n * n));

    gl_FragColor = vec4(deg1, deg2, deg3, 1.0);
}