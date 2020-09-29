fn main() {

}
#[cfg(test)]
pub mod tests{
    use eth_validator::validator::*;

    #[test]
    fn get_shard_transition_fields_random_data_test() {

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

        let a = get_shard_transition_fields(&beacon, shard, &shard_blocks);
        println!("lengths - {:?}", a.0);
        println!("data roots - {:?}", a.1);
    }

}