use anchor_lang::prelude::*;
use anchor_lang::{
    solana_program::sysvar::instructions::{
        load_instruction_at_checked, ID as INSTRUCTIONS_SYSVAR_ID,
    },
    Discriminator,
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

declare_id!("22222222222222222222222222222222222222222222");

#[program]
pub mod flash_loans {
    use core::borrow;

    use super::*;

    pub fn borrow(ctx: Context<Loan>, borrow_amount: u64) -> Result<()> {
        require!(borrow_amount > 0, ProtocolError::InvalidAmount);

        // signers seeds for the protocol account
        let seeds = &[b"protocol".as_ref(), &[ctx.bumps.protocol]];

        let signer_seeds = &[&seeds[..]];

        // transfer funds from protocol to borrower
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.protocol_ata.to_account_info(),
                    to: ctx.accounts.borrower_ata.to_account_info(),
                    authority: ctx.accounts.protocol.to_account_info(),
                },
                signer_seeds,
            ),
            borrow_amount,
        )?;

        let ixs = ctx.accounts.instructions.to_account_info();
        // Check if this is the first instruction in the transaction.
        let current_index = load_current_index_checked(&ctx.accounts.instructions)?;
        require_eq!(current_index, 0, ProtocolError::InvalidIx);

        // Check how many instruction we have in this transaction
        let instruction_sysvar = ixs.try_borrow_data()?;
        if let Ok(repay_ix) = load_instruction_at_checked(len as usize - 1, &ixs) {
            // Instruction checks
            require_keys_eq!(repay_ix.program_id, ID, ProtocolError::InvalidProgram);
            require!(
                repay_ix.data[0..8].eq(instruction::Repay::DISCRIMINATOR),
                ProtocolError::InvalidIx
            );
            // We could check the Wallet and Mint separately but by checking the ATA we do this automatically
            require_keys_eq!(
                repay_ix
                    .accounts
                    .get(3)
                    .ok_or(ProtocolError::InvalidBorrowerAta)?
                    .pubkey,
                ctx.accounts.borrower_ata.key(),
                ProtocolError::InvalidBorrowerAta
            );
            require_keys_eq!(
                repay_ix
                    .accounts
                    .get(4)
                    .ok_or(ProtocolError::InvalidProtocolAta)?
                    .pubkey,
                ctx.accounts.protocol_ata.key(),
                ProtocolError::InvalidProtocolAta
            );
        } else {
            return Err(ProtocolError::MissingRepayIx.into());
        }
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn Repay(ctx: Context<Loan>) -> Result<()> {
        let ixs = ctx.accounts.instructions.to_account_info();
        let mut amount_borrowed: u64;

        if let Ok(borrow_ix) = load_instruction_at_checked(0, &ixs) {
            /// check borrowd amount
                let mut borrowed_data: [u8;8] = [0u8;8];
                borrowed_data.copy_from_slice(&borrow_ix.data[8..16]);
                amount_borrowed = u64::from_le_bytes(borrowed_data);

        } else {
            return Err(ProtocolError::MissingBorrowIx.into());
        }
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Loan<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,

    // Program Derived Address (PDA) that owns the protocol's liquidity pool.
    #[account(
        seeds = [b"protocol".as_ref()],
        bump,
    )]
    pub protocol: SystemAccount<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = borrower,
        associated_token::mint = mint,
        associated_token::authority = borrower,
    )]
    pub borrower_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = protocol,
    )]
    pub protocol_ata: Account<'info, TokenAccount>,

    #[account(address = INSTRUCTIONS_SYSVAR_ID)]
    /// CHECK: InstructionsSysvar account
    instructions: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[error_code]
pub enum ProtocolError {
    // error enum
    #[msg("Invalid instruction")]
    InvalidIx,
    #[msg("Invalid instruction index")]
    InvalidInstructionIndex,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Not enough funds")]
    NotEnoughFunds,
    #[msg("Program Mismatch")]
    ProgramMismatch,
    #[msg("Invalid program")]
    InvalidProgram,
    #[msg("Invalid borrower ATA")]
    InvalidBorrowerAta,
    #[msg("Invalid protocol ATA")]
    InvalidProtocolAta,
    #[msg("Missing repay instruction")]
    MissingRepayIx,
    #[msg("Missing borrow instruction")]
    MissingBorrowIx,
    #[msg("Overflow")]
    Overflow,
}
