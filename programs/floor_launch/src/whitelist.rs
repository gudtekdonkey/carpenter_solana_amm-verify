//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!
//!

use solana_sha256_hasher::hashv;
use floor_types::constants::GENESIS_MAX_PROOF_LEN;

///
pub fn leaf(wallet: &[u8; 32]) -> [u8; 32] {
    hashv(&[&[0x00], wallet]).to_bytes()
}

///
pub fn parent(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    hashv(&[&[0x01], lo, hi]).to_bytes()
}

///
pub fn verify(root: &[u8; 32], wallet: &[u8; 32], proof: &[[u8; 32]]) -> bool {
    if *root == [0u8; 32] || proof.len() > GENESIS_MAX_PROOF_LEN {
        return false;
    }
    let mut node = leaf(wallet);
    for sib in proof {
        node = parent(&node, sib);
    }
    node == *root
}

///
#[cfg(test)]
pub fn build(wallets: &[[u8; 32]]) -> ([u8; 32], Vec<Vec<[u8; 32]>>) {
    assert!(!wallets.is_empty());
    let mut level: Vec<[u8; 32]> = wallets.iter().map(leaf).collect();

    let mut pos: Vec<usize> = (0..wallets.len()).collect();
    let mut proofs: Vec<Vec<[u8; 32]>> = vec![Vec::new(); wallets.len()];
    while level.len() > 1 {
        for (i, p) in pos.iter_mut().enumerate() {
            let sib = *p ^ 1;
            if sib < level.len() {
                proofs[i].push(level[sib]);
            }
            *p /= 2;
        }
        level = level.chunks(2).map(|c| if c.len() == 2 { parent(&c[0], &c[1]) } else { c[0] }).collect();
    }
    (level[0], proofs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(n: u8) -> [u8; 32] {
        let mut k = [0u8; 32];
        k[0] = n;
        k[31] = n.wrapping_mul(7);
        k
    }

///
    #[test]
    fn every_member_verifies_odd_and_even_counts() {
        for n in 1..=9u8 {
            let ws: Vec<_> = (1..=n).map(w).collect();
            let (root, proofs) = build(&ws);
            for (i, wl) in ws.iter().enumerate() {
                assert!(verify(&root, wl, &proofs[i]), "{n} wallets, #{i}");
            }
        }
    }

///
    #[test]
    fn a_tree_of_one() {
        let (root, proofs) = build(&[w(1)]);
        assert_eq!(root, leaf(&w(1)));
        assert!(proofs[0].is_empty());
        assert!(verify(&root, &w(1), &[]));
        assert!(!verify(&root, &w(2), &[]));
    }

///
///
    #[test]
    fn outsiders_and_tampering_are_refused() {
        let ws: Vec<_> = (1..=5u8).map(w).collect();
        let (root, proofs) = build(&ws);
        assert!(!verify(&root, &w(9), &proofs[0]), "a wallet outside the tree");
        let mut bad = proofs[2].clone();
        bad[0][5] ^= 1;
        assert!(!verify(&root, &ws[2], &bad), "a tampered sibling");
        assert!(!verify(&root, &ws[2], &proofs[2][1..]), "a dropped sibling");
        let mut more = proofs[2].clone();
        more.push([7u8; 32]);
        assert!(!verify(&root, &ws[2], &more), "an added sibling");
        assert!(!verify(&[0u8; 32], &ws[2], &proofs[2]), "a zero root admits nobody");

        let inner = parent(&leaf(&ws[0]), &leaf(&ws[1]));
        assert!(!verify(&root, &inner, &proofs[0][1..]), "an inner node is not a wallet");

        assert!(!verify(&root, &ws[0], &vec![[0u8; 32]; GENESIS_MAX_PROOF_LEN + 1]));
    }

///
    #[test]
    fn the_scheme_is_sha256_with_domain_bytes_and_sorted_pairs() {
        use sha2_check::sha256;
        let wl = w(1);
        let mut l = vec![0u8];
        l.extend_from_slice(&wl);
        assert_eq!(leaf(&wl), sha256(&l));
        let (a, b) = (leaf(&w(2)), leaf(&w(3)));
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
        let mut p = vec![1u8];
        p.extend_from_slice(&lo);
        p.extend_from_slice(&hi);
        assert_eq!(parent(&a, &b), sha256(&p));
        assert_eq!(parent(&b, &a), sha256(&p), "order-free");
    }

///
///
///
    #[test]
    fn the_shared_fixture_matches() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/genesis-whitelist/vectors.json");
        let mut cases = Vec::new();
        for n in [1u8, 2, 3, 5, 8] {
            let ws: Vec<[u8; 32]> = (0..n).map(|i| sha2_check::sha256(&[b'w', i, n])).collect();
            let (root, proofs) = build(&ws);
            cases.push((ws, root, proofs));
        }
        let hex = |b: &[u8; 32]| b.iter().map(|x| format!("{x:02x}")).collect::<String>();
        let mut out = String::from("{\n  \"scheme\": \"leaf = sha256(0x00 || wallet32); parent = sha256(0x01 || min(a,b) || max(a,b)); odd node carried up\",\n  \"cases\": [\n");
        for (ci, (ws, root, proofs)) in cases.iter().enumerate() {
            out.push_str(&format!("    {{\n      \"root\": \"{}\",\n      \"wallets\": [\n", hex(root)));
            for (i, wl) in ws.iter().enumerate() {
                let pr: Vec<String> = proofs[i].iter().map(|p| format!("\"{}\"", hex(p))).collect();
                out.push_str(&format!(
                    "        {{ \"wallet\": \"{}\", \"leaf\": \"{}\", \"proof\": [{}] }}{}\n",
                    hex(wl),
                    hex(&leaf(wl)),
                    pr.join(", "),
                    if i + 1 < ws.len() { "," } else { "" }
                ));
            }
            out.push_str(&format!("      ]\n    }}{}\n", if ci + 1 < cases.len() { "," } else { "" }));
        }
        out.push_str("  ]\n}\n");
        if std::env::var("GEN_WHITELIST_FIXTURE").is_ok() {
            std::fs::create_dir_all(std::path::Path::new(path).parent().unwrap()).unwrap();
            std::fs::write(path, &out).unwrap();
        }
        let committed = std::fs::read_to_string(path).expect("the committed fixture (GEN_WHITELIST_FIXTURE=1 writes it)");
        assert_eq!(committed, out, "the Rust scheme and the committed fixture disagree");
        for (ws, root, proofs) in &cases {
            for (i, wl) in ws.iter().enumerate() {
                assert!(verify(root, wl, &proofs[i]));
            }
        }
    }

///
    mod sha2_check {
        pub fn sha256(data: &[u8]) -> [u8; 32] {
            const K: [u32; 64] = [
                0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98,
                0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
                0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8,
                0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
                0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819,
                0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
                0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
                0xc67178f2,
            ];
            let mut h: [u32; 8] =
                [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
            let mut m = data.to_vec();
            let bits = (data.len() as u64) * 8;
            m.push(0x80);
            while m.len() % 64 != 56 {
                m.push(0);
            }
            m.extend_from_slice(&bits.to_be_bytes());
            for chunk in m.chunks(64) {
                let mut w = [0u32; 64];
                for i in 0..16 {
                    w[i] = u32::from_be_bytes(chunk[i * 4..i * 4 + 4].try_into().unwrap());
                }
                for i in 16..64 {
                    let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
                    let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
                    w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
                }
                let mut v = h;
                for i in 0..64 {
                    let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
                    let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
                    let t1 = v[7].wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
                    let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
                    let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
                    let t2 = s0.wrapping_add(maj);
                    v = [t1.wrapping_add(t2), v[0], v[1], v[2], v[3].wrapping_add(t1), v[4], v[5], v[6]];
                }
                for i in 0..8 {
                    h[i] = h[i].wrapping_add(v[i]);
                }
            }
            let mut out = [0u8; 32];
            for i in 0..8 {
                out[i * 4..i * 4 + 4].copy_from_slice(&h[i].to_be_bytes());
            }
            out
        }
        #[test]
        fn known_answer() {

            let h = sha256(b"abc");
            assert_eq!(h[..4], [0xba, 0x78, 0x16, 0xbf]);
            assert_eq!(h[28..], [0xf2, 0x00, 0x15, 0xad]);
        }
    }
}
