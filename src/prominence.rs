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
use crate::SpikefitSample;
use crate::err::SpikefitError;

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
pub(crate) fn peak_prominences_impl<T: SpikefitSample>(
    x: &[T],
    peaks: &[usize],
) -> Result<Vec<T>, SpikefitError> {
    if x.is_empty() {
        return Err(SpikefitError::SignalTooShort { len: 0 });
    }

    if peaks.is_empty() {
        return Ok(Vec::new());
    }

    let mut prominences = Vec::with_capacity(peaks.len());

    for &peak_idx in peaks {
        if peak_idx >= x.len() {
            return Err(SpikefitError::PeakOutOfBounds {
                index: peak_idx,
                len: x.len(),
            });
        }

        let peak_height = x[peak_idx];

        // Find left and right bounds of the peak
        // These are the lowest points between this peak and higher peaks
        // or the edges of the signal

        // Find minimum to the left
        let mut left_min = peak_height;
        let mut left_reached_minimum = false;
        for i in (0..peak_idx).rev() {
            if x[i] < left_min {
                left_min = x[i];
                left_reached_minimum = true;
            } else if left_reached_minimum && x[i] > left_min {
                // Stop when we start rising again after finding a minimum
                break;
            }
            // Stop if we hit a higher peak
            if x[i] > peak_height {
                break;
            }
        }

        // Find minimum to the right
        let mut right_min = peak_height;
        let mut right_reached_minimum = false;
        for (i, &x_val) in x.iter().enumerate().skip(peak_idx + 1) {
            if x_val < right_min {
                right_min = x_val;
                right_reached_minimum = true;
            } else if right_reached_minimum && x_val > right_min {
                // Stop when we start rising again after finding a minimum
                break;
            }
            // Stop if we hit a higher peak
            if x[i] > peak_height {
                break;
            }
        }

        // Prominence is the height above the highest of the two minima
        let min_height = if left_min > right_min {
            left_min
        } else {
            right_min
        };
        let prominence = peak_height - min_height;

        prominences.push(prominence);
    }

    Ok(prominences)
}
