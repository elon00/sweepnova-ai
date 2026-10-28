use anchor_lang::prelude::*;

#[error_code]
pub enum SweepNovaError {
    #[msg("Deposit amount must be strictly greater than zero.")]
    ZeroDeposit,
    #[msg("Deposit duration must be at least 24 hours (86,400 seconds).")]
    InvalidDuration,
    #[msg("Sweep mandate is currently inactive or revoked.")]
    MandateInactive,
    #[msg("Sweep mandate has passed its expiration timestamp.")]
    MandateExpired,
    #[msg("Mandate minimum sweep period has not elapsed yet.")]
    SweepPeriodNotElapsed,
    #[msg("Requested sweep amount exceeds per-period mandate cap.")]
    SweepAmountExceedsMandateCap,
}
