struct View {matrix:mat4x4<f32>,light:vec4<f32>,color:vec4<f32>};
@group(0) @binding(0) var<uniform> view:View;
struct V {@builtin(position) position:vec4<f32>,@location(0) point:vec3<f32>,@location(1) color:vec3<f32>,@location(2) material:vec4<f32>,@location(3) coating:vec3<f32>};
@vertex fn vs_main(@location(0) p:vec3<f32>,@location(1) start:vec4<f32>,@location(2) end:vec4<f32>,@location(3) color:vec4<f32>,@location(4) material:vec4<f32>,@location(5) flex:vec4<f32>,@location(6) coating:vec4<f32>)->V {
 let up=normalize(end.xyz-start.xyz);let reference=select(vec3(0.,1.,0.),vec3(1.,0.,0.),abs(up.y)>0.9);let right=normalize(cross(up,reference));let forward=cross(right,up);
 // Root cap is fixed. Shrink by displacement to stay inside the canonical envelope.
 let displacement=flex.xyz*p.y*p.y;
 let radius=max(0.,mix(start.w,end.w,p.y)-length(displacement));
 let point=mix(start.xyz,end.xyz,p.y)+(right*p.x+forward*p.z)*radius+displacement;
 var v:V;v.position=view.matrix*vec4(point,1.);v.point=point;v.color=color.rgb;v.material=material;v.coating=coating.xyz;return v;
}
@fragment fn fs_main(v:V)->@location(0) vec4<f32> {
 let normal=normalize(cross(dpdx(v.point),dpdy(v.point)));
 let ndl=abs(dot(normal,view.light.xyz));let eye=normalize(-v.point);
 let rim=1.-abs(dot(normal,eye));
 let wet=v.coating.x;let frost=v.coating.y;let dust=v.coating.z;
 var pigment=v.color*(1.-0.25*wet);
 pigment=mix(pigment,vec3(0.65,0.72,0.76),frost*0.65);
 pigment=mix(pigment,vec3(0.35,0.29,0.21),dust*0.45);
 let rough=clamp(v.material.x-0.3*wet+0.25*dust,0.12,1.);
 let spec=pow(max(0.,dot(reflect(-view.light.xyz,normal),eye)),mix(64.,4.,rough))*(1.-rough)*0.22;
 let sheen=vec3(0.08,0.12,0.18)*v.material.z*rim*rim;
 let transmission=v.material.y*0.14*rim;
 // Emission is local to selected organs, visible without bloom and cannot light the scene.
 return vec4(pigment*(vec3(0.035)+view.color.rgb*(0.82*ndl+transmission+spec))+sheen*view.color.rgb+pigment*v.material.w*0.45,1.);
}
