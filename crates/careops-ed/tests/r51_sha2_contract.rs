// Apache-2.0. Expected digests generated independently with Python3.15.1 hashlib.
// No digest expected value is computed by sha2 or the production helper.
use careops_ed::disposition::api_runner::compute_sha256_hex;

#[test]
fn independent_standard_full_byte_and_high_tick_vectors() {
    let vectors: Vec<(&str, Vec<u8>, &str)> = vec![
        (
            "empty",
            Vec::new(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            "abc",
            b"abc".to_vec(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            "standard-multiblock",
            b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq".to_vec(),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        ),
        (
            "binary-1",
            (0..1usize).map(|i| ((i * 131 + 17) % 256) as u8).collect(),
            "4a64a107f0cb32536e5bce6c98c393db21cca7f4ea187ba8c4dca8b51d4ea80a",
        ),
        (
            "binary-55",
            (0..55usize).map(|i| ((i * 131 + 17) % 256) as u8).collect(),
            "59aaae80b8e7958f5757b4ac9274f1fd57ec0dc7aa8349319102316781b92006",
        ),
        (
            "binary-56",
            (0..56usize).map(|i| ((i * 131 + 17) % 256) as u8).collect(),
            "dca902d31487ffab357ce36cc5abc6947fbb69127ec04c468ba5687268c4bf7c",
        ),
        (
            "binary-63",
            (0..63usize).map(|i| ((i * 131 + 17) % 256) as u8).collect(),
            "b8e655e9e7ad413b96f0a371eb1d6db71dcca27f4eee35892517cbe4bfaa6f9a",
        ),
        (
            "binary-64",
            (0..64usize).map(|i| ((i * 131 + 17) % 256) as u8).collect(),
            "e5146be62accc56709594cb45c651b361df94f622cbb09b91ea3ca7a2060bd53",
        ),
        (
            "binary-65",
            (0..65usize).map(|i| ((i * 131 + 17) % 256) as u8).collect(),
            "4b6c6774f0cbcd776a90e187b9494e04ca880c0afe3ea1c5cad51c8bbcb41f74",
        ),
        (
            "binary-127",
            (0..127usize)
                .map(|i| ((i * 131 + 17) % 256) as u8)
                .collect(),
            "b4f5398bf618d741f2ab0f6e2fcde06f4949c20d476b5f6e1d4bf8618bc8973a",
        ),
        (
            "binary-128",
            (0..128usize)
                .map(|i| ((i * 131 + 17) % 256) as u8)
                .collect(),
            "ed045c7c319e0c9286e1dea21c80bfd4a08eca43e02e13ee8a7a2c8b4b2baec9",
        ),
        (
            "binary-129",
            (0..129usize)
                .map(|i| ((i * 131 + 17) % 256) as u8)
                .collect(),
            "d6f2fca0407b3f98c9ce29ae92a6cdf489506450fdbb29172f6b18987bc685db",
        ),
        (
            "binary-1024",
            (0..1024usize)
                .map(|i| ((i * 131 + 17) % 256) as u8)
                .collect(),
            "575383deff0775797e1ff4c82da069141937c843a81cbe50959a96bcfa88140e",
        ),
        (
            "binary-65536",
            (0..65536usize)
                .map(|i| ((i * 131 + 17) % 256) as u8)
                .collect(),
            "d8f32a3b4c195f65444e342b0c61bdaf114802e84ab7821a1f88e3f66ea15100",
        ),
        (
            "binary-1048576",
            (0..1048576usize)
                .map(|i| ((i * 131 + 17) % 256) as u8)
                .collect(),
            "ea53eb498f9b7dc054aae73924bf25c569604d2c20aa9016f0b8714a1c180c7b",
        ),
        (
            "all-byte-values",
            (0..=255u8).collect(),
            "40aff2e9d2d8922e47afd4648e6967497158785fbd1da870e7110266bf944880",
        ),
        (
            "million-a",
            vec![b'a'; 1_000_000],
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0",
        ),
        (
            "u64-0",
            0u64.to_le_bytes().to_vec(),
            "af5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc",
        ),
        (
            "u64-1",
            1u64.to_le_bytes().to_vec(),
            "7c9fa136d4413fa6173637e883b6998d32e1d675f88cddff9dcbcf331820f4b8",
        ),
        (
            "u64-4294967296",
            4294967296u64.to_le_bytes().to_vec(),
            "01acecb507abfe1a354aa8064f4af5d3f1acd019e37db3c11c97523b71c76e9d",
        ),
        (
            "u64-9223372036854775808",
            9223372036854775808u64.to_le_bytes().to_vec(),
            "e6ad6c9a3a3b7658c35bacf6553fcb8ffe34387534a648fe18f875b8f7a86ddb",
        ),
        (
            "u64-18446744073709551615",
            18446744073709551615u64.to_le_bytes().to_vec(),
            "12a3ae445661ce5dee78d0650d33362dec29c4f82af05e7e57fb595bbbacf0ca",
        ),
    ];
    for (label, input, expected) in vectors {
        let actual = compute_sha256_hex(&input);
        assert_eq!(actual, expected, "{label}");
        assert_eq!(actual.len(), 64);
        assert!(actual
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));
    }
}

#[test]
fn one_input_bit_changes_the_independently_known_digest() {
    let input: Vec<_> = (0..=255u8).collect();
    let original = compute_sha256_hex(&input);
    let mut changed = input;
    changed[0] ^= 1;
    assert_ne!(original, compute_sha256_hex(&changed));
    assert_eq!(
        compute_sha256_hex(&changed),
        "7bb462c3bd371dd81c06ad1d2b635971cb56eb22233dfc9febe83e44c840b8d7"
    );
}
