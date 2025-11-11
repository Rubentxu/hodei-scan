# Cap'n Proto Serialization for Custom FactTypes

## Overview

This document describes the Cap'n Proto serialization support for hodei-scan IR v3.3, with specific focus on Custom FactTypes.

## Prerequisites

To use Cap'n Proto serialization, you need to install the Cap'n Proto compiler:

### Installing Cap'n Proto Compiler

**macOS:**
```bash
brew install capnp
```

**Ubuntu/Debian:**
```bash
sudo apt-get install capnproto
```

**Arch Linux:**
```bash
sudo pacman -S capnproto
```

**From source:**
```bash
git clone https://github.com/capnproto/capnproto
cd capnproto
make -j$(nproc)
sudo make install
```

Verify installation:
```bash
capnp --version
```

## Schema Changes (v3.2 → v3.3)

The IR schema has been updated to support Custom FactTypes:

### New Types Added

1. **FactValueType Enum**
   - `string`: UTF-8 text
   - `number`: 64-bit floating point
   - `boolean`: boolean value
   - `array`: list of FactValue
   - `object`: list of KeyValuePair

2. **FactValue Struct**
   - Union type supporting all 5 value types
   - Recursive structure for nested arrays/objects

3. **KeyValuePair Struct**
   - Simple key-value pair for object fields

4. **CustomFactType Struct**
   - `discriminant`: Unique plugin identifier (e.g., "my-plugin:custom-type")
   - `data`: List of KeyValuePair for dynamic fields

5. **Custom FactType Variant**
   - Added variant 16 to FactType union
   - Backward compatible with v3.2 IRs

## Building with Cap'n Proto

When the Cap'n Proto compiler is installed, build with:

```bash
cargo build --features capnp
```

This will:
1. Compile `schema/ir.capnp` to `src/generated/ir_capnp.rs`
2. Include the `capnp_serialization` module
3. Enable all Cap'n Proto functionality

## Usage Examples

### Serializing a Custom FactType

```rust
use hodei_ir::{FactType, FactValue, CustomFactType};
use std::collections::HashMap;

// Create a custom fact
let mut data = HashMap::new();
data.insert("severity".to_string(), FactValue::String("high".to_string()));
data.insert("count".to_string(), FactValue::Number(42.0));

let custom_fact = CustomFactType {
    discriminant: "security:auth-bypass".to_string(),
    data,
};

let fact_type = FactType::Custom(custom_fact);

// Serialize to bytes (when capnp feature is enabled)
#[cfg(feature = "capnp")]
{
    use hodei_ir::serialize_ir_to_bytes;

    let ir = create_test_ir();
    let bytes = serialize_ir_to_bytes(&ir)?;
    // Write to file or send over network
}
```

### Deserializing Custom FactTypes

```rust
#[cfg(feature = "capnp")]
{
    use hodei_ir::deserialize_ir_from_bytes;

    let bytes = read_from_file()?;
    let ir = deserialize_ir_from_bytes(&bytes)?;

    // Access custom facts
    for fact in &ir.facts {
        if let FactType::Custom(custom) = &fact.fact_type {
            println!("Plugin: {}", custom.discriminant);
            for (key, value) in &custom.data {
                println!("  {}: {:?}", key, value);
            }
        }
    }
}
```

## Performance Benefits

Cap'n Proto provides several advantages over JSON/serde:

1. **Binary Format**: ~50% smaller than JSON
2. **Zero-Copy Deserialization**: Read data directly from memory
3. **Schema Evolution**: Optional fields and backward compatibility
4. **Cross-Language**: Generate code for C++, Python, Java, etc.
5. **Fast Serialization**: 10-100x faster than JSON

## Migration from v3.2 to v3.3

The migration tool automatically handles schema version bumps:

```rust
use hodei_ir::migrate_ir;

// Migrate v3.2 IR to v3.3
let v32_ir = load_v32_ir();
let v33_ir = migrate_ir(v32_ir, SchemaVersion::V32, SchemaVersion::V33)?;
```

## Testing

Run tests with:

```bash
# All tests
cargo test

# Cap'n Proto specific tests (requires compiler)
cargo test --features capnp

# Performance benchmarks
cargo bench --features capnp
```

## Limitations

- Cap'n Proto compiler must be installed to build
- Generated schema code is ~500KB
- Some field types have different representations (e.g., f32 vs float64)
- Requires Rust 1.70+ for const generics support

## Troubleshooting

**Error: "capnp: command not found"**
- Install Cap'n Proto compiler (see Prerequisites)

**Error: "schema compilation failed"**
- Check `capnp --version` output
- Verify schema syntax in `schema/ir.capnp`

**Error: "feature 'capnp' not enabled"**
- Build with `--features capnp` flag
- Ensure compiler is installed

## Schema ID

The IR schema uses unique ID: `0xf0a1b2c3d4e5f601`

This ensures compatibility across versions and prevents schema conflicts.
