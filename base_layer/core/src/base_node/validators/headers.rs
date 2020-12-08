//  Copyright 2020, The Tari Project
//
//  Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
//  following conditions are met:
//
//  1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
//  disclaimer.
//
//  2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
//  following disclaimer in the documentation and/or other materials provided with the distribution.
//
//  3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
//  products derived from this software without specific prior written permission.
//
//  THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
//  INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
//  DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
//  SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
//  SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
//  WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
//  USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use crate::{
    blocks::BlockHeader,
    chain_storage::{
        BlockHeaderAccumulatedData,
        BlockHeaderAccumulatedDataBuilder,
        BlockchainBackend,
        BlockchainDatabase,
    },
    proof_of_work::{Difficulty,},
    validation::{
        helpers,
        helpers::check_header_timestamp_greater_than_median,
        HeaderValidation,
        ValidationError,
    },
};
use log::*;
use tari_crypto::tari_utilities::{hex::Hex, Hashable};
use crate::proof_of_work::randomx_factory::RandomXFactory;

const LOG_TARGET: &str = "c::bn::states::horizon_state_sync::headers";

pub struct HeaderValidator<B> {
    db: BlockchainDatabase<B>,
    randomx_factory: RandomXFactory
}

impl<B: BlockchainBackend> HeaderValidator<B> {
    pub fn new(db: BlockchainDatabase<B>, randomx_factory: RandomXFactory) -> Self {
        Self { db, randomx_factory }
    }
}

impl<B: BlockchainBackend> HeaderValidation for HeaderValidator<B> {
    fn validate(
        &self,
        header: &BlockHeader,
        previous_header: &BlockHeader,
        previous_data: &BlockHeaderAccumulatedData,
    ) -> Result<BlockHeaderAccumulatedDataBuilder, ValidationError>
    {
        let hash = header.hash();
        let header_id = format!("header #{} ({})", header.height, hash.to_hex());
        self.check_median_timestamp(header)?;
        trace!(
            target: LOG_TARGET,
            "BlockHeader validation: Median timestamp is ok for {} ",
            &header_id
        );
        let (achieved, target) = self.check_achieved_and_target_difficulty(header)?;
        let accum_data = BlockHeaderAccumulatedDataBuilder::default()
            .hash(hash)
            .target_difficulty(target)
            .achieved_difficulty(previous_data, header.pow_algo(), achieved);
        debug!(
            target: LOG_TARGET,
            "Block header validation: BlockHeader is VALID for {}", &header_id
        );
        Ok(accum_data)
    }
}

impl<B: BlockchainBackend> HeaderValidator<B> {
    /// Calculates the achieved and target difficulties at the specified height and compares them.
    pub fn check_achieved_and_target_difficulty(
        &self,
        block_header: &BlockHeader,
    ) -> Result<(Difficulty, Difficulty), ValidationError>
    {
        let difficulty_window = self
            .db
            .fetch_target_difficulty(block_header.pow_algo(), block_header.height)?;

        let target = difficulty_window.calculate();
        Ok((
            helpers::check_target_difficulty(block_header, target, &self.randomx_factory)?,
            target,
        ))
    }

    /// This function tests that the block timestamp is greater than the median timestamp at the specified height.
    pub fn check_median_timestamp(&self, block_header: &BlockHeader) -> Result<(), ValidationError> {
        let timestamps = self.db.fetch_block_timestamps(block_header.hash())?;
        check_header_timestamp_greater_than_median(block_header, &timestamps)
    }
}
