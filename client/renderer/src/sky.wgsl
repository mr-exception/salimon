// Analytically authored galactic band: decorative radiance, never catalog truth.
struct Sky { right:vec4<f32>, up:vec4<f32>, forward:vec4<f32>, pole:vec4<f32>, size:vec4<f32>, reserved:vec4<f32> };
@group(0) @binding(0) var<uniform> sky:Sky;
struct V { @builtin(position) position:vec4<f32>, @location(0) uv:vec2<f32>, @location(1) color:vec4<f32> };
@vertex fn vs_sky(@builtin(vertex_index) i:u32)->V { let p=array<vec2<f32>,3>(vec2(-1.,-1.),vec2(3.,-1.),vec2(-1.,3.)); var o:V; o.position=vec4(p[i],0.,1.); o.uv=p[i]; o.color=vec4(1.); return o; }
@fragment fn fs_sky(v:V)->@location(0) vec4<f32> {
 let d=normalize(sky.forward.xyz+v.uv.x*sky.right.xyz*sky.right.w+v.uv.y*sky.up.xyz*sky.up.w);
 let b=dot(d,sky.pole.xyz); let a=atan2(d.z,d.x);
 let broad=exp(-b*b*55.); let dust=exp(-pow(b+0.028*sin(a*7.),2.)*2500.);
 let knots=0.65+0.2*sin(a*13.+b*37.)+0.1*sin(a*31.-b*91.);
 let color=vec3(0.0003,0.0005,0.001)+broad*knots*(1.-0.8*dust)*vec3(0.018,0.022,0.035);
 return vec4(color*sky.pole.w,1.);
}
@vertex fn vs_star(@builtin(vertex_index) i:u32,@location(0) direction_radius:vec4<f32>,@location(1) color:vec4<f32>)->V {
 let q=array<vec2<f32>,6>(vec2(-1.,-1.),vec2(1.,-1.),vec2(-1.,1.),vec2(-1.,1.),vec2(1.,-1.),vec2(1.,1.));
 let z=dot(direction_radius.xyz,sky.forward.xyz); var o:V; o.uv=q[i]; o.color=color;
 let p=vec2(dot(direction_radius.xyz,sky.right.xyz)/sky.right.w,dot(direction_radius.xyz,sky.up.xyz)/sky.up.w);
 let radius=max(tan(direction_radius.w)/sky.up.w,1.4/sky.size.y*z);
 o.position=vec4(p+q[i]*vec2(radius*sky.size.y/sky.size.x,radius),0.,z); return o;
}
@fragment fn fs_star(v:V)->@location(0) vec4<f32> { let r=length(v.uv); if r>1. {discard;} return vec4(v.color.rgb*v.color.a,1.-smoothstep(0.7,1.,r)); }
