#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderMode {
    Classic,
    FlowField,
    InstitutionGraph,
    AccountLedger,
    HistoryStability,
    DoubleExposure, // Classic + FlowField overlay
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_mode_variants_match_expected_order() {
        assert_eq!(RenderMode::Classic as usize, 0);
        assert_eq!(RenderMode::FlowField as usize, 1);
        assert_eq!(RenderMode::InstitutionGraph as usize, 2);
        assert_eq!(RenderMode::AccountLedger as usize, 3);
        assert_eq!(RenderMode::HistoryStability as usize, 4);
        assert_eq!(RenderMode::DoubleExposure as usize, 5);
    }
}
