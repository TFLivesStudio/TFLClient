# Minecraft Java world fixture

This test data comes from the Apoptosis MIT-licensed repository, commit
`70af822a0ee2071560c9662b939f04876a065a1f`. Apoptosis generated the save with
Vanilla Minecraft Java 26.1.2, seed 1361882806, and captured the original
`level.dat` plus one original chunk in a minimal region file. The source
records that the data includes no private user data. [Manifest and checksums](https://github.com/antifield26/Apoptosis/blob/70af822a0ee2071560c9662b939f04876a065a1f/crates/test-support/fixtures/anvil/MANIFEST.txt) · [Provenance](https://github.com/antifield26/Apoptosis/blob/70af822a0ee2071560c9662b939f04876a065a1f/docs/research/provenance.md)

| File                |  Bytes | SHA-256                                                            |
| ------------------- | -----: | ------------------------------------------------------------------ |
| `level_26_1_2.dat`  |    393 | `a05f2ad09a7c25f1b9b551f913d1a86cc1f96e0a04db5817dbf365473940ecc6` |
| `region_26_1_2.mca` | 16,384 | `d3f37d9418b18eb031ee363a2d527c9c398cd502c067c46129e0fa24a6fc135c` |

GitHub Actions runs the world manager against a temporary copy. The test
checks discovery in a nested game directory with a Unicode name, visible
renaming, and byte-for-byte restoration of both files. The region fixture
contains one chunk. This test exercises the world manager's filesystem and
NBT behavior and does not start Minecraft.
