// Copyright 2016-2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// https://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! The binomial distribution.

use Rng;
use distributions::Distribution;
use distributions::log_gamma::log_gamma;
use std::f64::consts::PI;

/// The binomial distribution `Binomial(n, p)`.
///
/// This distribution has density function:
/// `f(k) = n!/(k! (n-k)!) p^k (1-p)^(n-k)` for `k >= 0`.
///
/// # Example
///
/// ```rust
/// use rand::distributions::{Binomial, Distribution};
///
/// let bin = Binomial::new(20, 0.3);
/// let v = bin.sample(&mut rand::thread_rng());
/// println!("{} is from a binomial distribution", v);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Binomial {
    n: u64, // number of trials
    p: f64, // probability of success
}

impl Binomial {
    /// Construct a new `Binomial` with the given shape parameters
    /// `n`, `p`. Panics if `p <= 0` or `p >= 1`.
    pub fn new(n: u64, p: f64) -> Binomial {
        assert!(p > 0.0, "Binomial::new called with p <= 0");
        assert!(p < 1.0, "Binomial::new called with p >= 1");
        Binomial { n: n, p: p }
    }
}

impl Distribution<u64> for Binomial {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> u64 {
        // binomial distribution is symmetrical with respect to p -> 1-p, k -> n-k
        // switch p so that it is less than 0.5 - this allows for lower expected values
        // we will just invert the result at the end
        let p = if self.p <= 0.5 {
            self.p
        } else {
            1.0 - self.p
        };

        // expected value of the sample
        let expected = self.n as f64 * p;

        let result =
            // for low expected values we just simulate n drawings
            if expected < 25.0 {
                let mut lresult = 0.0;
                for _ in 0 .. self.n {
                    if rng.gen_bool(p) {
                        lresult += 1.0;
                    }
                }
                lresult
            }
            // high expected value - do the rejection method
            else {
                // prepare some cached values
                let float_n = self.n as f64;
                let ln_fact_n = log_gamma(float_n + 1.0);
                let pc = 1.0 - p;
                let log_p = p.ln();
                let log_pc = pc.ln();
                let sq = (expected * (2.0 * pc)).sqrt();

                let mut lresult;

                loop {
                    let mut comp_dev: f64;
                    // we use the lorentzian distribution as the comparison distribution
                    // f(x) ~ 1/(1+x/^2)
                    loop {
                        // draw from the lorentzian distribution
                        comp_dev = (PI*rng.gen::<f64>()).tan();
                        // shift the peak of the comparison ditribution
                        lresult = expected + sq * comp_dev;
                        // repeat the drawing until we are in the range of possible values
                        if lresult >= 0.0 && lresult < float_n + 1.0 {
                            break;
                        }
                    }

                    // the result should be discrete
                    lresult = lresult.floor();

                    let log_binomial_dist = ln_fact_n - log_gamma(lresult+1.0) -
                        log_gamma(float_n - lresult + 1.0) + lresult*log_p + (float_n - lresult)*log_pc;
                    // this is the binomial probability divided by the comparison probability
                    // we will generate a uniform random value and if it is larger than this,
                    // we interpret it as a value falling out of the distribution and repeat
                    let comparison_coeff = (log_binomial_dist.exp() * sq) * (1.2 * (1.0 + comp_dev*comp_dev));

                    if comparison_coeff >= rng.gen() {
                        break;
                    }
                }

                lresult
            };

        // invert the result for p < 0.5
        if p != self.p {
            self.n - result as u64
        } else {
            result as u64
        }
    }
}

#[cfg(test)]
mod test {
    use distributions::Distribution;
    use super::Binomial;

    fn test_binomial_mean_and_variance(n: u64, p: f64) {
        let binomial = Binomial::new(n, p);
        let mut rng = ::test::rng(123);

        let expected_mean = n as f64 * p;
        let expected_variance = n as f64 * p * (1.0 - p);

        let mut results = [0.0; 1000];
        for i in results.iter_mut() { *i = binomial.sample(&mut rng) as f64; }

        let mean = results.iter().sum::<f64>() / results.len() as f64;
        assert!((mean as f64 - expected_mean).abs() < expected_mean / 50.0);

        let variance =
            results.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>()
            / (results.len() - 1) as f64;
        assert!((variance - expected_variance).abs() < expected_variance / 10.0);
    }

    #[test]
    fn test_binomial() {
        test_binomial_mean_and_variance(150, 0.1);
        test_binomial_mean_and_variance(70, 0.6);
        test_binomial_mean_and_variance(40, 0.5);
        test_binomial_mean_and_variance(20, 0.7);
        test_binomial_mean_and_variance(20, 0.5);
    }

    #[test]
    #[should_panic]
    fn test_binomial_invalid_lambda_zero() {
        Binomial::new(20, 0.0);
    }

    #[test]
    #[should_panic]
    fn test_binomial_invalid_lambda_neg() {
        Binomial::new(20, -10.0);
    }
}
