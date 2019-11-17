use std::collections::HashSet;
use std::iter::FromIterator;
use crate::error::Error;
use typenum::marker_traits::Unsigned;
use types::{
    config::Config,
    primitives::Epoch,
    types::{AttestationData, IndexedAttestation, Validator},
    beacon_state::BeaconState,
};

pub fn is_slashable_validator(validator: &Validator, epoch: Epoch) -> bool {
    !validator.slashed
        && validator.activation_epoch <= epoch
        && epoch < validator.withdrawable_epoch
}

pub fn is_active_validator(validator: &Validator, epoch: Epoch) -> bool {
    validator.activation_epoch <= epoch && epoch < validator.exit_epoch
}

pub fn is_slashable_attestation_data(data_1: &AttestationData, data_2: &AttestationData) -> bool {
    // Double vote
    (data_1 != data_2 && data_1.target.epoch == data_2.target.epoch) ||
    // Surround vote
    (data_1.source.epoch < data_2.source.epoch && data_2.target.epoch < data_1.target.epoch)
}

pub fn is_valid_indexed_attestation<C: Config>(
    _state: &BeaconState<C>,
    indexed_attestation: &IndexedAttestation<C>,
) -> Result<(), Error> {
    let bit_0_indices = &indexed_attestation.custody_bit_0_indices;
    let bit_1_indices = &indexed_attestation.custody_bit_1_indices;

    // Verify no index has custody bit equal to 1 [to be removed in phase 1]
    // if bit_1_indices.len() != 0 {
    //     return Err(Error::CustodyBitSet);
    // }

    // Verify max number of indices
    if (bit_0_indices.len() + bit_1_indices.len()) > C::MaxValidatorsPerCommittee::to_usize() {
        return Err(Error::MaxIndicesExceeded);
    }

    // Verify index sets are disjoint
    let is_disjoint = HashSet::<&u64>::from_iter(bit_0_indices.iter()).is_disjoint(&HashSet::<&u64>::from_iter(bit_1_indices.iter()));
    if !is_disjoint {
        return Err(Error::CustodyBitValidatorsIntersect);
    }

    // Verify indices are sorted
    let is_sorted = bit_0_indices.windows(2).all(|w| w[0] <= w[1]) && bit_1_indices.windows(2).all(|w| w[0] <= w[1]);
    if !is_sorted {
		return Err(Error::BadValidatorIndicesOrdering);
	}

    //     # Verify aggregate signature
    //     if not bls_verify_multiple(
    //         pubkeys=[
    //             bls_aggregate_pubkeys([state.validators[i].pubkey for i in bit_0_indices]),
    //             bls_aggregate_pubkeys([state.validators[i].pubkey for i in bit_1_indices]),
    //         ],
    //         message_hashes=[
    //             hash_tree_root(AttestationDataAndCustodyBit(data=indexed_attestation.data, custody_bit=0b0)),
    //             hash_tree_root(AttestationDataAndCustodyBit(data=indexed_attestation.data, custody_bit=0b1)),
    //         ],
    //         signature=indexed_attestation.signature,
    //         domain=get_domain(state, DOMAIN_BEACON_ATTESTER, indexed_attestation.data.target.epoch),
    //     ):
    //         return False

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::config::MainnetConfig;
    use types::primitives::*;
    use types::types::Checkpoint;
    use ssz_types::{VariableList};

    #[test]
    fn test_is_slashable_validator() {
        let v = Validator {
            slashed: false,
            activation_epoch: 0,
            withdrawable_epoch: 1,
            ..Validator::default()
        };
        assert_eq!(is_slashable_validator(&v, 0), true);
    }

    #[test]
    fn test_is_slashable_validator_already_slashed() {
        let v = Validator {
            slashed: true,
            activation_epoch: 0,
            withdrawable_epoch: 1,
            ..Validator::default()
        };
        assert_eq!(is_slashable_validator(&v, 0), false);
    }

    #[test]
    fn test_is_slashable_validator_activation_epoch_greater_than_epoch() {
        let v = Validator {
            slashed: false,
            activation_epoch: 1,
            withdrawable_epoch: 2,
            ..Validator::default()
        };
        assert_eq!(is_slashable_validator(&v, 0), false);
    }

    #[test]
    fn test_is_slashable_validator_withdrawable_epoch_equals_epoch() {
        let v = Validator {
            slashed: false,
            activation_epoch: 0,
            withdrawable_epoch: 1,
            ..Validator::default()
        };
        assert_eq!(is_slashable_validator(&v, 1), false);
    }

    #[test]
    fn test_is_active_validator() {
        let v = Validator {
            activation_epoch: 0,
            exit_epoch: 1,
            ..Validator::default()
        };
        assert_eq!(is_active_validator(&v, 0), true);
    }

    #[test]
    fn test_is_active_validator_activation_epoch_greater_than_epoch() {
        let v = Validator {
            activation_epoch: 1,
            exit_epoch: 2,
            ..Validator::default()
        };
        assert_eq!(is_active_validator(&v, 0), false);
    }

    #[test]
    fn test_is_active_validator_exit_epoch_equals_epoch() {
        let v = Validator {
            activation_epoch: 0,
            exit_epoch: 1,
            ..Validator::default()
        };
        assert_eq!(is_active_validator(&v, 1), false);
    }

    #[test]
    fn test_is_slashable_attestation_data_double_vote_false() {
        let attestation_data_1 = AttestationData {
            target: Checkpoint {
                epoch: 1,
                root: H256::from([0; 32]),
            },
            ..AttestationData::default()
        };
        let attestation_data_2 = AttestationData {
            target: Checkpoint {
                epoch: 1,
                root: H256::from([0; 32]),
            },
            ..AttestationData::default()
        };
        assert_eq!(
            is_slashable_attestation_data(&attestation_data_1, &attestation_data_2),
            false
        );
    }

    #[test]
    fn test_is_slashable_attestation_data_double_vote_true() {
        let attestation_data_1 = AttestationData {
            target: Checkpoint {
                epoch: 1,
                root: H256::from([0; 32]),
            },
            ..AttestationData::default()
        };
        let attestation_data_2 = AttestationData {
            target: Checkpoint {
                epoch: 1,
                root: H256::from([1; 32]),
            },
            ..AttestationData::default()
        };
        assert_eq!(
            is_slashable_attestation_data(&attestation_data_1, &attestation_data_2),
            true
        );
    }

    #[test]
    fn test_is_slashable_attestation_data_surround_vote_true() {
        let attestation_data_1 = AttestationData {
            source: Checkpoint {
                epoch: 0,
                root: H256::from([0; 32]),
            },
            target: Checkpoint {
                epoch: 3,
                root: H256::from([0; 32]),
            },
            ..AttestationData::default()
        };
        let attestation_data_2 = AttestationData {
            source: Checkpoint {
                epoch: 1,
                root: H256::from([1; 32]),
            },
            target: Checkpoint {
                epoch: 2,
                root: H256::from([0; 32]),
            },
            ..AttestationData::default()
        };
        assert_eq!(
            is_slashable_attestation_data(&attestation_data_1, &attestation_data_2),
            true
        );
    }

    // #[test]
    // fn test_is_valid_indexed_attestation_custody_bit_set() {
    //     let state: BeaconState<MainnetConfig> = BeaconState::<MainnetConfig>::default();
    //     let attestation: IndexedAttestation<MainnetConfig> = IndexedAttestation {
    //         custody_bit_1_indices: VariableList::from(vec![1, 2]),
    //         ..IndexedAttestation::default()
    //     };
    //     assert_eq!(
    //         is_valid_indexed_attestation::<MainnetConfig>(&state, &attestation),
    //         Err(Error::CustodyBitSet)
    //     );
    // }

    #[test]
    fn test_is_valid_indexed_attestation_max_indices_exceeded() {
        let state: BeaconState<MainnetConfig> = BeaconState::<MainnetConfig>::default();
        let bit_0_indices: Vec<u64> = (0u64..4096u64).collect();
        let bit_1_indices: Vec<u64> = vec![1];
        let attestation: IndexedAttestation<MainnetConfig> = IndexedAttestation {
            custody_bit_0_indices: VariableList::from(bit_0_indices),
            custody_bit_1_indices: VariableList::from(bit_1_indices),
            ..IndexedAttestation::default()
        };
        assert_eq!(
            is_valid_indexed_attestation::<MainnetConfig>(&state, &attestation),
            Err(Error::MaxIndicesExceeded)
        );
    }

    #[test]
    fn test_is_valid_indexed_attestation_custody_bit_validators_intersect() {
        let state: BeaconState<MainnetConfig> = BeaconState::<MainnetConfig>::default();
        let bit_0_indices: Vec<u64> = (0u64..64u64).collect();
        let bit_1_indices: Vec<u64> = vec![1u64];
        let attestation: IndexedAttestation<MainnetConfig> = IndexedAttestation {
            custody_bit_0_indices: VariableList::from(bit_0_indices),
            custody_bit_1_indices: VariableList::from(bit_1_indices),
            ..IndexedAttestation::default()
        };
        assert_eq!(
            is_valid_indexed_attestation::<MainnetConfig>(&state, &attestation),
            Err(Error::CustodyBitValidatorsIntersect)
        );
    }

    #[test]
    fn test_is_valid_indexed_attestation_bad_validator_indices_ordering() {
        let state: BeaconState<MainnetConfig> = BeaconState::<MainnetConfig>::default();
        let bit_0_indices: Vec<u64> = (0u64..64u64).collect();
        let bit_1_indices: Vec<u64> = vec![66u64, 65u64];
        let attestation: IndexedAttestation<MainnetConfig> = IndexedAttestation {
            custody_bit_0_indices: VariableList::from(bit_0_indices),
            custody_bit_1_indices: VariableList::from(bit_1_indices),
            ..IndexedAttestation::default()
        };
        assert_eq!(
            is_valid_indexed_attestation::<MainnetConfig>(&state, &attestation),
            Err(Error::BadValidatorIndicesOrdering)
        );
    }
}
