// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

using System;
using System.Diagnostics.CodeAnalysis;
using System.IO;
using System.Linq;
using System.Numerics;
using System.Text;

namespace Serde {
    public abstract class BinaryDeserializer: IDeserializer, IDisposable {
        protected BinaryReader input;
        private long containerDepthBudget;

        public BinaryDeserializer([NotNull] Stream inputStream, long maxContainerDepth) {
            input = new BinaryReader(inputStream);
            containerDepthBudget = maxContainerDepth;
        }

        public void Dispose() => input.Dispose();

        public abstract long deserialize_len();
        public abstract int deserialize_variant_index();
        public abstract void check_that_key_slices_are_increasing(ReadOnlySpan<byte> key1, ReadOnlySpan<byte> key2);

        public void increase_container_depth() {
            if (containerDepthBudget == 0) {
                throw new DeserializationException("Exceeded maximum container depth");
            }
            containerDepthBudget -= 1;
        }

        public void decrease_container_depth() {
            containerDepthBudget += 1;
        }

        public string deserialize_str() {
            long len = deserialize_len();
            if (len < 0 || len > int.MaxValue) {
                throw new DeserializationException("Incorrect length value for C# string");
            }
            byte[] content = input.ReadBytes((int)len);
            return Encoding.UTF8.GetString(content);
        }

        public Bytes deserialize_bytes() {
            long len = deserialize_len();
            if (len < 0 || len > int.MaxValue) {
                throw new DeserializationException("Incorrect length value for C# array");
            }
            byte[] content = input.ReadBytes((int)len);
            return new Bytes(content);
        }

        public bool deserialize_bool() => input.ReadBoolean();

        public Unit deserialize_unit() => new Unit();

        public virtual char deserialize_char() => throw new NotImplementedException();

        public virtual float deserialize_f32() => throw new NotImplementedException();

        public virtual double deserialize_f64() => throw new NotImplementedException();

        public byte deserialize_u8() => input.ReadByte();

        public ushort deserialize_u16() => input.ReadUInt16();

        public uint deserialize_u32() => input.ReadUInt32();

        public ulong deserialize_u64() => input.ReadUInt64();

        public BigInteger deserialize_u128() {
            BigInteger signed = deserialize_i128();
            if (signed >= 0) {
                return signed;
            } else {
                return signed + (BigInteger.One << 128);
            }
        }

        public sbyte deserialize_i8() => input.ReadSByte();

        public short deserialize_i16() => input.ReadInt16();

        public int deserialize_i32() => input.ReadInt32();

        public long deserialize_i64() => input.ReadInt64();

        public BigInteger deserialize_i128() {
            byte[] content = input.ReadBytes(16).Reverse().ToArray();
            return new BigInteger(content);
        }

        public bool deserialize_option_tag() => input.ReadBoolean();

        public long get_buffer_offset() => input.BaseStream.Position;
    }
}
