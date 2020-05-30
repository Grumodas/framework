use crate::{
    config::*, consts, fixed_vector, helper_functions_types::Error as HelperError, primitives::*,
    types::*, beacon_chain_types::*
};
use ethereum_types::H256 as Hash256;
use serde::{Deserialize, Serialize};
use ssz_new_derive::{SszDecode, SszEncode};
use ssz_types::{BitVector, Error as SszError, FixedVector, VariableList};
use tree_hash::TreeHash;
use tree_hash_derive::TreeHash;

#[derive(Debug, PartialEq)]
pub enum Error {
    EpochOutOfBounds,
    SlotOutOfBounds,
    ShardOutOfBounds,
    UnknownValidator,
    UnableToDetermineProducer,
    InvalidBitfield,
    ValidatorIsWithdrawable,
    UnableToShuffle,
    TooManyValidators,
    InsufficientValidators,
    InsufficientRandaoMixes,
    InsufficientBlockRoots,
    InsufficientIndexRoots,
    InsufficientAttestations,
    InsufficientCommittees,
    InsufficientStateRoots,
    NoCommitteeForShard,
    NoCommitteeForSlot,
    ZeroSlotsPerEpoch,
    PubkeyCacheInconsistent,
    PubkeyCacheIncomplete {
        cache_len: usize,
        registry_len: usize,
    },
    PreviousCommitteeCacheUninitialized,
    CurrentCommitteeCacheUninitialized,
    //RelativeEpochError(RelativeEpochError),
    //CommitteeCacheUninitialized(RelativeEpoch),
    SszTypesError(SszError),
    HelperError(HelperError),
}

impl From<SszError> for Error {
    fn from(error: SszError) -> Self {
        Error::SszTypesError(error)
    }
}

impl From<HelperError> for Error {
    fn from(error: HelperError) -> Self {
        Error::HelperError(error)
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, SszDecode, SszEncode, TreeHash)]
pub struct BeaconState<C: Config> {
    // Versioning
    pub genesis_time: u64,
    pub genesis_validators_root: H256, // Wip: this wasn't here in phase 0 implementation, why?
    pub slot: Slot,
    pub fork: Fork,

    // History
    pub latest_block_header: BeaconBlockHeader,
    pub block_roots: FixedVector<H256, C::SlotsPerHistoricalRoot>,
    pub state_roots: FixedVector<H256, C::SlotsPerHistoricalRoot>,
    pub historical_roots: VariableList<H256, C::HistoricalRootsLimit>,

    // Eth1
    pub eth1_data: Eth1Data,
    pub eth1_data_votes: VariableList<Eth1Data, C::SlotsPerEth1VotingPeriod>,
    pub eth1_deposit_index: u64,

    // Registry
    pub validators: VariableList<Validator, C::ValidatorRegistryLimit>,
    pub balances: VariableList<u64, C::ValidatorRegistryLimit>,

    // Randomness
    pub randao_mixes: FixedVector<H256, C::EpochsPerHistoricalVector>,

    // Slashings
    pub slashings: FixedVector<u64, C::EpochsPerSlashingsVector>,

    // Attestations
    pub previous_epoch_attestations:
        VariableList<PendingAttestation<C>, C::MaxAttestationsPerEpoch>,
    pub current_epoch_attestations: VariableList<PendingAttestation<C>, C::MaxAttestationsPerEpoch>,

    // Finality
    pub justification_bits: BitVector<consts::JustificationBitsLength>,
    pub previous_justified_checkpoint: Checkpoint,
    pub current_justified_checkpoint: Checkpoint,
    pub finalized_checkpoint: Checkpoint,

    // Phase 1
    pub shard_states: VariableList<ShardState, C::MaxShards>,
    pub online_countdown: VariableList<u8, C::ValidatorRegistryLimit>,
    pub current_light_committee: CompactCommittee<C>,
    pub next_light_committee: CompactCommittee<C>,

    //Custody Game
    pub exposed_derived_secrets: FixedVector<VariableList<ValidatorIndex, C::SlotsPerEpoch>, C::EarlyDerivedSecretPenaltyMaxFutureEpochs>
}

impl<C: Config> Default for BeaconState<C> {
    fn default() -> Self {
        Self {
            block_roots: fixed_vector::default(),
            state_roots: fixed_vector::default(),
            randao_mixes: fixed_vector::default(),
            slashings: fixed_vector::default(),

            genesis_validators_root: Default::default(),
            genesis_time: Default::default(),
            slot: Default::default(),
            fork: Default::default(),
            latest_block_header: Default::default(),
            historical_roots: Default::default(),
            eth1_data: Default::default(),
            eth1_data_votes: Default::default(),
            eth1_deposit_index: Default::default(),
            validators: Default::default(),
            balances: Default::default(),
            previous_epoch_attestations: Default::default(),
            current_epoch_attestations: Default::default(),
            justification_bits: Default::default(),
            previous_justified_checkpoint: Default::default(),
            current_justified_checkpoint: Default::default(),
            finalized_checkpoint: Default::default(),
            exposed_derived_secrets: Default::default(),
            shard_states: Default::default(),
            online_countdown: Default::default(),
            current_light_committee: Default::default(),
            next_light_committee: Default::default(),
        }
    }
}
