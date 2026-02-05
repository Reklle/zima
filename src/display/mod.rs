// sample/src/hypothesis/dagostino.rs
use std::fmt::{self, Display, Formatter};
use num_traits::{Float, ToPrimitive, FromPrimitive};
use comfy_table::*;
use comfy_table::presets::UTF8_FULL;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use crate::hypothesis::DagostinoPearsonResult;

impl<F> DagostinoPearsonResult<F>
where
    F: Float + Display + ToPrimitive + FromPrimitive,
{
    pub fn display(&self) -> String {
            let c = |x: f64| F::from_f64(x).expect("Failed to convert constant to F");

            let p_0001 = c(0.0001);
            let p_05 = c(0.05);
            let p_10 = c(0.10);

            let skew_0_5 = c(0.5);
            let skew_1_0 = c(1.0);
            let skew_2_0 = c(2.0);

            let kurt_0_5 = c(0.5);
            let kurt_2_0 = c(2.0);
            let kurt_5_0 = c(5.0);

            let stat_999 = c(999.0);
            let zero = F::zero();

            // --- Format values ---
            let p_display = if self.p_value < p_0001 {
                "< 0.0001".to_string()
            } else {
                format!("{:.4}", self.p_value)
            };

            let stat_display = if self.statistic > stat_999 {
                format!("{:.1e}", self.statistic.to_f64().unwrap_or(0.0))
            } else {
                format!("{:.2}", self.statistic)
            };

            let skew_display = format!("{:+.2}", self.skewness);
            let kurt_display = format!("{:.2}", self.kurtosis);

            // --- Interpretations (all in F) ---
            let p_interpretation = if self.p_value < p_05 {
                "🔴 Reject normality"
            } else if self.p_value < p_10 {
                "🟠 Weak evidence against normality"
            } else {
                "🟢 Cannot reject normality"
            };

            let skew_abs = self.skewness.abs();
            let skew_interpretation = if skew_abs < skew_0_5 {
                "🟢 Approx. symmetric"
            } else if skew_abs < skew_1_0 {
                if self.skewness > zero { "🟡 Moderate right skew" } else { "🟡 Moderate left skew" }
            } else if skew_abs < skew_2_0 {
                if self.skewness > zero { "🟠 Strong right skew" } else { "🟠 Strong left skew" }
            } else {
                if self.skewness > zero { "🔴 Extreme right skew" } else { "🔴 Extreme left skew" }
            };

            let kurt_interpretation = if self.kurtosis.abs() < kurt_0_5 {
                "🟢 Mesokurtic (normal tails)"
            } else if self.kurtosis > kurt_0_5 && self.kurtosis < kurt_2_0 {
                "🟡 Leptokurtic (heavy tails)"
            } else if self.kurtosis >= kurt_2_0 && self.kurtosis < kurt_5_0 {
                "🟠 Very heavy tails"
            } else if self.kurtosis >= kurt_5_0 {
                "🔴 Extreme heavy tails"
            } else {
                "🔵 Platykurtic (light tails)"
            };

            let mut title_table = Table::new();
                    title_table
                        .load_preset(UTF8_FULL)
                        .apply_modifier(UTF8_ROUND_CORNERS)
                        .set_content_arrangement(ContentArrangement::Dynamic)
                        .add_row(vec![Cell::new("D'Agostino-Pearson Omnibus Normality Test")
                            .set_alignment(CellAlignment::Center)]);

            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Metric").set_alignment(CellAlignment::Center),
                    Cell::new("Value").set_alignment(CellAlignment::Center),
                    Cell::new("Interpretation").set_alignment(CellAlignment::Center),
                ]);


            // Test statistics
            table
                .add_row(vec![
                    Cell::new("p-value").set_alignment(CellAlignment::Left),
                    Cell::new(&p_display).set_alignment(CellAlignment::Right),
                    Cell::new(p_interpretation).set_alignment(CellAlignment::Left),
                ])
                .add_row(vec![
                    Cell::new("Statistic").set_alignment(CellAlignment::Left),
                    Cell::new(&stat_display).set_alignment(CellAlignment::Right),
                    Cell::new("~ χ²(2)").set_alignment(CellAlignment::Left),
                ]);

            // Moments
            table
                .add_row(vec![
                    Cell::new("Skewness").set_alignment(CellAlignment::Left),
                    Cell::new(&skew_display).set_alignment(CellAlignment::Right),
                    Cell::new(skew_interpretation).set_alignment(CellAlignment::Left),
                ])
                .add_row(vec![
                    Cell::new("Kurtosis").set_alignment(CellAlignment::Left),
                    Cell::new(&kurt_display).set_alignment(CellAlignment::Right),
                    Cell::new(kurt_interpretation).set_alignment(CellAlignment::Left),
                ]);

            format!("{}\n{}",title_table, table)
        }
}

impl<F> Display for DagostinoPearsonResult<F>
where
    F: Float + Display + ToPrimitive + FromPrimitive,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}
