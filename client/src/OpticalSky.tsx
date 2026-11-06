import { useEffect, useMemo } from 'react'
import { useFrame } from '@react-three/fiber'
import { Quaternion, ShaderMaterial, Vector3 } from 'three'
import type { Position } from './coordinates.ts'
import { MAX_OPTICAL_BODIES, opticalSpheres } from './opticalSky.ts'
import type { Body } from './sampleUniverse.ts'

const fragmentShader = `
precision highp float;
uniform int count;
uniform vec3 directions[32], colors[32], lights[32];
uniform vec4 spheres[32]; // log2(distance), radius/distance, factored surface term, emission
uniform vec3 rightAxis, upAxis, forwardAxis;
uniform float tangent, aspect, pixelAngle;
varying vec2 screen;
float hash(vec2 p) { return fract(sin(dot(p,vec2(127.1,311.7)))*43758.5453); }
vec3 stars(vec3 ray) {
  // Sparse deterministic celestial directions, unrelated to the sample catalog.
  vec2 cell=vec2(atan(ray.z,ray.x)/6.2831853+0.5,asin(clamp(ray.y,-1.,1.))/3.14159265+0.5)*vec2(512.,256.);
  float brightness=0.;
  for(int y=-1;y<=1;y++) for(int x=-1;x<=1;x++) {
    vec2 key=floor(cell)+vec2(float(x),float(y));
    key.x=mod(key.x,512.);
    if(key.y<0. || key.y>=256. || hash(key)>0.008) continue;
    vec2 uv=(key+vec2(hash(key+13.),hash(key+37.)))/vec2(512.,256.);
    float lat=(uv.y-0.5)*3.14159265, lon=(uv.x-0.5)*6.2831853;
    vec3 direction=vec3(cos(lat)*cos(lon),sin(lat),cos(lat)*sin(lon));
    float separation=length(cross(ray,direction));
    if(dot(ray,direction)>0.) brightness+=0.48*exp(-pow(separation/max(pixelAngle*0.65,1e-8),2.));
  }
  return vec3(brightness);
}
void main() {
  vec3 ray=normalize(forwardAxis+rightAxis*screen.x*tangent*aspect+upAxis*screen.y*tangent);
  vec3 color=stars(ray);
  float nearest=1024.;
  for(int i=0;i<32;i++) {
    if(i>=count) break;
    vec3 dir=directions[i]; vec4 s=spheres[i];
    float along=dot(ray,dir);
    vec3 perpendicular=ray-dir*along;
    float disc=s.y*s.y-dot(perpendicular,perpendicular);
    float hit=-1.;
    if(s.z<0.) hit=0.;
    else if(along>0. && disc>=0.) hit=s.z/(along+sqrt(disc));
    float logHit=hit==0.?-1024.:log2(max(hit,1e-38))+s.x;
    if(hit>=0. && logHit<nearest) {
      nearest=logHit;
      vec3 surfaceNormal=ray*along-dir-ray*sqrt(max(disc,0.));
      vec3 n=surfaceNormal/max(length(surfaceNormal),1e-30);
      float illumination=s.w>0.5?1.:0.055+0.945*max(0.,dot(n,lights[i]));
      color=colors[i]*illumination;
    }
  }
  // Unresolved bodies: energy-limited optical PSF, never an enlarged solid disk.
  // The foreground ray's physical hit distance masks every farther point.
  for(int i=0;i<32;i++) {
    if(i>=count) break;
    vec4 s=spheres[i];
    if(s.y>=pixelAngle*0.8 || dot(ray,directions[i])<=0. || s.x>=nearest) continue;
    float separation=length(cross(ray,directions[i]));
    float energy=min(1.,pow(s.y/max(pixelAngle,1e-8),2.)* (s.w>0.5?8.:1.));
    color+=colors[i]*energy*exp(-pow(separation/max(pixelAngle*0.65,1e-8),2.));
  }
  gl_FragColor=vec4(color,1.);
}`

/** One reusable quad, bounded analytic sphere tests, no physical depth-buffer
 * range and no mesh tessellation/LOD churn. Call getOrigin from shared ship state.
 */
export function OpticalSky({ bodies, getOrigin }: { bodies: readonly Body[]; getOrigin: () => Position }) {
  const material = useMemo(() => new ShaderMaterial({
    depthTest: false, depthWrite: false, toneMapped: false,
    uniforms: {
      count: { value: 0 }, directions: { value: Array.from({ length: MAX_OPTICAL_BODIES }, () => new Vector3()) },
      colors: { value: Array.from({ length: MAX_OPTICAL_BODIES }, () => new Vector3()) },
      lights: { value: Array.from({ length: MAX_OPTICAL_BODIES }, () => new Vector3()) },
      spheres: { value: new Float32Array(MAX_OPTICAL_BODIES * 4) },
      rightAxis: { value: new Vector3() }, upAxis: { value: new Vector3() }, forwardAxis: { value: new Vector3() },
      tangent: { value: 1 }, aspect: { value: 1 }, pixelAngle: { value: 1 },
    },
    vertexShader: 'varying vec2 screen; void main(){ screen=position.xy; gl_Position=vec4(position.xy,0.,1.); }',
    fragmentShader,
  }), [])
  useEffect(() => () => material.dispose(), [material])
  const orientation = useMemo(() => new Quaternion(), [])
  useFrame(({ camera, size, gl }) => {
    const data = opticalSpheres(bodies, getOrigin()), uniforms = material.uniforms
    uniforms.count.value = data.length
    data.forEach((sphere, i) => {
      uniforms.directions.value[i].fromArray(sphere.direction)
      uniforms.colors.value[i].fromArray(sphere.color)
      uniforms.lights.value[i].fromArray(sphere.lightDirection)
      uniforms.spheres.value.set([Math.log2(sphere.distanceM), sphere.radiusRatio, sphere.surfaceTerm, Number(sphere.emissive)], i * 4)
    })
    camera.getWorldQuaternion(orientation)
    uniforms.rightAxis.value.set(1, 0, 0).applyQuaternion(orientation)
    uniforms.upAxis.value.set(0, 1, 0).applyQuaternion(orientation)
    uniforms.forwardAxis.value.set(0, 0, -1).applyQuaternion(orientation)
    const fov = 'fov' in camera ? Number(camera.fov) : 75
    uniforms.tangent.value = Math.tan(fov * Math.PI / 360)
    uniforms.aspect.value = size.width / Math.max(1, size.height)
    uniforms.pixelAngle.value = 2 * uniforms.tangent.value / (Math.max(1, size.height) * gl.getPixelRatio())
  }, -1)
  return <mesh frustumCulled={false} material={material} renderOrder={-1000}>
    <planeGeometry args={[2, 2]}/>
  </mesh>
}
