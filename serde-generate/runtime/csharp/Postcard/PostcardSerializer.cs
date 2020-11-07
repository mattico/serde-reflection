// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

using Serde;
using System;

namespace Postcard
{
    public class PostcardSerializer : BinarySerializer
    {
        public PostcardSerializer() : base(long.MaxValue) { }
        public PostcardSerializer(byte[] buffer) : base(buffer, long.MaxValue) { }
        public PostcardSerializer(ArraySegment<byte> buffer) : base(buffer, long.MaxValue) { }

        private void serialize_varint(uint value)
        {
            for (int i = 0; i < 5; i++)
            {
                byte x = (byte)(value & 0x7F);
                value >>= 7;
                if (value != 0)
                {
                    x |= 0x80;
                    output.Write(x);
                }
                else
                {
                    output.Write(x);
                    return;
                }
            }
        }

        public override void serialize_len(long value) => serialize_varint((uint)value);

        public override void serialize_variant_index(int value) => serialize_varint((uint)value);

        public override void sort_map_entries(int[] offsets)
        {
            // Not required by the format.
        }
    }
}
