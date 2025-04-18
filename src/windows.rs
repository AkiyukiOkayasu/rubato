use crate::Sample;

/// Different window functions that can be used to window the sinc function.
#[derive(Debug, Clone, Copy)]
pub enum WindowFunction {
    /// Blackman. Intermediate rolloff and intermediate attenuation.
    Blackman,
    /// Squared Blackman. Slower rolloff but better attenuation than Blackman.
    Blackman2,
    /// Blackman-Harris. Slow rolloff but good attenuation.
    BlackmanHarris,
    /// Squared Blackman-Harris. Slower rolloff but better attenuation than Blackman-Harris.
    BlackmanHarris2,
    /// Hann. Fast rolloff but not very high attenuation.
    Hann,
    /// Squared Hann. Slower rolloff and higher attenuation than simple Hann.
    Hann2,
}

/// Helper function. Standard Blackman-Harris window.
// The window created is periodic.
pub fn blackman_harris<T>(npoints: usize) -> Vec<T>
where
    T: Sample,
{
    trace!("Making a BlackmanHarris windows with {} points", npoints);
    let mut window = vec![T::zero(); npoints];
    let pi2 = T::coerce(2.0) * T::PI;
    let pi4 = T::coerce(4.0) * T::PI;
    let pi6 = T::coerce(6.0) * T::PI;
    let np_f = T::coerce(npoints);
    let a = T::coerce(0.35875);
    let b = T::coerce(0.48829);
    let c = T::coerce(0.14128);
    let d = T::coerce(0.01168);
    for (x, item) in window.iter_mut().enumerate() {
        let x_float = T::coerce(x);
        *item = a - b * (pi2 * x_float / np_f).cos() + c * (pi4 * x_float / np_f).cos()
            - d * (pi6 * x_float / np_f).cos();
    }
    window
}

/// Helper function. Standard Blackman window.
// The window created is periodic.
pub fn blackman<T>(npoints: usize) -> Vec<T>
where
    T: Sample,
{
    trace!("Making a Blackman windows with {} points", npoints);
    let mut window = vec![T::zero(); npoints];
    let pi2 = T::coerce(2.0) * T::PI;
    let pi4 = T::coerce(4.0) * T::PI;
    let np_f = T::coerce(npoints);
    let a = T::coerce(0.42);
    let b = T::coerce(0.5);
    let c = T::coerce(0.08);
    for (x, item) in window.iter_mut().enumerate() {
        let x_float = T::coerce(x);
        *item = a - b * (pi2 * x_float / np_f).cos() + c * (pi4 * x_float / np_f).cos();
    }
    window
}

/// Helper function. Standard Hann window.
// The window created is periodic.
pub fn hann<T>(npoints: usize) -> Vec<T>
where
    T: Sample,
{
    trace!("Making a Hann windows with {} points", npoints);
    let mut window = vec![T::zero(); npoints];
    let pi2 = T::coerce(2.0) * T::PI;
    let np_f = T::coerce(npoints);
    let a = T::coerce(0.5);
    for (x, item) in window.iter_mut().enumerate() {
        let x_float = T::coerce(x);
        *item = a - a * (pi2 * x_float / np_f).cos();
    }
    window
}

/// Make the selected window function.
pub fn make_window<T>(npoints: usize, windowfunc: WindowFunction) -> Vec<T>
where
    T: Sample,
{
    let mut window = match windowfunc {
        WindowFunction::BlackmanHarris | WindowFunction::BlackmanHarris2 => {
            blackman_harris::<T>(npoints)
        }
        WindowFunction::Blackman | WindowFunction::Blackman2 => blackman::<T>(npoints),
        WindowFunction::Hann | WindowFunction::Hann2 => hann::<T>(npoints),
    };
    match windowfunc {
        WindowFunction::Blackman2 | WindowFunction::BlackmanHarris2 | WindowFunction::Hann2 => {
            window.iter_mut().for_each(|y| *y = *y * *y);
        }
        _ => {}
    };
    window
}

/// Calculate a suitable relative cutoff frequency for the given sinc length using the given window function.
/// The result is based on an approximation, which gives good results for sinc lengths from 32 to 2048.
pub fn calculate_cutoff<T>(npoints: usize, windowfunc: WindowFunction) -> T
where
    T: Sample,
{
    // Coefficient values generated by cutoff_fit_cubic.py
    let (k1, k2, k3) = match windowfunc {
        WindowFunction::BlackmanHarris => (
            T::coerce(8.041443677716476),
            T::coerce(55.9506779343387),
            T::coerce(898.0287985384213),
        ),
        WindowFunction::BlackmanHarris2 => (
            T::coerce(13.745202940783823),
            T::coerce(121.73532586374934),
            T::coerce(5964.163279612051),
        ),
        WindowFunction::Blackman => (
            T::coerce(6.159598046201173),
            T::coerce(18.926415097606878),
            T::coerce(653.4247430458968),
        ),
        WindowFunction::Blackman2 => (
            T::coerce(9.506235102129398),
            T::coerce(79.13120634953742),
            T::coerce(1502.2316160588925),
        ),
        WindowFunction::Hann => (
            T::coerce(3.3481080887677166),
            T::coerce(10.106519434875038),
            T::coerce(78.96345249024414),
        ),
        WindowFunction::Hann2 => (
            T::coerce(5.38751148378734),
            T::coerce(29.69451915489501),
            T::coerce(184.82117462266237),
        ),
    };
    let one = T::one();
    let npoints_t = T::coerce(npoints);
    one / (k1 / npoints_t
        + k2 / (npoints_t * npoints_t)
        + k3 / (npoints_t * npoints_t * npoints_t)
        + one)
}

#[cfg(test)]
mod tests {
    extern crate approx;
    use crate::windows::blackman;
    use crate::windows::blackman_harris;
    use crate::windows::calculate_cutoff;
    use crate::windows::hann;
    use crate::windows::make_window;
    use crate::windows::WindowFunction;
    use approx::assert_abs_diff_eq;
    use test_log::test;

    #[test]
    fn test_blackman_harris() {
        let wnd = blackman_harris::<f64>(16);
        assert_abs_diff_eq!(wnd[8], 1.0, epsilon = 0.000001);
        assert!(wnd[0] < 0.001);
        assert!(wnd[15] < 0.1);
    }

    #[test]
    fn test_blackman() {
        let wnd = blackman::<f64>(16);
        assert_abs_diff_eq!(wnd[8], 1.0, epsilon = 0.000001);
        assert!(wnd[0] < 0.000001);
        assert!(wnd[15] < 0.1);
    }

    #[test]
    fn test_blackman2() {
        let wnd = make_window::<f64>(16, WindowFunction::Blackman);
        let wnd2 = make_window::<f64>(16, WindowFunction::Blackman2);
        assert_abs_diff_eq!(wnd[1] * wnd[1], wnd2[1], epsilon = 0.000001);
        assert_abs_diff_eq!(wnd[4] * wnd[4], wnd2[4], epsilon = 0.000001);
        assert_abs_diff_eq!(wnd[7] * wnd[7], wnd2[7], epsilon = 0.000001);
        assert!(wnd2[1] > 0.000001);
        assert!(wnd2[4] > 0.000001);
        assert!(wnd2[7] > 0.000001);
    }

    #[test]
    fn test_hann() {
        let wnd = hann::<f64>(16);
        assert_abs_diff_eq!(wnd[8], 1.0, epsilon = 0.000001);
        assert!(wnd[0] < 0.000001);
        assert!(wnd[15] < 0.1);
    }

    #[test]
    fn test_cutoff() {
        let cutoff = calculate_cutoff::<f64>(128, WindowFunction::Blackman);
        assert_abs_diff_eq!(cutoff, 0.953, epsilon = 0.001);
        let cutoff = calculate_cutoff::<f64>(256, WindowFunction::Blackman);
        assert_abs_diff_eq!(cutoff, 0.976, epsilon = 0.001);
        let cutoff = calculate_cutoff::<f64>(128, WindowFunction::Blackman2);
        assert_abs_diff_eq!(cutoff, 0.926, epsilon = 0.001);
        let cutoff = calculate_cutoff::<f64>(256, WindowFunction::Blackman2);
        assert_abs_diff_eq!(cutoff, 0.963, epsilon = 0.001);
        let cutoff = calculate_cutoff::<f64>(128, WindowFunction::BlackmanHarris);
        assert_abs_diff_eq!(cutoff, 0.937, epsilon = 0.001);
        let cutoff = calculate_cutoff::<f64>(256, WindowFunction::BlackmanHarris);
        assert_abs_diff_eq!(cutoff, 0.969, epsilon = 0.001);
        let cutoff = calculate_cutoff::<f64>(128, WindowFunction::BlackmanHarris2);
        assert_abs_diff_eq!(cutoff, 0.894, epsilon = 0.001);
        let cutoff = calculate_cutoff::<f64>(256, WindowFunction::BlackmanHarris2);
        assert_abs_diff_eq!(cutoff, 0.947, epsilon = 0.001);
        let cutoff = calculate_cutoff::<f64>(128, WindowFunction::Hann);
        assert_abs_diff_eq!(cutoff, 0.974, epsilon = 0.001);
        let cutoff = calculate_cutoff::<f64>(256, WindowFunction::Hann);
        assert_abs_diff_eq!(cutoff, 0.987, epsilon = 0.001);
        let cutoff = calculate_cutoff::<f64>(128, WindowFunction::Hann2);
        assert_abs_diff_eq!(cutoff, 0.958, epsilon = 0.001);
        let cutoff = calculate_cutoff::<f64>(256, WindowFunction::Hann2);
        assert_abs_diff_eq!(cutoff, 0.979, epsilon = 0.001);
    }
}
