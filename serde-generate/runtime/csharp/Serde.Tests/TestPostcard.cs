using NUnit.Framework;
using Postcard;

namespace Serde.Tests
{
    public class TestPostcard
    {
        [Test]
        public void TestSerializeVarint()
        {
            PostcardSerializer serializer = new PostcardSerializer();
            serializer.serialize_len(0);
            serializer.serialize_len(1);
            serializer.serialize_len(2);
            serializer.serialize_len(3);
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 0, 1, 2, 3 });

            serializer = new PostcardSerializer();
            serializer.serialize_len(127);
            serializer.serialize_len(128);
            serializer.serialize_len(512);
            CollectionAssert.AreEqual(serializer.get_bytes(), new byte[] { 127, 128, 1, 128, 4 });
        }

        [Test]
        public void TestCobs()
        {
            byte[][] decoded = new[]
            {
                new byte[] { 0 },
                new byte[] { 0, 0 },
                new byte[] { 0x11, 0x22, 0x00, 0x33 },
                new byte[] { 0x11, 0x22, 0x33, 0x44 },
                new byte[] { 0x11, 0x00, 0x00, 0x00 },
            };
            byte[][] encoded = new[]
            {
                new byte[] { 1, 1, 0 },
                new byte[] { 1, 1, 1, 0 },
                new byte[] { 3, 0x11, 0x22, 2, 0x33, 0 },
                new byte[] { 5, 0x11, 0x22, 0x33, 0x44, 0 },
                new byte[] { 2, 0x11, 1, 1, 1, 0 },
            };

            for (int i = 0; i < decoded.Length; i++)
            {
                CollectionAssert.AreEqual(encoded[i], CobsEncoder.EncodeFrame(decoded[i]), "At index {0}:", i);
                CollectionAssert.AreEqual(decoded[i], CobsDecoder.Decode(encoded[i]), "At index {0}:", i);
            }
        }
    }
}
