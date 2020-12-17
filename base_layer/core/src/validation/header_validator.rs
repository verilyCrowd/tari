use crate::validation::helpers::{check_timestamp_ftl, check_pow_data, check_header_timestamp_greater_than_median, check_target_difficulty};
use crate::blocks::BlockHeader;
use crate::validation::{ValidationError, HeaderValidation};
use log::*;
use crate::chain_storage::{BlockHeaderAccumulatedData, BlockHeaderAccumulatedDataBuilder, BlockchainBackend, fetch_headers};
use crate::proof_of_work::randomx_factory::RandomXFactory;
use crate::proof_of_work::Difficulty;

pub const LOG_TARGET: &str = "c::val::block_validators";

pub struct HeaderValidator {

    randomx_factory: RandomXFactory
}

impl HeaderValidator {
    /// Calculates the achieved and target difficulties at the specified height and compares them.
    pub fn check_achieved_and_target_difficulty<B:BlockchainBackend>(
        &self,
        db: &B,
        block_header: &BlockHeader,
    ) -> Result<(Difficulty, Difficulty), ValidationError>
    {
        let difficulty_window =
            db
            .fetch_target_difficulty(block_header.pow_algo(), block_header.height)?;

        let target = difficulty_window.calculate();
        Ok((
            check_target_difficulty(block_header, target, &self.randomx_factory)?,
            target,
        ))
    }

    /// This function tests that the block timestamp is greater than the median timestamp at the specified height.
    pub fn check_median_timestamp<B:BlockchainBackend>(&self, db: &BlockchainBackend, block_header: &BlockHeader) -> Result<(), ValidationError> {
        let timestamps = db.fetch_block_timestamps(block_header.hash())?;
        check_header_timestamp_greater_than_median(block_header, &timestamps)
    }
}

impl<B:BlockchainBackend> HeaderValidation<B> for HeaderValidator {
    /// The consensus checks that are done (in order of cheapest to verify to most expensive):
    /// 1. Is the block timestamp within the Future Time Limit (FTL)?
    /// 1. Is the Proof of Work valid?
    /// 1. Is the achieved difficulty of this block >= the target difficulty for this block?

    fn validate(&self, backend: &B, header: &BlockHeader, previous_header: &BlockHeader, previous_data: &BlockHeaderAccumulatedData) -> Result<BlockHeaderAccumulatedDataBuilder, ValidationError> {
        check_timestamp_ftl(&header, &self.rules)?;
        let hash = header.hash();
        let header_id = format!("header #{} ({})", header.height, header.hash().to_hex());
        trace!(
            target: LOG_TARGET,
            "BlockHeader validation: FTL timestamp is ok for {} ",
            header_id
        );
        self.check_median_timestamp(backend, header)?;
        trace!(
            target: LOG_TARGET,
            "BlockHeader validation: Median timestamp is ok for {} ",
            header_id
        );
        check_pow_data(header, &self.rules, backend)?;
        let (achieved, target) = self.check_achieved_and_target_difficulty(backend, header)?;
        let accum_data = BlockHeaderAccumulatedDataBuilder::default()
            .hash(hash)
            .target_difficulty(target)
            .achieved_difficulty(previous_data, header.pow_algo(), achieved);
        trace!(
            target: LOG_TARGET,
            "BlockHeader validation: Achieved difficulty is ok for {} ",
            header_id
        );
        debug!(
            target: LOG_TARGET,
            "Block header validation: BlockHeader is VALID for {}", header_id
        );
        Ok(accum_data)
    }
}
