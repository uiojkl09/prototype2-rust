//! Independent geometric capsule queries. Retail filtering/contact response is unknown.
use crate::{capsule::Capsule, scene::CollisionMesh};
use anyhow::{Result, bail, ensure};
use serde::Serialize;
type V = [f64; 3];
fn add(a: V, b: V) -> V {
    std::array::from_fn(|i| a[i] + b[i])
}
fn sub(a: V, b: V) -> V {
    std::array::from_fn(|i| a[i] - b[i])
}
fn scale(a: V, s: f64) -> V {
    a.map(|x| x * s)
}
fn dot(a: V, b: V) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
fn cross(a: V, b: V) -> V {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn norm(a: V) -> f64 {
    dot(a, a).sqrt()
}
fn point_segment(p: V, a: V, b: V) -> V {
    let d = sub(b, a);
    let dd = dot(d, d);
    add(
        a,
        scale(
            d,
            if dd > 0. {
                (dot(sub(p, a), d) / dd).clamp(0., 1.)
            } else {
                0.
            },
        ),
    )
}
fn in_triangle(p: V, a: V, b: V, c: V) -> bool {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);
    let aa = dot(ab, ab);
    let bb = dot(ac, ac);
    let abac = dot(ab, ac);
    let denominator = aa * bb - abac * abac;
    if denominator <= 1e-20 {
        return false;
    }
    let u = (bb * dot(ap, ab) - abac * dot(ap, ac)) / denominator;
    let v = (aa * dot(ap, ac) - abac * dot(ap, ab)) / denominator;
    u >= -1e-12 && v >= -1e-12 && u + v <= 1. + 1e-12
}
fn point_triangle(p: V, a: V, b: V, c: V) -> V {
    let n = cross(sub(b, a), sub(c, a));
    let nn = dot(n, n);
    if nn > 1e-20 {
        let q = sub(p, scale(n, dot(sub(p, a), n) / nn));
        if in_triangle(q, a, b, c) {
            return q;
        }
    }
    [
        point_segment(p, a, b),
        point_segment(p, b, c),
        point_segment(p, c, a),
    ]
    .into_iter()
    .min_by(|x, y| dot(sub(p, *x), sub(p, *x)).total_cmp(&dot(sub(p, *y), sub(p, *y))))
    .unwrap()
}
fn segment_segment(p: V, q: V, a: V, b: V) -> (V, V) {
    let d = sub(q, p);
    let e = sub(b, a);
    let r = sub(p, a);
    let dd = dot(d, d);
    let ee = dot(e, e);
    let de = dot(d, e);
    let dr = dot(d, r);
    let er = dot(e, r);
    if dd <= 1e-20 {
        return (p, point_segment(p, a, b));
    }
    if ee <= 1e-20 {
        return (point_segment(a, p, q), a);
    }
    let determinant = dd * ee - de * de;
    let mut s = if determinant > 1e-20 {
        (de * er - ee * dr) / determinant
    } else {
        0.
    }
    .clamp(0., 1.);
    let t = (de * s + er) / ee;
    let t = if t < 0. {
        s = (-dr / dd).clamp(0., 1.);
        0.
    } else if t > 1. {
        s = ((de - dr) / dd).clamp(0., 1.);
        1.
    } else {
        t
    };
    (add(p, scale(d, s)), add(a, scale(e, t)))
}
fn segment_triangle(p: V, q: V, tri: [V; 3]) -> (V, V) {
    let [a, b, c] = tri;
    let n = cross(sub(b, a), sub(c, a));
    let d = sub(q, p);
    let denominator = dot(n, d);
    if denominator.abs() > 1e-20 {
        let t = dot(n, sub(a, p)) / denominator;
        if (0. ..=1.).contains(&t) {
            let hit = add(p, scale(d, t));
            if in_triangle(hit, a, b, c) {
                return (hit, hit);
            }
        }
    }
    [
        (p, point_triangle(p, a, b, c)),
        (q, point_triangle(q, a, b, c)),
        segment_segment(p, q, a, b),
        segment_segment(p, q, b, c),
        segment_segment(p, q, c, a),
    ]
    .into_iter()
    .min_by(|(x, y), (u, v)| {
        dot(sub(*x, *y), sub(*x, *y)).total_cmp(&dot(sub(*u, *v), sub(*u, *v)))
    })
    .unwrap()
}
#[derive(Debug, Clone, Serialize)]
pub struct Hit {
    pub fraction: f64,
    pub point: V,
    pub normal: V,
    pub initial_overlap: bool,
    pub chunk_offset: usize,
    pub face_index: usize,
    pub unknown_face_tag: u16,
}
/// Earliest contact under translation; two-sided triangles, no response or retail tags.
/// Skin is a caller-selected numerical tolerance, not a recovered gameplay margin.
pub fn sweep(
    meshes: &[CollisionMesh],
    capsule: &Capsule,
    origin: V,
    delta: V,
    skin: f64,
) -> Result<Option<Hit>> {
    capsule.validate()?;
    ensure!(
        origin.iter().chain(&delta).all(|x| x.is_finite()) && norm(delta).is_finite(),
        "invalid sweep vectors"
    );
    ensure!(
        skin.is_finite() && (0. ..=0.1).contains(&skin),
        "invalid query skin"
    );
    let ends = capsule.endpoints(origin);
    ensure!(
        ends.iter().flatten().all(|x| x.is_finite()),
        "capsule endpoint overflow"
    );
    let radius = f64::from(capsule.radius);
    let lower: V =
        std::array::from_fn(|i| ends[0][i].min(ends[1][i]) + delta[i].min(0.) - radius - skin);
    let upper: V =
        std::array::from_fn(|i| ends[0][i].max(ends[1][i]) + delta[i].max(0.) + radius + skin);
    ensure!(
        lower.iter().chain(&upper).all(|x| x.is_finite()),
        "sweep bounds overflow"
    );
    let mut best: Option<Hit> = None;
    for mesh in meshes {
        for (face_index, face) in mesh.faces.iter().enumerate() {
            let mut tri = [[0.; 3]; 3];
            for i in 0..3 {
                tri[i] = mesh
                    .positions
                    .get(usize::from(face[i]))
                    .ok_or_else(|| anyhow::anyhow!("collision face index out of range"))?
                    .map(f64::from);
            }
            ensure!(
                tri.iter().flatten().all(|x| x.is_finite()),
                "non-finite collision vertex"
            );
            if (0..3)
                .any(|i| tri.iter().all(|v| v[i] < lower[i]) || tri.iter().all(|v| v[i] > upper[i]))
            {
                continue;
            }
            let mut fraction = 0.;
            let mut finished = false;
            for _ in 0..64 {
                let translation = scale(delta, fraction);
                let (p, q) =
                    segment_triangle(add(ends[0], translation), add(ends[1], translation), tri);
                let offset = sub(p, q);
                let distance = norm(offset);
                let normal = if distance > 1e-12 {
                    scale(offset, 1. / distance)
                } else {
                    let n = cross(sub(tri[1], tri[0]), sub(tri[2], tri[0]));
                    let length = norm(n);
                    if length <= 1e-12 {
                        let travel = norm(delta);
                        if travel > 1e-12 {
                            scale(delta, -1. / travel)
                        } else {
                            [0., 1., 0.]
                        }
                    } else {
                        scale(
                            n,
                            if dot(n, delta) > 0. {
                                -1. / length
                            } else {
                                1. / length
                            },
                        )
                    }
                };
                let gap = distance - radius - skin;
                let closing = -dot(normal, delta);
                let overlap = fraction == 0. && distance < radius - 1e-8;
                if overlap || (gap <= 1e-8 && closing > 1e-12) {
                    let hit = Hit {
                        fraction,
                        point: q,
                        normal,
                        initial_overlap: overlap,
                        chunk_offset: mesh.chunk_offset,
                        face_index,
                        unknown_face_tag: face[3],
                    };
                    if best.as_ref().is_none_or(|old| fraction < old.fraction) {
                        best = Some(hit);
                    }
                    finished = true;
                    break;
                }
                if closing <= 1e-12 {
                    finished = true;
                    break;
                }
                fraction += gap.max(0.) / closing;
                if fraction > best.as_ref().map_or(1., |h| h.fraction) {
                    finished = true;
                    break;
                }
            }
            if !finished {
                bail!(
                    "capsule sweep did not converge at chunk 0x{:x}, face {face_index}",
                    mesh.chunk_offset
                );
            }
        }
    }
    Ok(best)
}
