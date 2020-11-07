// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

using Serde;
using System;
using System.Collections.Generic;

namespace Postcard
{
    public class PostcardDeserializer : BinaryDeserializer
    {
        public PostcardDeserializer(byte[] input) : base(input, long.MaxValue) { }
        public PostcardDeserializer(ArraySegment<byte> input) : base(input, long.MaxValue) { }

        private uint deserialize_varint()
        {
            List<byte> data = new List<byte>(5);
            for (int i = 0; i < data.Capacity; i++)
            {
                byte x = reader.ReadByte();
                data.Add(x);
                if ((x & 0x80) == 0) break;
            }
            if ((data[data.Count - 1] & 0x80) != 0)
            {
                throw new DeserializationException("Varint larger than 32-bits");
            }
            data.Reverse();

            uint result = 0;
            foreach (byte b in data)
            {
                result <<= 7;
                result |= (uint)b & 0x7F;
            }
            return result;
        }

        public override long deserialize_len() => deserialize_varint();

        public override int deserialize_variant_index() => (int)deserialize_varint();

        public override void check_that_key_slices_are_increasing(Range key1, Range key2)
        {
            // Not required by the format.
        }
    }
}
