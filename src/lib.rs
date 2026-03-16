/*
 * // Copyright (c) Radzivon Bartoshyk 3/2026. All rights reserved.
 * //
 * // Redistribution and use in source and binary forms, with or without modification,
 * // are permitted provided that the following conditions are met:
 * //
 * // 1.  Redistributions of source code must retain the above copyright notice, this
 * // list of conditions and the following disclaimer.
 * //
 * // 2.  Redistributions in binary form must reproduce the above copyright notice,
 * // this list of conditions and the following disclaimer in the documentation
 * // and/or other materials provided with the distribution.
 * //
 * // 3.  Neither the name of the copyright holder nor the names of its
 * // contributors may be used to endorse or promote products derived from
 * // this software without specific prior written permission.
 * //
 * // THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * // AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * // IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * // DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
 * // FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * // DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
 * // SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
 * // CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
 * // OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * // OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
pub use crate::err::SpikefitError;
use num_traits::float::FloatCore;
use std::fmt::{Debug, Display};
use std::ops::{Mul, Sub};

mod err;
mod prominence;
mod searcher;
mod widths;

/// Builder for computing peak widths.
///
/// Construct with [`PeakWidthsOptions::new`], optionally override
/// [`peaks`](PeakWidthsOptions::peaks) and/or
/// [`rel_height`](PeakWidthsOptions::rel_height), then call
/// [`compute`](PeakWidthsOptions::find).
///
/// # Examples
///
/// ```no_test
/// // Compute widths for all detected peaks at half-prominence (default):
/// let result = PeakWidthsBuilder::new(&signal)
///     .find()?;
///
/// // Specify a custom peak list and measure at 80 % of prominence:
/// let result = PeakWidthsBuilder::new(&signal)
///     .peaks(&[3, 7, 14])
///     .rel_height(0.8)
///     .find()?;
/// ```
pub struct PeakWidthsOptions<'a, T> {
    /// The signal.
    x: &'a [T],
    /// Caller-supplied peak indices.  `None` → auto-detect.
    peaks: Option<&'a [usize]>,
    /// Relative height for the width measurement.  `None` → 0.5.
    rel_height: Option<T>,
}

impl<'a, T> PeakWidthsOptions<'a, T> {
    /// Create a builder for signal `x`.
    pub fn new(x: &'a [T]) -> Self {
        Self {
            x,
            peaks: None,
            rel_height: None,
        }
    }

    /// Override the peak indices to use.
    ///
    /// When not called the builder auto-detects all local maxima.
    pub fn peaks(mut self, peaks: &'a [usize]) -> Self {
        self.peaks = Some(peaks);
        self
    }

    /// Set the relative height at which to measure widths.
    ///
    /// Must be in `[0.0, 1.0]`.  Defaults to `0.5` (half-prominence / FWHM).
    pub fn rel_height(mut self, rh: T) -> Self {
        self.rel_height = Some(rh);
        self
    }
}

impl<'a> PeakWidthsOptions<'a, f32> {
    pub fn find(&self) -> Result<SpikefitWidths<f32>, SpikefitError> {
        peak_widths_f32(self.x, self)
    }
}

/// Builder for [`find_peaks`].
///
/// Construct with [`FindPeaksOptions::new`], chain any combination of filter
/// methods, then call [`compute`](FindPeaksOptions::find).
///
/// # Examples
///
/// ```no_test
/// // All local maxima — no filters:
/// let peaks = FindPeaksOptions::new(&signal).compute()?;
///
/// // Full filter chain:
/// let peaks = FindPeaksBuilder::new(&signal)
///     .height(3.0)
///     .threshold(0.5)
///     .distance(4)
///     .prominence(1.0)
///     .width(2.0)
///     .find()?;
/// ```
pub struct FindPeaksOptions<'a, T> {
    x: &'a [T],
    height: Option<f64>,
    threshold: Option<f64>,
    distance: Option<usize>,
    prominence: Option<f64>,
    width: Option<f64>,
}

impl<'a, T> FindPeaksOptions<'a, T> {
    /// Create a builder for signal `x`.
    pub fn new(x: &'a [T]) -> Self {
        Self {
            x,
            height: None,
            threshold: None,
            distance: None,
            prominence: None,
            width: None,
        }
    }

    /// Minimum absolute peak height.
    pub fn height(mut self, h: f64) -> Self {
        self.height = Some(h);
        self
    }

    /// Minimum vertical distance from a peak to each of its direct neighbours.
    pub fn threshold(mut self, t: f64) -> Self {
        self.threshold = Some(t);
        self
    }

    /// Minimum number of samples between any two peaks.
    ///
    /// When two peaks are closer than `d`, the shorter one is discarded.
    pub fn distance(mut self, d: usize) -> Self {
        self.distance = Some(d);
        self
    }

    /// Minimum peak prominence.
    pub fn prominence(mut self, p: f64) -> Self {
        self.prominence = Some(p);
        self
    }

    /// Minimum peak width (in samples, measured at half-prominence).
    pub fn width(mut self, w: f64) -> Self {
        self.width = Some(w);
        self
    }
}

impl FindPeaksOptions<'_, f32> {
    /// Run the peak-detection pipeline and return the matching indices.
    ///
    /// Filters are applied in the same order as the original `find_peaks`:
    /// height → threshold → distance → prominence → width.
    ///
    /// # Errors
    ///
    /// Propagates any [`SpikefitError`] produced by the underlying helpers.
    pub fn find(self) -> Result<Vec<usize>, SpikefitError> {
        find_peaks_impl(
            self.x,
            self.height,
            self.threshold,
            self.distance,
            self.prominence,
            self.width,
        )
    }
}

impl FindPeaksOptions<'_, f64> {
    /// Run the peak-detection pipeline and return the matching indices.
    ///
    /// Filters are applied in the same order as the original `find_peaks`:
    /// height → threshold → distance → prominence → width.
    ///
    /// # Errors
    ///
    /// Propagates any [`SpikefitError`] produced by the underlying helpers.
    pub fn find(self) -> Result<Vec<usize>, SpikefitError> {
        find_peaks_impl(
            self.x,
            self.height,
            self.threshold,
            self.distance,
            self.prominence,
            self.width,
        )
    }
}

pub(crate) trait SpikefitSample:
    Debug
    + 'static
    + PartialOrd
    + PartialEq
    + Sub<Self, Output = Self>
    + Sized
    + Copy
    + Mul<Self, Output = Self>
    + FloatCore
    + Display
    + Default
{
    const HALF: Self;
    const TWO: Self;
}

impl SpikefitSample for f64 {
    const HALF: Self = 0.5;
    const TWO: Self = 2.0;
}
impl SpikefitSample for f32 {
    const HALF: Self = 0.5;
    const TWO: Self = 2.0;
}

/// Calculate the prominences of peaks in a signal.
///
/// The prominence of a peak measures how much the peak stands out due to its
/// intrinsic height and location relative to other peaks.
///
/// # Arguments
///
/// * `x` - The signal in which the peaks occur
/// * `peaks` - Indices of peaks in `x`
///
/// # Returns
///
/// * Vector of prominences for each peak
pub fn peak_prominences(x: &[f64], peaks: &[usize]) -> Result<Vec<f64>, SpikefitError> {
    peak_prominences_impl(x, peaks)
}

/// Calculate the prominences of peaks in a signal.
///
/// The prominence of a peak measures how much the peak stands out due to its
/// intrinsic height and location relative to other peaks.
///
/// # Arguments
///
/// * `x` - The signal in which the peaks occur
/// * `peaks` - Indices of peaks in `x`
///
/// # Returns
///
/// * Vector of prominences for each peak
pub fn peak_prominences_f32(x: &[f32], peaks: &[usize]) -> Result<Vec<f32>, SpikefitError> {
    peak_prominences_impl(x, peaks)
}

use crate::prominence::peak_prominences_impl;
use crate::searcher::find_peaks_impl;
use crate::widths::peak_widths_impl;
pub use widths::SpikefitWidths;

/// Calculate the width of peaks in a signal at a relative height.
///
/// # Arguments
///
/// * `x` - The signal in which the peaks occur
/// * `peaks` - Indices of peaks in `x`
/// * `rel_height` - Relative height of the boundary with respect to the peak height (default: 0.5)
pub fn peak_widths(
    x: &[f64],
    options: &PeakWidthsOptions<'_, f64>,
) -> Result<SpikefitWidths<f64>, SpikefitError> {
    if let Some(peaks) = options.peaks {
        peak_widths_impl(x, peaks, options.rel_height)
    } else {
        Err(SpikefitError::SignalTooShort { len: 0 })
    }
}

/// Calculate the width of peaks in a signal at a relative height.
///
/// # Arguments
///
/// * `x` - The signal in which the peaks occur
/// * `peaks` - Indices of peaks in `x`
/// * `rel_height` - Relative height of the boundary with respect to the peak height (default: 0.5)
pub fn peak_widths_f32(
    x: &[f32],
    options: &PeakWidthsOptions<'_, f32>,
) -> Result<SpikefitWidths<f32>, SpikefitError> {
    if let Some(peaks) = options.peaks {
        peak_widths_impl(x, peaks, options.rel_height)
    } else {
        Err(SpikefitError::SignalTooShort { len: 0 })
    }
}

/// Find peaks in a 1D signal.
///
/// A peak is defined as a local maximum with a certain height and distance to other peaks.
///
/// # Arguments
///
/// * `x` - The signal to find peaks in
/// * `height` - Optional minimum peak height (a value, a tuple (min, max), or a vector of values)
/// * `threshold` - Optional minimum height difference to neighboring samples
/// * `distance` - Optional minimum distance between peaks (in samples)
/// * `prominence` - Optional minimum peak prominence
/// * `width` - Optional minimum peak width
///
/// # Returns
///
/// * Vector of peak indices
pub fn find_peaks(
    x: &[f64],
    options: FindPeaksOptions<'_, f64>,
) -> Result<Vec<usize>, SpikefitError> {
    find_peaks_impl(
        x,
        options.height,
        options.threshold,
        options.distance,
        options.prominence,
        options.width,
    )
}

/// Find peaks in a 1D signal.
///
/// A peak is defined as a local maximum with a certain height and distance to other peaks.
///
/// # Arguments
///
/// * `x` - The signal to find peaks in
/// * `height` - Optional minimum peak height (a value, a tuple (min, max), or a vector of values)
/// * `threshold` - Optional minimum height difference to neighboring samples
/// * `distance` - Optional minimum distance between peaks (in samples)
/// * `prominence` - Optional minimum peak prominence
/// * `width` - Optional minimum peak width
///
/// # Returns
///
/// * Vector of peak indices
pub fn find_peaks_f32(
    x: &[f32],
    options: FindPeaksOptions<'_, f32>,
) -> Result<Vec<usize>, SpikefitError> {
    find_peaks_impl(
        x,
        options.height,
        options.threshold,
        options.distance,
        options.prominence,
        options.width,
    )
}
