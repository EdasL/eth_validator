pub mod validator {

    pub struct BeaconState {
        shard_states: Vec<ShardState>,
        slot: Slot,
    }

    pub struct ShardBlock {
        slot: Slot,
        shard: Shard,
        body: ByteList,
    }

    pub struct SignedShardBlock {
        message: ShardBlock,
    }

    #[derive(Clone)]
    pub struct ShardState {
        slot: Slot,
    }

    pub type Shard = u64; // Shard number
    pub type Root = i32; // a Merkle root
    pub type Slot = u64; // a slot number
    pub type ByteList = Vec<u8>;

    fn hash_tree_root(list: &ByteList) -> i32 {
        list.len() as i32 // Mock function
    }

    //https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase1/beacon-chain.md#compute_offset_slots
    fn compute_offset_slots(start_slot: Slot, end_slot: Slot) -> Vec<u64> {
        let mut v = Vec::new();
        for i in start_slot + 1..end_slot {
            v.push(i);
        }
        v
    }

    fn process_shard_block(shard_state: &ShardState, block: &ShardBlock) {}

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
            let shard_state_copy = shard_state.clone();
            process_shard_block(shard_state, &shard_block.message);
            shard_states.push(shard_state_copy);
            shard_block_lengths.push(shard_block.message.body.len() as u64);
        }
        (shard_block_lengths, shard_data_roots, shard_states)
    }
}
