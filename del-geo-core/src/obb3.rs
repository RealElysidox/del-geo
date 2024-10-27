//! 3D Oriented Bounding Box (OBB)

use std::f64::consts::PI;

use rand::distributions::{Distribution, Standard};

pub fn from_random<RAND, Real>(reng: &mut RAND) -> [Real; 12]
where
    RAND: rand::Rng,
    Real: num_traits::Float,
    Standard: Distribution<Real>,
{
    let one = Real::one();
    let aabb_m1p1 = [-one, -one, -one, one, one, one];
    let cntr = crate::aabb3::sample(&aabb_m1p1, reng);
    let u = crate::aabb3::sample(&aabb_m1p1, reng);
    let v = crate::aabb3::sample(&aabb_m1p1, reng);
    let v = crate::vec3::orthogonalize(&u, &v);
    let w = crate::aabb3::sample(&aabb_m1p1, reng);
    let w = crate::vec3::orthogonalize(&u, &w);
    let w = crate::vec3::orthogonalize(&v, &w);
    [
        cntr[0], cntr[1], cntr[2], u[0], u[1], u[2], v[0], v[1], v[2], w[0], w[1], w[2],
    ]
}

pub fn is_include_point<Real>(obb: &[Real; 12], p: &[Real; 3], eps: Real) -> bool
where
    Real: num_traits::Float,
{
    let s = Real::one() + eps;
    let d = [p[0] - obb[0], p[1] - obb[1], p[2] - obb[2]];
    {
        let lx = obb[3] * obb[3] + obb[4] * obb[4] + obb[5] * obb[5];
        let dx = obb[3] * d[0] + obb[4] * d[1] + obb[5] * d[2];
        if dx.abs() > lx * s {
            return false;
        }
    }
    {
        let ly = obb[6] * obb[6] + obb[7] * obb[7] + obb[8] * obb[8];
        let dy = obb[6] * d[0] + obb[7] * d[1] + obb[8] * d[2];
        if dy.abs() > ly * s {
            return false;
        }
    }
    {
        let lz = obb[9] * obb[9] + obb[10] * obb[10] + obb[11] * obb[11];
        let dz = obb[9] * d[0] + obb[10] * d[1] + obb[11] * d[2];
        if dz.abs() > lz * s {
            return false;
        }
    }
    true
}

/// return the normalized axes and the magnitude of each axis
pub fn unit_axes_and_half_edge_lengths<Real>(obb: &[Real; 12]) -> ([[Real; 3]; 3], [Real; 3])
where
    Real: num_traits::Float,
{
    let l0 = (obb[3] * obb[3] + obb[4] * obb[4] + obb[5] * obb[5]).sqrt();
    let l1 = (obb[6] * obb[6] + obb[7] * obb[7] + obb[8] * obb[8]).sqrt();
    let l2 = (obb[9] * obb[9] + obb[10] * obb[10] + obb[11] * obb[11]).sqrt();
    let l0_inv = Real::one() / l0;
    let l1_inv = Real::one() / l1;
    let l2_inv = Real::one() / l2;
    let axes = [
        [obb[3] * l0_inv, obb[4] * l0_inv, obb[5] * l0_inv],
        [obb[6] * l1_inv, obb[7] * l1_inv, obb[8] * l1_inv],
        [obb[9] * l2_inv, obb[10] * l2_inv, obb[11] * l2_inv],
    ];
    let sizes = [l0, l1, l2];
    (axes, sizes)
}

/// Projection of an OBB at axis, return (min,max)
pub fn range_axis<Real>(obb: &[Real; 12], axis: &[Real; 3]) -> (Real, Real)
where
    Real: num_traits::Float,
{
    let c = obb[0] * axis[0] + obb[1] * axis[1] + obb[2] * axis[2];
    let x = (obb[3] * axis[0] + obb[4] * axis[1] + obb[5] * axis[2]).abs();
    let y = (obb[6] * axis[0] + obb[7] * axis[1] + obb[8] * axis[2]).abs();
    let z = (obb[9] * axis[0] + obb[10] * axis[1] + obb[11] * axis[2]).abs();
    (c - x - y - z, c + x + y + z)
}

/// Find the distance of two ranges, return None if they are overlapped
fn distance_between_two_ranges<Real>(a: (Real, Real), b: (Real, Real)) -> Option<Real>
where
    Real: num_traits::Float,
{
    if a.0 > b.1 {
        return Some(a.0 - b.1);
    }
    if b.0 > a.1 {
        return Some(b.0 - a.1);
    }
    None
}

pub fn distance_to_obb3<Real>(obb_i: &[Real; 12], obb_j: &[Real; 12]) -> Real
where
    Real: num_traits::Float,
{
    let center_i: [Real; 3] = obb_i[0..3].try_into().unwrap();
    let axis_size_i = unit_axes_and_half_edge_lengths(obb_i);
    let center_j: [Real; 3] = obb_j[0..3].try_into().unwrap();
    let axis_size_j = unit_axes_and_half_edge_lengths(obb_j);
    let mut max_dist = Real::zero();

    for i in 0..3 {
        let axis_i = axis_size_i.0[i];
        let lh_i = axis_size_i.1[i];
        let c_i = crate::vec3::dot(&axis_i, &center_i);
        let range_i = (c_i - lh_i, c_i + lh_i);
        let range_j = range_axis(obb_j, &axis_i);
        let Some(dist) = distance_between_two_ranges(range_i, range_j) else {
            continue;
        };
        if dist > max_dist {
            max_dist = dist;
        }
    }
    for j in 0..3 {
        let axis_j = axis_size_j.0[j];
        let lh_j = axis_size_j.1[j];
        let c_j = crate::vec3::dot(&axis_j, &center_j);
        let range_j = (c_j - lh_j, c_j + lh_j);
        let range_i = range_axis(obb_i, &axis_j);
        let Some(dist) = distance_between_two_ranges(range_i, range_j) else {
            continue;
        };
        if dist > max_dist {
            max_dist = dist;
        }
    }
    for i in 0..3 {
        let axis_i = axis_size_i.0[i];
        for j in 0..3 {
            let axis_j = axis_size_j.0[j];
            let axis = crate::vec3::cross(&axis_i, &axis_j);
            let range_i = range_axis(obb_i, &axis);
            let range_j = range_axis(obb_j, &axis);
            let Some(dist) = distance_between_two_ranges(range_i, range_j) else {
                continue;
            };
            if dist > max_dist {
                max_dist = dist;
            }
        }
    }
    max_dist
}

pub fn nearest_to_point3<Real>(obb: &[Real; 12], p: &[Real; 3]) -> [Real; 3]
where
    Real: num_traits::Float,
{
    if is_include_point(&obb, p, Real::zero()) {
        return *p;
    }
    let (axes, hlen) = unit_axes_and_half_edge_lengths(obb);
    let d = [p[0] - obb[0], p[1] - obb[1], p[2] - obb[2]];
    let t0 = crate::vec3::dot(&axes[0], &d).clamp(-hlen[0], hlen[0]);
    let t1 = crate::vec3::dot(&axes[1], &d).clamp(-hlen[1], hlen[1]);
    let t2 = crate::vec3::dot(&axes[2], &d).clamp(-hlen[2], hlen[2]);
    [
        obb[0] + t0 * axes[0][0] + t1 * axes[1][0] + t2 * axes[2][0],
        obb[1] + t0 * axes[0][1] + t1 * axes[1][1] + t2 * axes[2][1],
        obb[2] + t0 * axes[0][2] + t1 * axes[1][2] + t2 * axes[2][2],
    ]
}

#[test]
fn test_nearest_to_point3() {
    use rand::SeedableRng;
    let mut reng = rand_chacha::ChaChaRng::seed_from_u64(0u64);
    for _itr in 0..100 {
        let obb = from_random::<_, f64>(&mut reng);
        let p = crate::aabb3::sample(&[-1., -1., -1., 1., 1., 1.], &mut reng);
        let p_near = nearest_to_point3(&obb, &p);
        for _iter in 0..10 {
            let eps = 1.0e-2;
            let dp = crate::aabb3::sample(&[-eps, -eps, -eps, eps, eps, eps], &mut reng);
            let q = [p_near[0] + dp[0], p_near[1] + dp[1], p_near[2] + dp[2]];
            let q = nearest_to_point3(&obb, &q);
            let len0 = crate::edge3::length(&p, &p_near);
            let len1 = crate::edge3::length(&p, &q);
            dbg!(len0, len1);
            assert!(len0 <= len1);
        }
    }
}

/// Use Separating Axis Theorem (SAT) to check if two OBBs are intersected
pub fn is_intersect_to_obb3<Real>(obb_i: &[Real; 12], obb_j: &[Real; 12]) -> bool
where
    Real: num_traits::Float,
{
    let axes = {
        let (axes_i, _) = unit_axes_and_half_edge_lengths(&obb_i);
        let (axes_j, _) = unit_axes_and_half_edge_lengths(&obb_j);
        [
            axes_i[0], axes_i[1], axes_i[2], axes_j[0], axes_j[1], axes_j[2],
        ]
    };
    for axis in axes.iter() {
        let range_i = range_axis(obb_i, axis);
        let range_j = range_axis(obb_j, axis);
        if distance_between_two_ranges(range_i, range_j).is_some() {
            return false;
        }
    }
    true
}

#[test]
fn test_is_intersect_to_obb3() {
    use rand::SeedableRng;
    let mut reng = rand_chacha::ChaChaRng::seed_from_u64(0u64);
    for _iter in 0..100 {
        let obb_i = from_random::<_, f64>(&mut reng);
        let obb_j = from_random(&mut reng);
        let res0 = is_intersect_to_obb3(&obb_i, &obb_j);
        let p0 = arrayref::array_ref![obb_i, 0, 3]; // center
        let p1 = nearest_to_point3(&obb_j, p0);
        let p2 = nearest_to_point3(&obb_i, &p1);
        let p3 = nearest_to_point3(&obb_j, &p2);
        let p4 = nearest_to_point3(&obb_i, &p3);
        let p5 = nearest_to_point3(&obb_j, &p4);
        let p6 = nearest_to_point3(&obb_i, &p5);
        let len45 = crate::edge3::length(&p4, &p5);
        let len56 = crate::edge3::length(&p5, &p6);
        assert!(len56 <= len45);
        if len56 > 0. && len56 < len45 {
            continue;
        } // still converging
        let res1 = len56 < 0.0001;
        assert_eq!(res0, res1, "{} {}", len45, len56);
    }
}

#[test]
fn test2_is_intersect_to_obb3() {
    for i in 0..2 {
        let obb_i = [0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 0., 1.];

        // special obb need to check additional axes other than six orientation axes
        // this test will fail if not checking addtional axes
        let d = 2.0 + 0.01 * (i as f64); // small distance to the obbs make them seperated
        let quad = crate::quaternion::around_axis(&[1., 0., -1.], -PI * 0.5f64);
        let rot_mat = crate::quaternion::to_mat3_col_major(&quad);
        let u = crate::mat3_col_major::mult_vec(&rot_mat, &[1., 0., 0.]);
        let v = crate::mat3_col_major::mult_vec(&rot_mat, &[0., 1., 0.]);
        let w = crate::mat3_col_major::mult_vec(&rot_mat, &[0., 0., 1.]);
        let obb_j = [
            d, 0., -d, u[0], u[1], u[2], v[0], v[1], v[2], w[0], w[1], w[2],
        ];

        let p0 = arrayref::array_ref![obb_i, 0, 3]; // center
        let p1 = nearest_to_point3(&obb_j, p0);
        let p2 = nearest_to_point3(&obb_i, &p1);
        let p3 = nearest_to_point3(&obb_j, &p2);
        let p4 = nearest_to_point3(&obb_i, &p3);
        let p5 = nearest_to_point3(&obb_j, &p4);
        let p6 = nearest_to_point3(&obb_i, &p5);

        let len45 = crate::edge3::length(&p4, &p5);
        let len56 = crate::edge3::length(&p5, &p6);
        assert!(len56 <= len45);

        let res1 = len56 < 0.0001; // this will be false since not intersected
        let res0 = is_intersect_to_obb3(&obb_i, &obb_j);
        assert_eq!(res0, res1, "{} {}", len45, len56);
    }
}

/// Get 8 corners of an obb3
/// The first four points defines the front face of the obb3, the last four defines back face.
pub fn corner_points<Real>(obb: &[Real; 12]) -> [[Real; 3]; 8]
where
    Real: num_traits::Float,
{
    [
        [
            obb[0] - obb[3] - obb[6] - obb[9],
            obb[1] - obb[4] - obb[7] - obb[10],
            obb[2] - obb[5] - obb[8] - obb[11],
        ],
        [
            obb[0] + obb[3] - obb[6] - obb[9],
            obb[1] + obb[4] - obb[7] - obb[10],
            obb[2] + obb[5] - obb[8] - obb[11],
        ],
        [
            obb[0] + obb[3] + obb[6] - obb[9],
            obb[1] + obb[4] + obb[7] - obb[10],
            obb[2] + obb[5] + obb[8] - obb[11],
        ],
        [
            obb[0] - obb[3] + obb[6] - obb[9],
            obb[1] - obb[4] + obb[7] - obb[10],
            obb[2] - obb[5] + obb[8] - obb[11],
        ],
        [
            obb[0] - obb[3] - obb[6] + obb[9],
            obb[1] - obb[4] - obb[7] + obb[10],
            obb[2] - obb[5] - obb[8] + obb[11],
        ],
        [
            obb[0] + obb[3] - obb[6] + obb[9],
            obb[1] + obb[4] - obb[7] + obb[10],
            obb[2] + obb[5] - obb[8] + obb[11],
        ],
        [
            obb[0] + obb[3] + obb[6] + obb[9],
            obb[1] + obb[4] + obb[7] + obb[10],
            obb[2] + obb[5] + obb[8] + obb[11],
        ],
        [
            obb[0] - obb[3] + obb[6] + obb[9],
            obb[1] - obb[4] + obb[7] + obb[10],
            obb[2] - obb[5] + obb[8] + obb[11],
        ],
    ]
}
