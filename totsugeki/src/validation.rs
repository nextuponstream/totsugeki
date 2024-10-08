//! Validation

/// All validation types before automatically progressing a bracket
#[derive(Copy, Clone)]
pub enum AutomaticMatchValidationMode {
    /// No trust: only TO is allowed to report matches
    ///
    /// You should default to this mode for offline tournaments (especially when
    /// people travel for your tournament)
    ///
    /// Player reports will be discarded
    Strict,
    /// Some trust: if both player agree on result, then match will validate
    /// automatically.
    ///
    /// Usually for big online tournament where you don't want a bad (or
    /// malicious) report to grab tournament organiser's attention needlessly
    Flexible,
    /// Full trust: if any player reports, match is validated. This is useful
    /// for small tournaments (<30 participants) where good faith is assumed and
    /// the chance of a single bad report won't annoy the organisers too much
    Lax,
}

impl Default for AutomaticMatchValidationMode {
    /// Match result integrity is key. Therefore, the default is to let
    /// tournament organiser validate the outcome of each and every match
    fn default() -> Self {
        AutomaticMatchValidationMode::Strict
    }
}
