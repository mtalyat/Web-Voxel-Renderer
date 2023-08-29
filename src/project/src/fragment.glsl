precision mediump float;

const int MAX_STEPS = 255;
const float EPSILON = 0.0001;

uniform float fov; // field of view
uniform vec3 eye; // camera pos
uniform vec3 up; // up direction (probably (0, 1, 0))
uniform vec2 resolution; // size of canvas
uniform float time; // time in seconds
uniform vec3 background_color; // background color as R G B
uniform float near; // min render distance
uniform float far; // max render distance

const int OBJ_SHAPE_CUBE = 0;
const int OBJ_SHAPE_SPHERE = 1;

struct Object {
    vec3 position;
    vec3 size;
    vec4 color;
    int shape;
};

struct Result {
    float distance;
    vec4 color;
};

const int OBJ_COUNT = 2;

uniform Object objects[OBJ_COUNT];

// distance from p to sphere at origin with radius r
float sphereSDF(vec3 p, vec3 c, float r)
{
    return distance(p, c) - r;
}

// distance from p to cube at origin with size s
float cubeSDF(vec3 p, vec3 c, vec3 s)
{
    vec3 o = abs(p - c) - s;
    float ud = length(max(o,0.));
    float n = max(max(min(o.x,0.),min(o.y,0.)),min(o.z,0.));
    return ud + n;
}

// rotate on y
mat4 rotateY(float theta)
{
    float c = cos(theta);
    float s = sin(theta);

    return mat4(
        vec4(c,0,s,0),
        vec4(0,1,0,0),
        vec4(-s,0,c,0),
        vec4(0,0,0,1)
    );
}

// distance to any object in scene
Result sceneSDF(vec3 p)
{
    Object obj;
    int shape;
    float dist = 0.;
    float resultDist = 0.;
    vec4 resultColor;
    for(int i = 0; i < OBJ_COUNT; i++)
    {
        obj = objects[i];
        shape = obj.shape;
        
        if (shape == OBJ_SHAPE_CUBE)
        {
            dist = cubeSDF(p, obj.position, obj.size);
        } else if (shape == OBJ_SHAPE_SPHERE)
        {
            dist = sphereSDF(p, obj.position, obj.size.x);
        } else
        {
            // invalid shape type
            dist = resultDist;
        }

        if(i == 0)
        {
            // always set to the first one
            resultDist = dist;
            resultColor = obj.color;
        } else
        {
            if (dist < resultDist)
            {
                resultDist = dist;
                resultColor = obj.color;
            }
        }
    }

    return Result(resultDist, resultColor);
}

// shortest distance to surface
// return shortest distance from i to object
Result sds(vec3 eye, vec3 dir, float start, float end)
{
    float depth = start;
    for (int i = 0; i < MAX_STEPS; i++)
    {
        Result result = sceneSDF(eye + depth * dir);
        float dist = result.distance;
        if (dist < EPSILON)
        {
            // hit
            return Result(depth, result.color);
        }
        depth += dist;
        if (depth >= end)
        {
            // too far
            return Result(end, vec4(0.0,0.0,0.0,0.0));
        }
    }

    // too many steps
    return Result(end, vec4(0.0,0.0,0.0,0.0));
}

// get direction of ray for pixel
vec3 calcRayDir(float fov, vec2 size, vec2 fragCoord)
{
    vec2 xy = fragCoord - size / 2.0;
    float z = size.y / 2.0 / tan(radians(fov) / 2.0);
    return normalize(vec3(xy, -z));
}

// get estimate of normal
vec3 calcNormal(vec3 p)
{
    return normalize(vec3(
        sceneSDF(vec3(p.x + EPSILON, p.y, p.z)).distance - sceneSDF(vec3(p.x - EPSILON, p.y, p.z)).distance,
        sceneSDF(vec3(p.x, p.y + EPSILON, p.z)).distance - sceneSDF(vec3(p.x, p.y - EPSILON, p.z)).distance,
        sceneSDF(vec3(p.x, p.y, p.z + EPSILON)).distance - sceneSDF(vec3(p.x, p.y, p.z - EPSILON)).distance
    ));
}

// lighting contribution
vec3 calcLighting(vec3 diffuse, vec3 specular, float alpha, vec3 p, vec3 eye, vec3 lightPos, vec3 lightIntensity)
{
    vec3 N = calcNormal(p);
    vec3 L = normalize(lightPos - p);
    vec3 V = normalize(eye - p);
    vec3 R = normalize(reflect(-L, N));

    float dotLN = dot(L, N);
    float dotRV = dot(R, V);

    if (dotLN < 0.)
    {
        // light not visible from this point
        return vec3(0.,0.,0.);
    }

    if (dotRV < 0.)
    {
        // light reflecting away from eye
        return lightIntensity * (diffuse * dotLN);
    }

    return lightIntensity * (diffuse * dotLN + specular * pow(dotRV, alpha));
}

// phong illumination
vec3 calcIllumination(vec3 ambient, vec3 diffuse, vec3 specular, float alpha, vec3 p, vec3 eye)
{
    const vec3 ambientLight = .5 * vec3(1.,1.,1.);
    vec3 color = ambientLight * ambient;

    vec3 light1Pos = vec3(4. * sin(time), 2., 4. * cos(time));
    vec3 light1Intensity = vec3(.4, .4, .4);

    color += calcLighting(diffuse, specular, alpha, p, eye, light1Pos, light1Intensity);

    vec3 light2Pos = vec3(2. * sin(.37 * time), 2. * cos(.37 * time), 2.);
    vec3 light2Intensity = vec3(.4, .4, .4);

    color += calcLighting(diffuse, specular, alpha, p, eye, light2Pos, light2Intensity);

    return color;
}

// change view matrix to simulate moving the camera
mat4 calcView(vec3 eye, vec3 c, vec3 up)
{
    vec3 f = normalize(c - eye);
    vec3 s = normalize(cross(f, up));
    vec3 u = cross(s, f);
    return mat4(
        vec4(s, 0.),
        vec4(u,0.),
        vec4(-f,0.),
        vec4(0.,0.,0.,1.)
    );
}

// gl_FragCoord, gl_FragColor

void main() {
    vec3 viewDir = calcRayDir(fov, resolution, gl_FragCoord.xy);

    mat4 viewToWorld = calcView(eye, vec3(0.,0.,0.), up);
    vec3 worldDir = (viewToWorld * vec4(viewDir, 0.)).xyz;

    Result result = sds(eye, worldDir, near, far);

    if(result.distance > far - EPSILON)
    {
        // hit nothing
        gl_FragColor = vec4(background_color, 1.0);
        return;
    }

    vec3 p = eye + result.distance * worldDir;

    vec3 ambient = vec3(.2, .2, .2);
    vec3 diffuse = vec3(.2, .2, .2);
    vec3 specular = vec3(1., 1., 1.);
    float shiny = 10.;

    // lighting color
    vec4 color = vec4(calcIllumination(ambient, diffuse, specular, shiny, p, eye), 1.0);

    // mix with object color
    color *= result.color;

    // final color
    gl_FragColor = color;
}