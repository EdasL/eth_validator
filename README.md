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
    randao_mixes: Vector[Root, EPOCHS_PER_HISTORICAL_VECTOR]
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
    # Phase 1
    current_epoch_start_shard: Shard
    shard_states: List[ShardState, MAX_SHARDS]
    online_countdown: List[OnlineEpochs, VALIDATOR_REGISTRY_LIMIT]  # not a raw byte array, considered its large size.
    current_light_committee: CompactCommittee
    next_light_committee: CompactCommittee
    # Custody game
    # Future derived secrets already exposed; contains the indices of the exposed validator
    # at RANDAO reveal period % EARLY_DERIVED_SECRET_PENALTY_MAX_FUTURE_EPOCHS
    exposed_derived_secrets: Vector[List[ValidatorIndex, MAX_EARLY_DERIVED_SECRET_REVEALS * SLOTS_PER_EPOCH],
                                    EARLY_DERIVED_SECRET_PENALTY_MAX_FUTURE_EPOCHS]
    custody_chunk_challenge_records: List[CustodyChunkChallengeRecord, MAX_CUSTODY_CHUNK_CHALLENGE_RECORDS]
    custody_chunk_challenge_index: uint64
    
    # Needed fields
    slot: Slot
    shard_states: List[ShardState, MAX_SHARDS]
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

## Functions

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

## Constants

```python
# Mainnet preset - phase 1

CONFIG_NAME: "mainnet"

# phase1-fork
# ---------------------------------------------------------------
PHASE_1_FORK_VERSION: 0x01000000
# [STUB]
PHASE_1_FORK_SLOT: 0
INITIAL_ACTIVE_SHARDS: 64


# beacon-chain
# ---------------------------------------------------------------
# Misc
# 2**10 (= 1,024)
MAX_SHARDS: 1024
# 2**7 (= 128)
LIGHT_CLIENT_COMMITTEE_SIZE: 128
# 2**3 (= 8)
GASPRICE_ADJUSTMENT_COEFFICIENT: 8

# Shard block configs
# 2**20 (= 1048,576) bytes
MAX_SHARD_BLOCK_SIZE: 1048576
# 2**18 (= 262,144) bytes
TARGET_SHARD_BLOCK_SIZE: 262144
# Note: MAX_SHARD_BLOCKS_PER_ATTESTATION is derived from the list length.
SHARD_BLOCK_OFFSETS: [1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233]
# len(SHARD_BLOCK_OFFSETS)
MAX_SHARD_BLOCKS_PER_ATTESTATION: 12
# 2**12 (= 4,096)
BYTES_PER_CUSTODY_CHUNK: 4096
# ceillog2(MAX_SHARD_BLOCK_SIZE // BYTES_PER_CUSTODY_CHUNK)
CUSTODY_RESPONSE_DEPTH: 8

# Gwei values
# 2**14 (= 16,384) Gwei
MAX_GASPRICE: 16384
# 2**3 (= 8) Gwei
MIN_GASPRICE: 8

# Time parameters
# 2**3 (= 8) | online epochs
ONLINE_PERIOD: 8
# 2**8 (= 256) | epochs
LIGHT_CLIENT_COMMITTEE_PERIOD: 256

# Max operations per block
# 2**20 (= 1,048,576) 
MAX_CUSTODY_CHUNK_CHALLENGE_RECORDS: 1048576

# Domain types
DOMAIN_SHARD_PROPOSAL: 0x80000000
DOMAIN_SHARD_COMMITTEE: 0x81000000
DOMAIN_LIGHT_CLIENT: 0x82000000
# custody-game spec
DOMAIN_CUSTODY_BIT_SLASHING: 0x83000000
DOMAIN_LIGHT_SELECTION_PROOF: 0x84000000
DOMAIN_LIGHT_AGGREGATE_AND_PROOF: 0x85000000

# custody-game
# ---------------------------------------------------------------
# Time parameters
# 2**1 (= 2) epochs, 12.8 minutes
RANDAO_PENALTY_EPOCHS: 2
# 2**15 (= 32,768) epochs, ~146 days
EARLY_DERIVED_SECRET_PENALTY_MAX_FUTURE_EPOCHS: 32768
# 2**14 (= 16,384) epochs ~73 days
EPOCHS_PER_CUSTODY_PERIOD: 16384
# 2**11 (= 2,048) epochs, ~9 days
CUSTODY_PERIOD_TO_RANDAO_PADDING: 2048
# 2**15 (= 32,768) epochs, ~146 days
MAX_CHUNK_CHALLENGE_DELAY: 32768

# Misc parameters
# 2**256 - 189
CUSTODY_PRIME: 115792089237316195423570985008687907853269984665640564039457584007913129639747
# 3
CUSTODY_SECRETS: 3
# 2**5 (= 32) bytes
BYTES_PER_CUSTODY_ATOM: 32
# 1/1024 chance of custody bit 1
CUSTODY_PROBABILITY_EXPONENT: 10

# Max operations
# 2**8 (= 256)
MAX_CUSTODY_KEY_REVEALS: 256
# 2**0 (= 1)
MAX_EARLY_DERIVED_SECRET_REVEALS: 1
# 2**2 (= 2)
MAX_CUSTODY_CHUNK_CHALLENGES: 4
# 2** 4 (= 16)
MAX_CUSTODY_CHUNK_CHALLENGE_RESP: 16
# 2**0 (= 1)
MAX_CUSTODY_SLASHINGS: 1

# Reward and penalty quotients
EARLY_DERIVED_SECRET_REVEAL_SLOT_REWARD_MULTIPLE: 2
# 2**8 (= 256)
MINOR_REWARD_QUOTIENT: 256
```

## Custom types

We define the following Python custom types for type hinting and readability:

| Name | SSZ equivalent | Description |
| - | - | - |
| `Slot` | `uint64` | a slot number |
| `Epoch` | `uint64` | an epoch number |
| `CommitteeIndex` | `uint64` | a committee index at a slot |
| `ValidatorIndex` | `uint64` | a validator registry index |
| `Gwei` | `uint64` | an amount in Gwei |
| `Root` | `Bytes32` | a Merkle root |
| `Version` | `Bytes4` | a fork version number |
| `DomainType` | `Bytes4` | a domain type |
| `ForkDigest` | `Bytes4` | a digest of the current fork data |
| `Domain` | `Bytes32` | a signature domain |
| `BLSPubkey` | `Bytes48` | a BLS12-381 public key |
| `BLSSignature` | `Bytes96` | a BLS12-381 signature |
