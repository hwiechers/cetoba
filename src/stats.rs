use std;

use special::Gamma;

// The Dirichlet PDF with 3 categories.
// Only p1 and p2 are required as p3 = 1. - p1 - p2.
pub fn dirichlet_pdf(alpha: &[f64; 3], p1: f64, p2: f64) -> f64 {
    if p1 < 0. {
        panic!("p1 has a negative value")
    }
    if p2 < 0. {
        panic!("p2 has a negative value")
    }
    let p3 = 1. - p1 - p2;

    let sum_alpha: f64 = alpha.iter().sum();
    let alpha1 = alpha[0];
    let alpha2 = alpha[1];
    let alpha3 = alpha[2];

    sum_alpha.gamma() * (p1.powf(alpha1 - 1.) / alpha1.gamma())
        * (p2.powf(alpha2 - 1.) / alpha2.gamma()) * (p3.powf(alpha3 - 1.) / alpha3.gamma())
}

// This function uses the fixed-point method for estimating the parameters of
// The Dirichlet-multinomial/Polya distribution from
// "Estimating a Dirichlet distibution" by Thomas P. Minka.
// See https://tminka.github.io/papers/dirichlet/minka-dirichlet.pdf.
// The input samples must be in the order [white_win_count, draw_count, black_win_count] and the
// output is in the form [alpha_white, alpha_draw, alpha_black].
pub fn fit_polya(samples: &[[u32; 3]]) -> [f64; 3] {
    let mut alpha: [f64; 3] = [10., 10., 10.];

    loop {
        let old_alpha = alpha;
        let alpha_sum = alpha.iter().sum::<f64>();

        let denominator: f64 = samples
            .iter()
            .map(|s| (f64::from(s.iter().sum::<u32>()) + alpha_sum).digamma() - alpha_sum.digamma())
            .sum::<f64>();

        for index in 0..3 {
            let term1 = samples
                .iter()
                .map(|s| (f64::from(s[index]) + alpha[index]).digamma() - alpha[index].digamma())
                .sum::<f64>();
            let numerator = alpha[index] * term1;
            alpha[index] = numerator / denominator;
        }

        let dist2 = old_alpha
            .iter()
            .zip(alpha.iter())
            .map(|(&left, &right)| (left - right) * (left - right))
            .sum::<f64>();

        if dist2 < std::f64::EPSILON {
            break;
        }
    }

    alpha
}
