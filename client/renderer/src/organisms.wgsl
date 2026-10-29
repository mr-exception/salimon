struct View {matrix:mat4x4<f32>,light:vec4<f32>,color:vec4<f32>};
@group(0) @binding(0) var<uniform> view:View;
struct V {@builtin(position) position:vec4<f32>,@location(0) point:vec3<f32>,@location(1) color:vec3<f32>};
@vertex fn vs_main(@location(0) p:vec3<f32>,@location(1) start:vec4<f32>,@location(2) end:vec4<f32>,@location(3) color:vec4<f32>)->V {
 let up=normalize(end.xyz-start.xyz);let reference=select(vec3(0.,1.,0.),vec3(1.,0.,0.),abs(up.y)>0.9);let right=normalize(cross(up,reference));let forward=cross(right,up);
 let radius=mix(start.w,end.w,p.y);let point=mix(start.xyz,end.xyz,p.y)+(right*p.x+forward*p.z)*radius;
 var v:V;v.position=view.matrix*vec4(point,1.);v.point=point;v.color=color.rgb;return v;
}
@fragment fn fs_main(v:V)->@location(0) vec4<f32> {let normal=normalize(cross(dpdx(v.point),dpdy(v.point)));let diffuse=abs(dot(normal,view.light.xyz));return vec4(v.color*(vec3(0.18)+view.color.rgb*0.82*diffuse),1.);}
