//! Native NAD83 geographic to EPSG:5070 projection.
//!
//! Implements the ellipsoidal Albers Equal Area forward equations from
//! USGS Bulletin 1532 using the EPSG:5070 parameters and GRS80 ellipsoid.
//! Transcendental functions use the same Rust libm implementation on native
//! and WASM targets. Platform libc differences otherwise change graph hashes.

pub const PROJECTION_MATH: &str = "libm-f64-v1";

const SEMI_MAJOR_METRES: f64 = 6_378_137.0;
const INVERSE_FLATTENING: f64 = 298.257_222_101;
const LATITUDE_OF_ORIGIN_DEGREES: f64 = 23.0;
const CENTRAL_MERIDIAN_DEGREES: f64 = -96.0;
const STANDARD_PARALLEL_1_DEGREES: f64 = 29.5;
const STANDARD_PARALLEL_2_DEGREES: f64 = 45.5;

#[derive(Debug, Clone, Copy)]
struct AlbersConstants {
    eccentricity: f64,
    n: f64,
    c: f64,
    rho_origin: f64,
}

fn authalic_q(latitude: f64, eccentricity: f64) -> f64 {
    let sin_latitude = libm::sin(latitude);
    let e_sin = eccentricity * sin_latitude;
    let eccentricity_squared = eccentricity * eccentricity;
    (1.0 - eccentricity_squared)
        * (sin_latitude / (1.0 - e_sin * e_sin)
            - libm::log((1.0 - e_sin) / (1.0 + e_sin)) / (2.0 * eccentricity))
}

fn meridional_scale(latitude: f64, eccentricity_squared: f64) -> f64 {
    libm::cos(latitude) / libm::sqrt(1.0 - eccentricity_squared * libm::sin(latitude).powi(2))
}

fn constants() -> AlbersConstants {
    let flattening = 1.0 / INVERSE_FLATTENING;
    let eccentricity_squared = 2.0 * flattening - flattening * flattening;
    let eccentricity = libm::sqrt(eccentricity_squared);
    let phi_0 = LATITUDE_OF_ORIGIN_DEGREES.to_radians();
    let phi_1 = STANDARD_PARALLEL_1_DEGREES.to_radians();
    let phi_2 = STANDARD_PARALLEL_2_DEGREES.to_radians();
    let m_1 = meridional_scale(phi_1, eccentricity_squared);
    let m_2 = meridional_scale(phi_2, eccentricity_squared);
    let q_0 = authalic_q(phi_0, eccentricity);
    let q_1 = authalic_q(phi_1, eccentricity);
    let q_2 = authalic_q(phi_2, eccentricity);
    let n = (m_1 * m_1 - m_2 * m_2) / (q_2 - q_1);
    let c = m_1 * m_1 + n * q_1;
    let rho_origin = SEMI_MAJOR_METRES * libm::sqrt(c - n * q_0) / n;
    AlbersConstants {
        eccentricity,
        n,
        c,
        rho_origin,
    }
}

/// Project a NAD83 longitude/latitude pair in degrees to EPSG:5070 metres.
pub fn nad83_to_epsg5070(longitude_degrees: f64, latitude_degrees: f64) -> (f64, f64) {
    let parameters = constants();
    let latitude = latitude_degrees.to_radians();
    let theta = parameters.n * (longitude_degrees - CENTRAL_MERIDIAN_DEGREES).to_radians();
    let q = authalic_q(latitude, parameters.eccentricity);
    let rho = SEMI_MAJOR_METRES * libm::sqrt(parameters.c - parameters.n * q) / parameters.n;
    (rho * libm::sin(theta), parameters.rho_origin - rho * libm::cos(theta))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn false_origin_projects_to_zero() {
        let (x, y) = nad83_to_epsg5070(CENTRAL_MERIDIAN_DEGREES, LATITUDE_OF_ORIGIN_DEGREES);
        assert!(x.abs() < 1e-9, "false-origin easting: {x}");
        assert!(y.abs() < 1e-9, "false-origin northing: {y}");
    }

    #[test]
    fn central_meridian_is_symmetric() {
        let (west_x, west_y) = nad83_to_epsg5070(-106.0, 40.0);
        let (east_x, east_y) = nad83_to_epsg5070(-86.0, 40.0);
        assert!((west_x + east_x).abs() < 1e-8);
        assert!((west_y - east_y).abs() < 1e-8);
    }

    #[test]
    fn conus_coordinate_has_metre_scale() {
        let (x, y) = nad83_to_epsg5070(-75.0, 35.0);
        assert!(x > 1_500_000.0 && x < 2_500_000.0, "easting: {x}");
        assert!(y > 1_000_000.0 && y < 2_000_000.0, "northing: {y}");
    }

    #[test]
    fn portable_projection_agrees_with_independent_proj_coordinates() {
        // PROJ via pyproj 3.7.2, EPSG:4269 -> EPSG:5070, always_xy=true.
        // Micrometre tolerance tests accuracy, not cross-target byte identity.
        for (lon, lat, expected_x, expected_y) in [
            (-75.0, 35.0, 1_885_428.3905428401, 1_535_969.2858012654),
            (-71.4, 41.8, 2_006_951.8610253863, 2_349_754.233280133),
            (-150.0, 64.0, -2_951_378.515114467, 5_306_310.851495624),
            (-155.6, 19.6, -6_042_072.898642913, 1_592_255.3185960194),
        ] {
            let (x, y) = nad83_to_epsg5070(lon, lat);
            assert!((x - expected_x).abs() < 1e-6, "{lon},{lat} x={x}");
            assert!((y - expected_y).abs() < 1e-6, "{lon},{lat} y={y}");
        }
    }
}
