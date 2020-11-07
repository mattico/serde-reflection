// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

using System;
using System.Diagnostics.CodeAnalysis;
using System.IO;

namespace Serde.Bincode {
    public class BincodeDeserializer : BinaryDeserializer {
        public BincodeDeserializer([NotNull] Stream input) : base(input, long.MaxValue) { }

        public override long deserialize_len() {
            long value = input.ReadInt64();
            if (value < 0 || value > int.MaxValue) {
                throw new DeserializationException("Incorrect length value");
            }
            return value;
        }

        public override int deserialize_variant_index() => input.ReadInt32();

        public override void check_that_key_slices_are_increasing(ReadOnlySpan<byte> key1, ReadOnlySpan<byte> key2) {
            // Not required by the format.
        }
    }
}
