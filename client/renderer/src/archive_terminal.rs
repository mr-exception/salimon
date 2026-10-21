//! Code-authored flush cabin wall terminal, rendered in the existing ship draw.
/// Ship-local meters. Faces +Z, opposite the Fabricator, outside the walking lane.
pub const ARCHIVE_TERMINAL_METERS: [f64; 3] = [-3.0, 1.7, -4.13];
pub(crate) fn append(vertices: &mut Vec<f32>) {
    for (width, height, z, display, color) in [
        (0.46, 0.36, -4.15, 0.0, [0.035, 0.045, 0.05, 1.0]),
        (0.40, 0.30, -4.13, 5.0, [0.04, 0.12, 0.13, 1.0]),
    ] {
        for [u, v] in [[0., 0.], [0., 1.], [1., 1.], [0., 0.], [1., 1.], [1., 0.]] {
            vertices.extend_from_slice(&[
                -3.0 + (u * 2.0 - 1.0) * width,
                1.7 + (1.0 - v * 2.0) * height,
                z,
                0.,
                0.,
                1.,
                color[0],
                color[1],
                color[2],
                color[3],
                0.,
                0.,
                0.,
                0.,
                1.,
                u,
                v,
                display,
                0.,
            ]);
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn terminal_faces_cabin_and_uvs_are_viewer_right_top_left() {
        let mut v = Vec::new();
        super::append(&mut v);
        assert_eq!(v.len(), 12 * 19);
        let screen: Vec<_> = v
            .as_chunks::<19>()
            .0
            .iter()
            .filter(|v| v[17] == 5.0)
            .collect();
        for v in screen {
            assert_eq!(&v[3..6], &[0., 0., 1.]);
            assert!((v[0] - (-3.4 + v[15] * 0.8)).abs() < 1e-6);
            assert!((v[1] - (2.0 - v[16] * 0.6)).abs() < 1e-6);
        }
    }
}
