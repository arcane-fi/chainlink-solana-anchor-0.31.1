//! Chainlink feed client for Solana - Anchor v0.31.1 Compatible Fork
#![deny(rustdoc::all)]
#![allow(rustdoc::missing_doc_code_examples)]
#![deny(missing_docs)]

extern crate borsh;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    instruction::{AccountMeta, Instruction},
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::result::Result;

// The library uses this to verify the keys
solana_program::declare_id!("HEvSKofvBgfaexv23kMabbYqxasxU3mQ4ibBMEmJWHny");

#[derive(BorshSerialize, BorshDeserialize)]
enum Query {
    Version,
    Decimals,
    Description,
    RoundData { round_id: u32 },
    LatestRoundData,
    Aggregator,
}

/// Represents a single oracle round.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Round {
    /// The round id.
    pub round_id: u32,
    /// Slot at the time the report was received on chain.
    pub slot: u64,
    /// Round timestamp, as reported by the oracle.
    pub timestamp: u32,
    /// Current answer, formatted to `decimals` decimal places.
    pub answer: i128,
}

fn query<'info, T: BorshDeserialize>(
    program_id: AccountInfo<'info>,
    feed: AccountInfo<'info>,
    scope: Query,
) -> Result<T, ProgramError> {
    // Import std::io types explicitly to avoid conflicts with borsh::io
    use std::io::{Cursor, Write};
    
    const QUERY_INSTRUCTION_DISCRIMINATOR: &[u8] =
        &[0x27, 0xfb, 0x82, 0x9f, 0x2e, 0x88, 0xa4, 0xa9];
    
    // Avoid array resizes by using the maximum response size as the initial capacity.
    const MAX_SIZE: usize = QUERY_INSTRUCTION_DISCRIMINATOR.len() + std::mem::size_of::<Pubkey>();
    let mut data = Cursor::new(Vec::with_capacity(MAX_SIZE));
    data.write_all(QUERY_INSTRUCTION_DISCRIMINATOR)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    scope.serialize(&mut data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    let ix = Instruction {
        program_id: *program_id.key,
        accounts: vec![AccountMeta::new_readonly(*feed.key, false)],
        data: data.into_inner(),
    };

    invoke(&ix, &[feed.clone()])?;

    let (_key, data) =
        solana_program::program::get_return_data().expect("chainlink store had no return_data!");
    let data = T::try_from_slice(&data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    Ok(data)
}

/// Query the feed version.
pub fn version<'info>(
    program_id: AccountInfo<'info>,
    feed: AccountInfo<'info>,
) -> Result<u8, ProgramError> {
    query(program_id, feed, Query::Version)
}

/// Returns the amount of decimal places.
pub fn decimals<'info>(
    program_id: AccountInfo<'info>,
    feed: AccountInfo<'info>,
) -> Result<u8, ProgramError> {
    query(program_id, feed, Query::Decimals)
}

/// Returns the feed description.
pub fn description<'info>(
    program_id: AccountInfo<'info>,
    feed: AccountInfo<'info>,
) -> Result<String, ProgramError> {
    query(program_id, feed, Query::Description)
}

/// Returns round data for the latest round.
pub fn latest_round_data<'info>(
    program_id: AccountInfo<'info>,
    feed: AccountInfo<'info>,
) -> Result<Round, ProgramError> {
    query(program_id, feed, Query::LatestRoundData)
}

/// Returns the address of the underlying OCR2 aggregator.
pub fn aggregator<'info>(
    program_id: AccountInfo<'info>,
    feed: AccountInfo<'info>,
) -> Result<Pubkey, ProgramError> {
    query(program_id, feed, Query::Aggregator)
}

/// Convert Anchor AccountInfo to Solana AccountInfo for library compatibility
pub fn anchor_to_solana_account_info<'a, 'info: 'a>(
    anchor_info: &'a anchor_lang::prelude::AccountInfo<'info>
) -> solana_program::account_info::AccountInfo<'info> {
    solana_program::account_info::AccountInfo {
        key: anchor_info.key,
        is_signer: anchor_info.is_signer,
        is_writable: anchor_info.is_writable,
        lamports: anchor_info.lamports.clone(),
        data: anchor_info.data.clone(),
        owner: anchor_info.owner,
        executable: anchor_info.executable,
        rent_epoch: anchor_info.rent_epoch,
    }
}

/// Anchor-compatible wrapper for latest_round_data
pub fn latest_round_data_anchor<'info>(
    program_id: &anchor_lang::prelude::AccountInfo<'info>,
    feed: &anchor_lang::prelude::AccountInfo<'info>,
) -> anchor_lang::prelude::Result<Round> {
    let program_info = anchor_to_solana_account_info(program_id);
    let feed_info = anchor_to_solana_account_info(feed);
    
    latest_round_data(program_info, feed_info)
        .map_err(|_| anchor_lang::error::Error::from(anchor_lang::error::ErrorCode::AccountNotEnoughKeys))
}

/// Anchor-compatible wrapper for decimals
pub fn decimals_anchor<'info>(
    program_id: &anchor_lang::prelude::AccountInfo<'info>,
    feed: &anchor_lang::prelude::AccountInfo<'info>,
) -> anchor_lang::prelude::Result<u8> {
    let program_info = anchor_to_solana_account_info(program_id);
    let feed_info = anchor_to_solana_account_info(feed);
    
    decimals(program_info, feed_info)
        .map_err(|_| anchor_lang::error::Error::from(anchor_lang::error::ErrorCode::AccountNotEnoughKeys))
}

/// Anchor-compatible wrapper for description
pub fn description_anchor<'info>(
    program_id: &anchor_lang::prelude::AccountInfo<'info>,
    feed: &anchor_lang::prelude::AccountInfo<'info>,
) -> anchor_lang::prelude::Result<String> {
    let program_info = anchor_to_solana_account_info(program_id);
    let feed_info = anchor_to_solana_account_info(feed);
    
    description(program_info, feed_info)
        .map_err(|_| anchor_lang::error::Error::from(anchor_lang::error::ErrorCode::AccountNotEnoughKeys))
}