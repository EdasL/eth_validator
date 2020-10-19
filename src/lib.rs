pub mod validator {
    use std::{collections::HashSet, ptr::eq};

    pub struct BeaconState {
        pub shard_states: Vec<ShardState>,
        pub slot: Slot,
    }

    pub struct ShardBlock {
        pub slot: Slot,
        pub shard: Shard,
        pub body: ByteList,
    }

    pub struct SignedShardBlock {
        pub message: ShardBlock,
    }

    #[derive(Clone)]
    pub struct ShardState {
        pub slot: Slot,
        pub gasprice: Gwei,
        pub latest_block_root: Root,
    }

    pub type Shard = u64; // Shard number
    pub type Root = i32; // a Merkle root
    pub type Slot = u64; // a slot number
    pub type ByteList = Vec<u8>;
    pub type BitList = [bool; MAX_VALIDATORS_PER_COMMITTEE];
    pub type Gwei = u64; // an amount in Gwei
    pub type Epoch = u64; // an epoch number
    pub type CommitteeIndex = u64; // a committee index at a slot
    pub type BLSSignature = u128; // a BLS12-381 signature
    pub type ValidatorIndex = u64; // a validator registry index

    const SHARD_BLOCK_OFFSETS: [i32; 12] = [1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233];
    const TARGET_SHARD_BLOCK_SIZE: u64 = 262144;
    const GASPRICE_ADJUSTMENT_COEFFICIENT: u64 = 8;
    const MAX_GASPRICE: u64 = 16384;
    const MIN_GASPRICE: u64 = 8;
    const MAX_VALIDATORS_PER_COMMITTEE: usize = 2048;
    const INITIAL_ACTIVE_SHARDS: u64 = 64;
    const SLOTS_PER_EPOCH: u64 = 32;
    const EFFECTIVE_BALANCE_INCREMENT: u64 = 1000000000;

    fn hash_tree_root<T>(_obj: &T) -> i32 {
        404 // Mock function
    }

    //https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase1/beacon-chain.md#compute_offset_slots
    fn compute_offset_slots(start_slot: Slot, end_slot: Slot) -> Vec<u64> {
        let mut v = Vec::<Slot>::new();
        for x in SHARD_BLOCK_OFFSETS.iter() {
            if start_slot + (*x as Slot) < end_slot {
                v.push(start_slot + (*x as Slot));
            }
        }
        v
    }

    fn process_shard_block(shard_state: &mut ShardState, block: &ShardBlock) {
        shard_state.slot = block.slot;
        let prev_gasprice = shard_state.gasprice;
        let shard_block_length = block.body.len();
        shard_state.gasprice = compute_updated_gasprice(prev_gasprice, shard_block_length as u64);
        if shard_block_length != 0 {
            shard_state.latest_block_root = hash_tree_root(block)
        }
    }

    fn compute_updated_gasprice(prev_gasprice: Gwei, shard_block_length: u64) -> Gwei {
        if shard_block_length > TARGET_SHARD_BLOCK_SIZE {
            let delta = prev_gasprice * (shard_block_length - TARGET_SHARD_BLOCK_SIZE)
                / TARGET_SHARD_BLOCK_SIZE
                / GASPRICE_ADJUSTMENT_COEFFICIENT;
            std::cmp::min(prev_gasprice + delta, MAX_GASPRICE)
        } else {
            let delta = prev_gasprice * (TARGET_SHARD_BLOCK_SIZE - shard_block_length)
                / TARGET_SHARD_BLOCK_SIZE
                / GASPRICE_ADJUSTMENT_COEFFICIENT;
            std::cmp::max(prev_gasprice, MIN_GASPRICE + delta) - delta
        }
    }

    //https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase1/beacon-chain.md#get_latest_slot_for_shard
    fn get_latest_slot_for_shard(state: &BeaconState, shard: Shard) -> Slot {
        state.shard_states[shard as usize].slot
    }

    pub fn get_shard_transition_fields(
        beacon_state: &BeaconState,
        shard: Shard,
        shard_blocks: &Vec<SignedShardBlock>,
    ) -> (Vec<u64>, Vec<Root>, Vec<ShardState>) {
        let mut shard_block_lengths = Vec::<u64>::new();
        let mut shard_data_roots = Vec::<Root>::new();
        let mut shard_states = Vec::<ShardState>::new();

        let shard_state = &beacon_state.shard_states[shard as usize];
        let mut shard_block_slots = Vec::<Slot>::new();

        for shard_block in shard_blocks.iter() {
            shard_block_slots.push(shard_block.message.slot);
        }

        let offset_slots = compute_offset_slots(
            get_latest_slot_for_shard(beacon_state, shard),
            (beacon_state.slot + 1) as Slot,
        );

        for slot in offset_slots.iter() {
            let shard_block: &SignedShardBlock;
            let new_shard_block = SignedShardBlock {
                message: ShardBlock {
                    slot: *slot,
                    shard: shard,
                    body: Vec::new(),
                },
            };
            if shard_block_slots.contains(slot) {
                shard_block =
                    &shard_blocks[shard_block_slots.iter().position(|x| x == slot).unwrap()];
                shard_data_roots.push(hash_tree_root(&shard_block.message.body));
            } else {
                shard_data_roots.push(Root::default());
                shard_block = &new_shard_block;
            };
            let mut shard_state_copy = shard_state.clone();
            process_shard_block(&mut shard_state_copy, &shard_block.message);
            shard_states.push(shard_state_copy);
            shard_block_lengths.push(shard_block.message.body.len() as u64);
        }
        (shard_block_lengths, shard_data_roots, shard_states)
    }

    pub fn get_shard_winning_roots(
        state: &BeaconState,
        attestations: &Vec<Attestation>,
    ) -> (Vec<Shard>, Vec<Root>) {
        let mut shards = Vec::<Shard>::new();
        let mut winning_roots = Vec::<Root>::new();
        let online_indices = get_online_validator_indices(state);
        let on_time_attestation_slot = compute_previous_slot(state.slot);
        let committee_count =
            get_committee_count_per_slot(state, compute_epoch_at_slot(on_time_attestation_slot));
        for committee_index in 0..committee_count {
            let shard = compute_shard_from_committee_index(
                state,
                committee_index,
                on_time_attestation_slot,
            );
            let mut shard_attestations = Vec::<&Attestation>::new();
            for attestation in attestations {
                if is_on_time_attestation(state, &attestation.data)
                    && attestation.data.index == committee_index
                {
                    shard_attestations.push(attestation);
                }
            }
            let committee = get_beacon_committee(state, on_time_attestation_slot, committee_index);
            let shard_transition_roots = shard_attestations
                .iter()
                .map(|x| x.data.shard_transition_root)
                .collect::<HashSet<Root>>();
            let sorted_shard_trans_roots = {
                let mut shard_transition_roots =
                    shard_transition_roots.into_iter().collect::<Vec<Root>>();
                shard_transition_roots.sort_unstable();
                shard_transition_roots
            };

            for shard_transition_root in sorted_shard_trans_roots {
                //             transition_attestations = [
                //                 a for a in shard_attestations
                //                 if a.data.shard_transition_root == shard_transition_root
                //             ]
                let transition_attestations = shard_attestations
                    .iter()
                    .filter(|a| a.data.shard_transition_root == shard_transition_root)
                    .map(|a| *a)
                    .collect::<Vec<&Attestation>>();

                let mut transition_participants = HashSet::<ValidatorIndex>::new();
                for attestation in transition_attestations {
                    let participants = get_attesting_indices(
                        state,
                        &attestation.data,
                        attestation.aggregation_bits,
                    );
                    transition_participants = transition_participants
                        .union(&participants.into_iter().collect())
                        .map(|x| *x)
                        .collect();
                    let enough_online_stake = {
                        let online_indices = online_indices
                            .iter()
                            .map(|x| *x)
                            .collect::<HashSet<ValidatorIndex>>();
                        let committee = committee
                            .iter()
                            .map(|x| *x)
                            .collect::<HashSet<ValidatorIndex>>();
                        get_total_balance(
                            state,
                            &online_indices
                                .intersection(&transition_participants)
                                .map(|x| *x)
                                .collect(),
                        ) * 3
                            >= get_total_balance(
                                state,
                                &online_indices
                                    .intersection(&committee)
                                    .map(|x| *x)
                                    .collect(),
                            ) * 2
                    };

                    if enough_online_stake {
                        shards.push(shard);
                        winning_roots.push(shard_transition_root);
                        break;
                    }
                }
            }
        }

        (shards, winning_roots)
    }

    pub struct Checkpoint {
        pub root: Root,
        pub epoch: Epoch,
    }

    pub struct AttestationData {
        pub slot: Slot,
        pub index: CommitteeIndex,
        pub beacon_block_root: Root,
        pub source: Checkpoint,
        pub target: Checkpoint,
        pub shard_transition_root: Root,
    }

    pub struct Attestation {
        pub aggregation_bits: BitList,
        pub data: AttestationData,
        pub signature: BLSSignature,
    }

    // Functions

    pub fn get_online_validator_indices(_state: &BeaconState) -> Vec<ValidatorIndex> {
        let mut set_of_validator_indexes = Vec::<ValidatorIndex>::new();

        // push anything depending on your needs while implementing
        set_of_validator_indexes.push(1 as u64);
        set_of_validator_indexes.push(2 as u64);

        set_of_validator_indexes
    }

    pub fn compute_previous_slot(slot: Slot) -> Slot {
        if slot > 0 {
            return slot - 1;
        } else {
            return slot;
        }
    }

    // Return the number of committees in each slot for the given ``epoch``.
    pub fn get_committee_count_per_slot(_state: &BeaconState, _epoch: Epoch) -> u64 {
        10 as u64
    }

    pub fn compute_shard_from_committee_index(
        _state: &BeaconState,
        index: CommitteeIndex,
        _slot: Slot,
    ) -> Shard {
        // 1 represents shard start value at slot, modify if needed
        (index + 1) % INITIAL_ACTIVE_SHARDS
    }

    // Check if the given ``attestation_data`` is on-time.
    pub fn is_on_time_attestation(state: &BeaconState, attestation_data: &AttestationData) -> bool {
        return eq(&attestation_data.slot, &compute_previous_slot(state.slot));
    }

    // Return the epoch number at ``slot``
    pub fn compute_epoch_at_slot(slot: Slot) -> Epoch {
        return slot / SLOTS_PER_EPOCH;
    }

    // Return the beacon committee at ``slot`` for ``index``
    pub fn get_beacon_committee(
        _state: &BeaconState,
        _slot: Slot,
        _index: CommitteeIndex,
    ) -> Vec<ValidatorIndex> {
        let mut set_of_validator_indexes = Vec::<ValidatorIndex>::new();

        // push anything depending on your needs while implementing
        set_of_validator_indexes.push(1 as u64);
        set_of_validator_indexes.push(2 as u64);

        set_of_validator_indexes
    }

    // Return the set of attesting indices corresponding to ``data`` and ``bits``.
    pub fn get_attesting_indices(
        state: &BeaconState,
        data: &AttestationData,
        _bits: BitList,
    ) -> Vec<ValidatorIndex> {
        let mut set_of_validator_indexes = Vec::<ValidatorIndex>::new();
        let _committee: std::vec::Vec<u64> = get_beacon_committee(state, data.slot, data.index);

        // push anything depending on your needs while implementing
        set_of_validator_indexes.push(1 as u64);
        set_of_validator_indexes.push(2 as u64);

        set_of_validator_indexes
    }

    // Return the combined effective balance of the ``indices``.
    // ``EFFECTIVE_BALANCE_INCREMENT`` Gwei minimum to avoid divisions by zero.
    // Math safe up to ~10B ETH, afterwhich this overflows uint64.
    pub fn get_total_balance(_state: &BeaconState, _indices: &Vec<ValidatorIndex>) -> Gwei {
        return EFFECTIVE_BALANCE_INCREMENT;
    }
}
