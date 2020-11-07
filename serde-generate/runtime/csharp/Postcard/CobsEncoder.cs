// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

using System;

namespace Postcard
{
    public static class CobsEncoder
    {
        public static int MaxEncodedSize(int size) => size + 2 + size / 254;

        /// <summary>Encodes a complete COBS frame, including the '0' terminator.</summary>
        /// <exception cref="IndexOutOfRangeException">The output is not large enough to encode the input data</exception>
        public static int EncodeFrame(ReadOnlySpan<byte> input, Span<byte> output)
        {
            byte code = 1;
            int code_index = 0;
            int output_index = 1;
            foreach (byte b in input)
            {
                if (b == 0)
                {
                    output[code_index] = code;
                    code = 1;
                    code_index = output_index;
                    output_index++;
                }
                else
                {
                    output[output_index] = b;
                    output_index++;
                    code++;

                    if (code == byte.MaxValue)
                    {
                        output[code_index] = code;
                        code = 1;
                        code_index = output_index;
                        output_index++;
                    }
                }
            }

            if (output_index == 1)
                return 0;

            output[code_index] = code;
            output[output_index] = 0;
            return output_index + 1;
        }

        /// <summary>Encodes a complete COBS frame, including the '0' terminator.</summary>
        public static byte[] EncodeFrame(ReadOnlySpan<byte> input)
        {
            byte[] buf = new byte[MaxEncodedSize(input.Length)];
            int len = EncodeFrame(input, buf);
            Array.Resize(ref buf, len);
            return buf;
        }
    }
}
