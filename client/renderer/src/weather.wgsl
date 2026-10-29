struct Weather {right:vec4<f32>,up:vec4<f32>,forward:vec4<f32>,observer:vec4<f32>,sun:vec4<f32>,color_radius:vec4<f32>,axis_x_base:vec4<f32>,axis_y_thickness:vec4<f32>,axis_z_time:vec4<f32>,size_samples:vec4<f32>,r0:vec4<f32>,r1:vec4<f32>,r2:vec4<f32>,r3:vec4<f32>};
@group(0) @binding(0) var<uniform> w:Weather;
@group(0) @binding(1) var opaque_depth:texture_depth_2d;
@group(0) @binding(2) var weather_field:texture_2d<f32>;
@group(0) @binding(3) var wind_field:texture_2d<f32>;
struct V {@builtin(position) pos:vec4<f32>,@location(0) uv:vec2<f32>};
@vertex fn vs_main(@builtin(vertex_index) i:u32)->V {let p=array<vec2<f32>,3>(vec2(-1.,-1.),vec2(3.,-1.),vec2(-1.,3.));var v:V;v.pos=vec4(p[i],0.,1.);v.uv=p[i];return v;}
fn bounds(p:vec3<f32>,d:vec3<f32>,radius:f32)->vec2<f32> {let b=dot(p,d);let h=b*b-dot(p,p)+radius*radius;if h<0. {return vec2(1.,-1.);}let r=sqrt(h);return vec2(-b-r,-b+r);}
fn body(p:vec3<f32>)->vec3<f32> {return vec3(dot(p,w.axis_x_base.xyz),dot(p,w.axis_y_thickness.xyz),dot(p,w.axis_z_time.xyz));}
fn wrap(p:vec2<i32>)->vec2<i32> {return vec2((p.x+64)%64,clamp(p.y,0,31));}
fn sample_field(texture:texture_2d<f32>,d:vec3<f32>)->vec4<f32> {
 let u=fract(atan2(d.z,d.x)/6.28318530718+1.);let v=asin(clamp(d.y,-1.,1.))/3.14159265359+0.5;
 let p=vec2(u*64.,v*32.)-0.5;let i=vec2<i32>(floor(p));let f=fract(p);
 return mix(mix(textureLoad(texture,wrap(i),0),textureLoad(texture,wrap(i+vec2(1,0)),0),f.x),mix(textureLoad(texture,wrap(i+vec2(0,1)),0),textureLoad(texture,wrap(i+vec2(1,1)),0),f.x),f.y);
}
@fragment fn fs_main(v:V)->@location(0) vec4<f32> {
 let ray=normalize(w.forward.xyz+v.uv.x*w.right.xyz*w.right.w+v.uv.y*w.up.xyz*w.up.w);
 let outer=1.+w.axis_x_base.w+w.axis_y_thickness.w;
 let range=bounds(w.observer.xyz,ray,outer);let begin=max(0.,range.x);var end=range.y;
 let ground=bounds(w.observer.xyz,ray,1.);if ground.x>0. {end=min(end,ground.x);}
 let z=textureLoad(opaque_depth,vec2<i32>(v.pos.xy),0);if z>0. {end=min(end,w.forward.w/(z*max(dot(ray,w.forward.xyz),0.00001)));}
 if end<=begin {return vec4(0.);}
 let step=(end-begin)/w.size_samples.w;var transmission=1.;var color=vec3(0.);
 for(var i=0u;i<24u;i++) {if f32(i)>=w.size_samples.w {break;}
  let p=w.observer.xyz+ray*(begin+(f32(i)+0.5)*step);let n=normalize(body(p));let weather=sample_field(weather_field,n);let wind=sample_field(wind_field,n);
  let h=max(length(p)-1.,0.);let cloud_height=(h-w.axis_x_base.w)/w.axis_y_thickness.w;
  let envelope=smoothstep(0.,0.15,cloud_height)*(1.-smoothstep(0.8,1.,cloud_height));
  // Only sub-grid visual texture sways with actual body-fixed wind. Macro fields are world samples.
  let advected=body(p)*w.color_radius.w-wind.xyz*(sin(w.axis_z_time.w)*500.);
  let phase=dot(advected,vec3(0.0007,0.0009,0.0005));
  let wisps=0.5+0.25*sin(phase)+0.25*sin(phase*2.3+dot(n,vec3(19.,7.,13.)));
  let cloud=smoothstep(1.-weather.x,1.,wisps)*weather.x*envelope*0.002;
  let dust=weather.w*exp(-h/max(w.axis_x_base.w,0.000001));
  // Stationary world-space streak pattern with shared-clock fall and wind; no surface accumulation.
  let streak=pow(max(0.,sin(dot(advected,vec3(0.9,0.01,0.7))+w.axis_z_time.w*4096.)),24.);
  let rain=weather.z*6.*streak*(1.-smoothstep(w.axis_x_base.w,outer-1.,h));
  let optical=(cloud+dust+rain)*step*w.color_radius.w;let alpha=1.-exp(-optical);
  let day=max(dot(normalize(p),w.sun.xyz),0.)*w.sun.w;
  let storm_shade=1.-0.65*weather.y;
  let condensate=mix(vec3(0.65,0.68,0.73),vec3(0.35,0.45,0.58),select(0.,1.,wind.w<w.r0.x));
  let tint=(condensate*(cloud+rain)+vec3(0.55,0.32,0.16)*dust)/max(cloud+rain+dust,0.0000001);
  color+=transmission*alpha*tint*(0.04+0.96*day)*storm_shade*w.color_radius.rgb;
  transmission*=1.-alpha;
 }
 return vec4(color,1.-transmission);
}
