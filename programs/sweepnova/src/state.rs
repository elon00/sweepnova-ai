use anchor_lang::prelude::*;

#[account]
pub struct ProtocolConfig {
    pub admin: Pubkey,
    pub emergency_multisig: Pubkey,
    pub fee_treasury: Pubkey,
    pub is_paused: bool,
    pub protocol_fee_bps: u16,
    pub bump: u8,
}

impl ProtocolConfig {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 1 + 2 + 1;
}

#[account]
pub struct UserVault {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub cumulative_deposited: u64,
    pub active_positions_count: u32,
    pub bump: u8,
}

impl UserVault {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 4 + 1;
}

#[account]
pub struct DepositPosition {
    pub position_id: u64,
    pub user_vault: Pubkey,
    pub principal_amount: u64,
    pub interest_rate_bps: u32,
    pub creation_timestamp: i64,
    pub maturity_timestamp: i64,
    pub status: PositionStatus,
    pub withdrawn_amount: u64,
    pub bump: u8,
}

impl DepositPosition {
    pub const LEN: usize = 8 + 8 + 32 + 8 + 4 + 8 + 8 + 1 + 8 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum PositionStatus {
    Active,
    Matured,
    LiquidatedEarly,
    Closed,
}

#[account]
pub struct SweepMandate {
    pub mandate_id: u64,
    pub user: Pubkey,
    pub source_token_account: Pubkey,
    pub destination_vault: Pubkey,
    pub max_sweep_amount_per_period: u64,
    pub sweep_period_seconds: i64,
    pub last_sweep_timestamp: i64,
    pub total_swept: u64,
    pub expiration_timestamp: i64,
    pub is_active: bool,
    pub bump: u8,
}
