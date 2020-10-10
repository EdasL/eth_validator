fn main() {

}
#[cfg(test)]
pub mod tests{
    use eth_validator::validator::*;

    #[test]
    fn get_shard_transition_fields_four_signed_shards_blocks() {

        let shard_state_1 = ShardState {
            slot: 1,
            gasprice: 0,
            latest_block_root: 0
        };

        let shard_state_2 = ShardState {
            slot: 1,
            gasprice: 0,
            latest_block_root: 0
        };

        let shard_states = vec![shard_state_1,shard_state_2];

        let signed_shard_block_1 = SignedShardBlock {
            message: ShardBlock {
                slot: 3,
                shard: 1,
                body: vec![1,2,3]
            }
        };

        let signed_shard_block_2 = SignedShardBlock {
            message: ShardBlock {
                slot: 4,
                shard: 1,
                body: vec![1,2,3]
            }
        };

        let shard_blocks = vec![signed_shard_block_1, signed_shard_block_2];

        let beacon = BeaconState {
            shard_states,
            slot: 5
        };

        let shard : Shard = 1;

        let result = get_shard_transition_fields(&beacon, shard, &shard_blocks);

        let expected_shard_state1 = ShardState{
            slot: 2,
            gasprice: 8,
            latest_block_root: 0
        };
        let expected_shard_state2 = ShardState{
            slot: 3,
            gasprice: 8,
            latest_block_root: 404
        };
        let expected_shard_state3 = ShardState{
            slot: 4,
            gasprice: 8,
            latest_block_root: 404
        };
        let expected_shard_states = vec![expected_shard_state1, expected_shard_state2, expected_shard_state3];
        assert_eq!(result.0, vec![0, 3, 3]);
        assert_eq!(result.1, vec![0, 404, 404]);
        for index in 0..result.2.len() {
            assert_eq!(result.2[index].slot, expected_shard_states[index].slot);
            assert_eq!(result.2[index].gasprice, expected_shard_states[index].gasprice);
            assert_eq!(result.2[index].latest_block_root, expected_shard_states[index].latest_block_root);
        }
    }

    #[test]
    fn get_shard_transition_fields_signed_shard_blocks_with_the_same_slot_number() {
        let shard_state_1 = ShardState {
            slot: 1,
            gasprice: 0,
            latest_block_root: 0
        };

        let shard_state_2 = ShardState {
            slot: 1,
            gasprice: 0,
            latest_block_root: 0
        };

        let shard_states = vec![shard_state_1,shard_state_2];

        let signed_shard_block_1 = SignedShardBlock {
            message: ShardBlock {
                slot: 1,
                shard: 1,
                body: vec![1,2,3]
            }
        };

        let signed_shard_block_2 = SignedShardBlock {
            message: ShardBlock {
                slot: 1,
                shard: 1,
                body: vec![1,2,3]
            }
        };

        let shard_blocks = vec![signed_shard_block_1, signed_shard_block_2];

        let beacon = BeaconState {
            shard_states,
            slot: 1
        };

        let shard : Shard = 1;

        let result = get_shard_transition_fields(&beacon, shard, &shard_blocks);

        assert!(result.0.is_empty());
        assert!(result.1.is_empty());
        assert!(result.2.is_empty());
    }
}
