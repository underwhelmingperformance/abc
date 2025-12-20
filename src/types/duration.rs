//! Musical duration representation.
//!
//! This module provides the `Duration` type for representing musical durations
//! as rational numbers to avoid floating-point precision errors.

/// A musical duration represented as a rational number.
///
/// Durations are stored as numerator/denominator pairs to avoid floating-point
/// precision errors. For example, a quarter note is represented as 1/4, and
/// an eighth note as 1/8.
///
/// # Examples
///
/// ```
/// # use abc::types::Duration;
/// let quarter_note = Duration::new(1, 4);
/// let eighth_note = Duration::new(1, 8);
/// assert!(quarter_note > eighth_note);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Duration {
    numerator: u32,
    denominator: u32,
}

impl Duration {
    /// A zero duration (0/1).
    ///
    /// Used for events that mark a point in time without consuming duration,
    /// such as guitar chord symbols and annotations.
    pub const ZERO: Self = Self {
        numerator: 0,
        denominator: 1,
    };

    /// Creates a new duration from a numerator and denominator.
    ///
    /// # Panics
    ///
    /// Panics if `denominator` is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// # use abc::types::Duration;
    /// let quarter = Duration::new(1, 4);
    /// let dotted_quarter = Duration::new(3, 8);
    /// ```
    #[must_use]
    pub const fn new(numerator: u32, denominator: u32) -> Self {
        assert!(denominator > 0, "Duration denominator must be non-zero");
        Self {
            numerator,
            denominator,
        }
    }

    /// Returns the numerator of this duration.
    #[must_use]
    pub const fn numerator(&self) -> u32 {
        self.numerator
    }

    /// Returns the denominator of this duration.
    #[must_use]
    pub const fn denominator(&self) -> u32 {
        self.denominator
    }

    /// Normalises this duration by reducing it to its simplest form.
    ///
    /// This divides both numerator and denominator by their greatest common divisor.
    ///
    /// # Examples
    ///
    /// ```
    /// # use abc::types::Duration;
    /// let duration = Duration::new(8, 16);
    /// let normalised = duration.normalise();
    /// assert_eq!(normalised, Duration::new(1, 2));
    /// ```
    #[must_use]
    pub fn normalise(self) -> Self {
        let gcd = gcd(self.numerator, self.denominator);
        Self {
            numerator: self.numerator / gcd,
            denominator: self.denominator / gcd,
        }
    }
}

impl Default for Duration {
    fn default() -> Self {
        Self::ZERO
    }
}

/// Computes the greatest common divisor using Euclid's algorithm.
const fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

impl std::ops::Mul<u32> for &Duration {
    type Output = Duration;

    fn mul(self, factor: u32) -> Duration {
        Duration {
            numerator: self.numerator * factor,
            denominator: self.denominator,
        }
    }
}

impl std::ops::Div<u32> for &Duration {
    type Output = Duration;

    fn div(self, divisor: u32) -> Duration {
        assert!(divisor > 0, "Duration divisor must be non-zero");
        Duration {
            numerator: self.numerator,
            denominator: self.denominator * divisor,
        }
    }
}

impl std::ops::Add for &Duration {
    type Output = Duration;

    fn add(self, other: Self) -> Duration {
        // a/b + c/d = (ad + bc) / bd
        Duration {
            numerator: self.numerator * other.denominator + other.numerator * self.denominator,
            denominator: self.denominator * other.denominator,
        }
        .normalise()
    }
}

impl PartialOrd for Duration {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Duration {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare by cross-multiplication to avoid overflow
        (self.numerator as u64 * other.denominator as u64)
            .cmp(&(other.numerator as u64 * self.denominator as u64))
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(1, 4, 2, 2, 4)]
    #[case(1, 8, 3, 3, 8)]
    #[case(1, 4, 4, 4, 4)]
    fn test_duration_multiply(
        #[case] num: u32,
        #[case] denom: u32,
        #[case] factor: u32,
        #[case] expected_num: u32,
        #[case] expected_denom: u32,
    ) {
        let duration = Duration::new(num, denom);
        let result = &duration * factor;
        assert_eq!(result, Duration::new(expected_num, expected_denom));
    }

    #[rstest]
    #[case(1, 4, 2, 1, 8)]
    #[case(1, 2, 4, 1, 8)]
    #[case(3, 4, 2, 3, 8)]
    fn test_duration_divide(
        #[case] num: u32,
        #[case] denom: u32,
        #[case] divisor: u32,
        #[case] expected_num: u32,
        #[case] expected_denom: u32,
    ) {
        let duration = Duration::new(num, denom);
        let result = &duration / divisor;
        assert_eq!(result, Duration::new(expected_num, expected_denom));
    }

    #[rstest]
    #[case(1, 4, 1, 4, 1, 2)] // 1/4 + 1/4 = 1/2 (normalised)
    #[case(1, 4, 1, 8, 3, 8)] // 1/4 + 1/8 = 3/8 (normalised)
    #[case(1, 2, 1, 4, 3, 4)] // 1/2 + 1/4 = 3/4 (normalised)
    fn test_duration_add(
        #[case] num1: u32,
        #[case] denom1: u32,
        #[case] num2: u32,
        #[case] denom2: u32,
        #[case] expected_num: u32,
        #[case] expected_denom: u32,
    ) {
        let dur1 = Duration::new(num1, denom1);
        let dur2 = Duration::new(num2, denom2);
        let result = &dur1 + &dur2;
        assert_eq!(result, Duration::new(expected_num, expected_denom));
    }

    #[test]
    #[should_panic(expected = "Duration divisor must be non-zero")]
    fn test_duration_divide_by_zero() {
        let duration = Duration::new(1, 4);
        let _ = &duration / 0;
    }
}
