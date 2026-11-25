import { useEffect, useMemo } from 'react'
import { useFrame } from '@react-three/fiber'
import { Quaternion, ShaderMaterial, Vector3 } from 'three'
import type { Position } from './coordinates.ts'
import { MAX_OPTICAL_BODIES, opticalSpheres } from './opticalSky.ts'
import type { Body } from './sampleUniverse.ts'
import { OPTICAL_ICE } from './bodyPalette.ts'
import type { PlanetReference } from './planetDefinition.ts'

const fragmentShader = `
precision highp float;
uniform int count;
uniform vec3 directions[32], colors[32], accents[32], lights[32];
uniform float icy[32];
uniform vec3 iceColor;
uniform vec4 spheres[32]; // log2(distance), radius/distance, factored surface term, emission
uniform vec3 rightAxis, upAxis, forwardAxis;
uniform float tangent, aspect, pixelAngle;
uniform bool scannerEnvironment, orientationGrid;
varying vec2 screen;
float hash(vec2 p) { return fract(sin(dot(p,vec2(127.1,311.7)))*43758.5453); }
// Triangular octahedral surface cells, fixed in world axes. Quantization is
// shading only: physical silhouettes and rationalized depth stay analytic.
vec3 facet(vec3 n) {
  vec3 p=abs(n)/(abs(n.x)+abs(n.y)+abs(n.z));
  // Keep rounded axis/edge samples inside the final valid triangle. Without
  // this, metre-clearance normals can alternate between invalid pole cells.
  vec2 cell=p.xy*5.99999;
  float upper=step(1.,fract(cell.x)+fract(cell.y))*(1.-step(5.,floor(cell.x)+floor(cell.y)));
  vec2 center=(floor(cell)+vec2((1.+upper)/3.))/6.;
  return normalize(vec3(center,1.-center.x-center.y)*sign(n+vec3(1e-20)));
}
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
  if(scannerEnvironment) {
    // Inertial directions only: neither ship origin nor time enters this field.
    // Dark linear RGB values preserve contrast with cyan scanner markers.
    float navy=0.5+0.5*dot(ray,normalize(vec3(-0.3,0.8,-0.5)));
    color=color*vec3(0.72,0.82,0.95)+mix(vec3(0.012,0.023,0.045),vec3(0.035,0.065,0.105),navy);
    if(orientationGrid) {
      // 15-degree angular graticule, antialiased in screen pixels. Fade near
      // poles and below 5 degrees FOV so telescope lines cannot fill the view.
      vec2 angles=vec2(atan(ray.z,ray.x),asin(clamp(ray.y,-1.,1.)))/0.261799388;
      vec2 edge=abs(fract(angles+0.5)-0.5);
      float width=max(pixelAngle/0.261799388,1e-7);
      float line=1.-smoothstep(width*0.5,width*1.5,min(edge.x,edge.y));
      float fade=smoothstep(0.04366,0.13165,tangent)*(1.-smoothstep(0.9,0.99,abs(ray.y)));
      color+=vec3(0.012,0.023,0.031)*line*fade;
    }
  }
  float nearest=1024.;
  int nearestIndex=-1;
  vec3 nearestNormal=vec3(0.,0.,1.);
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
      // Near a surface, subtracting the two large ray terms loses metre-scale
      // normal components. Reuse the rationalized hit there; retain the
      // perpendicular form for tiny telescope disks where hit rounds to one.
      vec3 surfaceNormal=s.z>=0. && s.y>0.5?ray*hit-dir:ray*along-dir-ray*sqrt(max(disc,0.));
      nearestNormal=surfaceNormal/max(length(surfaceNormal),1e-30);
      nearestIndex=i;
    }
  }
  // Shade only the visible surface, outside the bounded intersection loop.
  // This also avoids cloning expensive facet code in unrolling GPU drivers.
  if(nearestIndex>=0) {
      int i=nearestIndex;
      vec4 s=spheres[i];
      vec3 n=nearestNormal;
      vec3 face=facet(n);
      // Continuous screen-size LOD suppresses subpixel facet noise. No mesh
      // creation, draw changes or discrete topology switches during approach.
      float detail=smoothstep(3.,12.,s.y/max(pixelAngle,1e-8));
      float pigment=0.5+0.5*sin(dot(face,vec3(17.,31.,23.)));
      vec3 albedo=mix(colors[i],accents[i],mix(0.5,pigment,detail));
      // The same latitude ramp as canonical geological ice, never a guessed
      // atmosphere/biome. Published v1 surfaces do not receive this treatment.
      albedo=mix(albedo,iceColor,icy[i]*smoothstep(0.65,0.85,abs(n.y)));
      vec3 shaded=normalize(mix(n,face,detail));
      float illumination=s.w>0.5?0.86+0.14*max(0.,dot(shaded,-ray)):0.085+0.915*max(0.,dot(shaded,lights[i]));
      color=albedo*illumination;
      // Restrained stellar limb emission inside the real disk, no bloom halo
      // or invented atmosphere that could reveal an occulted/distant body.
      if(s.w>0.5) color+=accents[i]*0.08*pow(1.-max(0.,dot(n,-ray)),3.);
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
export function OpticalSky({ bodies, getOrigin, definitionForBody, scannerEnvironment = false, orientationGrid = false }: { bodies: readonly Body[]; getOrigin: () => Position; definitionForBody?: (bodyId: string) => PlanetReference | undefined; scannerEnvironment?: boolean; orientationGrid?: boolean }) {
  const material = useMemo(() => new ShaderMaterial({
    depthTest: false, depthWrite: false, toneMapped: false,
    uniforms: {
      count: { value: 0 }, directions: { value: Array.from({ length: MAX_OPTICAL_BODIES }, () => new Vector3()) },
      colors: { value: Array.from({ length: MAX_OPTICAL_BODIES }, () => new Vector3()) },
      accents: { value: Array.from({ length: MAX_OPTICAL_BODIES }, () => new Vector3()) },
      icy: { value: new Float32Array(MAX_OPTICAL_BODIES) }, iceColor: { value: new Vector3(...OPTICAL_ICE) },
      lights: { value: Array.from({ length: MAX_OPTICAL_BODIES }, () => new Vector3()) },
      spheres: { value: new Float32Array(MAX_OPTICAL_BODIES * 4) },
      rightAxis: { value: new Vector3() }, upAxis: { value: new Vector3() }, forwardAxis: { value: new Vector3() },
      tangent: { value: 1 }, aspect: { value: 1 }, pixelAngle: { value: 1 },
      scannerEnvironment: { value: false }, orientationGrid: { value: false },
    },
    vertexShader: 'varying vec2 screen; void main(){ screen=position.xy; gl_Position=vec4(position.xy,0.,1.); }',
    fragmentShader,
  }), [])
  useEffect(() => () => material.dispose(), [material])
  const orientation = useMemo(() => new Quaternion(), [])
  useFrame(({ camera, size, gl }) => {
    const data = opticalSpheres(bodies, getOrigin(), definitionForBody), uniforms = material.uniforms
    uniforms.scannerEnvironment.value = scannerEnvironment
    uniforms.orientationGrid.value = orientationGrid
    uniforms.count.value = data.length
    data.forEach((sphere, i) => {
      uniforms.directions.value[i].fromArray(sphere.direction)
      uniforms.colors.value[i].fromArray(sphere.color)
      uniforms.accents.value[i].fromArray(sphere.accent)
      uniforms.icy.value[i] = Number(sphere.polarIce)
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
