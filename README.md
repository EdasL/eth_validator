# eth_validator

```python
def get_shard_transition_fields(
    beacon_state: BeaconState,
    shard: Shard,
    shard_blocks: Sequence[SignedShardBlock],
) -> Tuple[Sequence[uint64], Sequence[Root], Sequence[ShardState]]:
    shard_block_lengths = []  # type: PyList[uint64]
    shard_data_roots = []  # type: PyList[Root]
    shard_states = []  # type: PyList[ShardState]

    shard_state = beacon_state.shard_states[shard]
    shard_block_slots = [shard_block.message.slot for shard_block in shard_blocks]
    offset_slots = compute_offset_slots(
        get_latest_slot_for_shard(beacon_state, shard),
        Slot(beacon_state.slot + 1),
    )
    for slot in offset_slots:
        if slot in shard_block_slots:
            shard_block = shard_blocks[shard_block_slots.index(slot)]
            shard_data_roots.append(hash_tree_root(shard_block.message.body))
        else:
            shard_block = SignedShardBlock(message=ShardBlock(slot=slot, shard=shard))
            shard_data_roots.append(Root())
        shard_state = shard_state.copy()
        process_shard_block(shard_state, shard_block.message)
        shard_states.append(shard_state)
        shard_block_lengths.append(uint64(len(shard_block.message.body)))

    return shard_block_lengths, shard_data_roots, shard_states
```

#### `BeaconState`

```python
class BeaconState(Container):
    # Versioning
    genesis_time: uint64
    genesis_validators_root: Root
    slot: Slot
    fork: Fork
    # History
    latest_block_header: BeaconBlockHeader
    block_roots: Vector[Root, SLOTS_PER_HISTORICAL_ROOT]
    state_roots: Vector[Root, SLOTS_PER_HISTORICAL_ROOT]
    historical_roots: List[Root, HISTORICAL_ROOTS_LIMIT]
    # Eth1
    eth1_data: Eth1Data
    eth1_data_votes: List[Eth1Data, EPOCHS_PER_ETH1_VOTING_PERIOD * SLOTS_PER_EPOCH]
    eth1_deposit_index: uint64
    # Registry
    validators: List[Validator, VALIDATOR_REGISTRY_LIMIT]
    balances: List[Gwei, VALIDATOR_REGISTRY_LIMIT]
    # Randomness
    randao_mixes: Vector[Bytes32, EPOCHS_PER_HISTORICAL_VECTOR]
    # Slashings
    slashings: Vector[Gwei, EPOCHS_PER_SLASHINGS_VECTOR]  # Per-epoch sums of slashed effective balances
    # Attestations
    previous_epoch_attestations: List[PendingAttestation, MAX_ATTESTATIONS * SLOTS_PER_EPOCH]
    current_epoch_attestations: List[PendingAttestation, MAX_ATTESTATIONS * SLOTS_PER_EPOCH]
    # Finality
    justification_bits: Bitvector[JUSTIFICATION_BITS_LENGTH]  # Bit set for every recent justified epoch
    previous_justified_checkpoint: Checkpoint  # Previous epoch snapshot
    current_justified_checkpoint: Checkpoint
    finalized_checkpoint: Checkpoint
```

### `ShardBlock`

```python
class ShardBlock(Container):
    shard_parent_root: Root
    beacon_parent_root: Root
    slot: Slot
    shard: Shard
    proposer_index: ValidatorIndex
    body: ByteList[MAX_SHARD_BLOCK_SIZE]
```

### `SignedShardBlock`

```python
class SignedShardBlock(Container):
    message: ShardBlock
    signature: BLSSignature
```

### `ShardState`

```python
class ShardState(Container):
    slot: Slot
    gasprice: Gwei
    latest_block_root: Root
```

```yaml
root: bytes32         -- string, hash-tree-root of the value, hex encoded, with prefix 0x
```

### `slots.yaml`

An integer. The amount of slots to process (i.e. the difference in slots between pre and post), always a positive number.

#### `compute_offset_slots`

```python
def compute_offset_slots(start_slot: Slot, end_slot: Slot) -> Sequence[Slot]:
    """
    Return the offset slots that are greater than ``start_slot`` and less than ``end_slot``.
    """
    return [Slot(start_slot + x) for x in SHARD_BLOCK_OFFSETS if start_slot + x < end_slot]
```

#### `get_latest_slot_for_shard`

```python
def get_latest_slot_for_shard(state: BeaconState, shard: Shard) -> Slot:
    """
    Return the latest slot number of the given ``shard``.
    """
    return state.shard_states[shard].slot
```

#### `hash_tree_root`

```python
def hash_tree_root(obj: View) -> Bytes32:
    return Bytes32(obj.get_backing().merkle_root())
```

#### `process_shard_block`

```python
def process_shard_block(shard_state: ShardState,
                        block: ShardBlock) -> None:
    """
    Update ``shard_state`` with shard ``block``.
    """
    shard_state.slot = block.slot
    prev_gasprice = shard_state.gasprice
    shard_block_length = len(block.body)
    shard_state.gasprice = compute_updated_gasprice(prev_gasprice, uint64(shard_block_length))
    if shard_block_length != 0:
        shard_state.latest_block_root = hash_tree_root(block)
```

#### `compute_updated_gasprice`

```python
def compute_updated_gasprice(prev_gasprice: Gwei, shard_block_length: uint64) -> Gwei:
    if shard_block_length > TARGET_SHARD_BLOCK_SIZE:
        delta = (prev_gasprice * (shard_block_length - TARGET_SHARD_BLOCK_SIZE)
                 // TARGET_SHARD_BLOCK_SIZE // GASPRICE_ADJUSTMENT_COEFFICIENT)
        return min(prev_gasprice + delta, MAX_GASPRICE)
    else:
        delta = (prev_gasprice * (TARGET_SHARD_BLOCK_SIZE - shard_block_length)
                 // TARGET_SHARD_BLOCK_SIZE // GASPRICE_ADJUSTMENT_COEFFICIENT)
        return max(prev_gasprice, MIN_GASPRICE + delta) - delta
```
