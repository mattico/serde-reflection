// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

using System;

namespace Postcard
{
    public static class CobsDecoder
    {
        /// <summary>Decodes COBS input into output using the sentinel value `0`. Returns the length of the decoded message
        /// without the terminating sentinel.</summary>
        /// <exception cref="ArgumentException">There is not enough data in input to complete the cobs frame</exception>
        public static int Decode(ReadOnlySpan<byte> input, Span<byte> output)
        {
            int source_index = 0;
            int dest_index = 0;

            while (source_index < input.Length)
            {
                byte code = input[source_index];

                if (source_index + code > input.Length && code != 1)
                {
                    throw new ArgumentException("Not enough data to complete cobs frame");
                }

                source_index++;

                for (int i = 1; i < code; i++)
                {
                    output[dest_index] = input[source_index];
                    source_index++;
                    dest_index++;
                }

                if (code != 0xFF && source_index < input.Length)
                {
                    output[dest_index] = 0;
                    dest_index++;
                }
            }

            return dest_index - 1;
        }

        /// <summary>Decodes COBS input in-place using the sentinel value `0`.</summary>
        /// <exception cref="ArgumentException">There is not enough data in input to complete the cobs frame</exception>
        public static int DecodeInPlace(Span<byte> data) => Decode(data, data);

        /// <summary>Decodes COBS input using the sentinel value `0`.</summary>
        /// <exception cref="ArgumentException">There is not enough data in input to complete the cobs frame</exception>
        public static byte[] Decode(ReadOnlySpan<byte> data)
        {
            byte[] buf = new byte[data.Length];
            int len = Decode(data, buf);
            Array.Resize(ref buf, len);
            return buf;
        }
    }
}
