// Fixed contact bands in object space: no time, noise, textures or mutable geometry.
// Parameters: bounded age, material response, stable ID seed, reserved.
fn surface_wear(base: vec3<f32>, p: vec3<f32>, age: vec4<f32>) -> vec3<f32> {
    let contact = 1.0-smoothstep(0.18,0.55,abs(p.z));
    let bands = 1.0-smoothstep(0.025,0.07,abs(fract((p.x+p.y*0.3+age.z*0.031)*2.0)-0.5));
    let mask = max(contact*0.55,bands*0.8);
    var worn = mix(base,vec3<f32>(0.40,0.43,0.45),0.7); // painted scuffs/grip wear
    if age.y > 0.5 { worn=min(base*1.22+vec3<f32>(0.025),vec3<f32>(1.0)); } // polished contact metal
    if age.y > 1.5 { worn=mix(base,vec3<f32>(0.55,0.48,0.40),0.55); } // woven fabric fade
    if age.y > 2.5 { worn=base*vec3<f32>(1.15,0.77,0.55); } // moderate heat discoloration
    return mix(base,worn,age.x*mask);
}
