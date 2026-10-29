// Planet-radius units preserve local f64 subtraction and avoid galaxy-scale GPU positions.
struct Atmosphere { right:vec4<f32>, up:vec4<f32>, forward:vec4<f32>, observer:vec4<f32>, beta_height:vec4<f32>, sun:vec4<f32>, color:vec4<f32>, size:vec4<f32> };
@group(0) @binding(0) var<uniform> a:Atmosphere;
@group(0) @binding(1) var opaque_depth:texture_depth_2d;
struct V { @builtin(position) pos:vec4<f32>, @location(0) uv:vec2<f32> };
@vertex fn vs_main(@builtin(vertex_index) i:u32)->V {let p=array<vec2<f32>,3>(vec2(-1.,-1.),vec2(3.,-1.),vec2(-1.,3.));var v:V;v.pos=vec4(p[i],0.,1.);v.uv=p[i];return v;}
fn interval(p:vec3<f32>,d:vec3<f32>,radius:f32)->vec2<f32> {let b=dot(p,d);let h=b*b-dot(p,p)+radius*radius;if h<0. {return vec2(1.,-1.);}let root=sqrt(h);return vec2(-b-root,-b+root);}
fn density(p:vec3<f32>)->f32 {return exp(-max(length(p)-1.,0.)/a.beta_height.w);}
@fragment fn fs_main(v:V)->@location(0) vec4<f32> {
 let ray=normalize(a.forward.xyz+v.uv.x*a.right.xyz*a.right.w+v.uv.y*a.up.xyz*a.up.w);
 let shell=interval(a.observer.xyz,ray,a.observer.w);let begin=max(shell.x,0.);var end=shell.y;
 let ground=interval(a.observer.xyz,ray,1.);if ground.x>0. {end=min(end,ground.x);}
 let z=textureLoad(opaque_depth,vec2<i32>(v.pos.xy),0);if z>0. {end=min(end,a.forward.w/(z*max(dot(ray,a.forward.xyz),0.00001)));}
 if end<=begin {return vec4(0.);}
 let step=(end-begin)/16.; var optical=0.;var scatter=vec3(0.);
 let mu=dot(ray,a.sun.xyz);let phase=0.0596831*(1.+mu*mu);
 for(var i=0u;i<16u;i++) {
  let p=a.observer.xyz+ray*(begin+(f32(i)+0.5)*step);let local=density(p)*step;
  let shadow=interval(p,a.sun.xyz,1.);let sun_shell=interval(p,a.sun.xyz,a.observer.w);var sun_optical=0.;
  if shadow.x<=0. || shadow.y<=0. {
   let ds=max(sun_shell.y,0.)/6.;for(var j=0u;j<6u;j++){sun_optical+=density(p+a.sun.xyz*((f32(j)+0.5)*ds))*ds;}
   scatter+=exp(-a.beta_height.xyz*(optical+0.5*local+sun_optical))*local*a.beta_height.xyz*phase*a.sun.w*a.color.rgb*8.;
  }
  optical+=local;
 }
 let transmittance=exp(-a.beta_height.xyz*optical);
 return vec4(scatter,1.-dot(transmittance,vec3(0.33333333)));
}
