use criterion::criterion_main;

mod bench {
    use criterion::{criterion_group, Criterion};
    use curv::cryptographic_primitives::secret_sharing::feldman_vss::VerifiableSS;
    use curv::elliptic::curves::secp256_k1::{FE, GE};
    use curv::elliptic::curves::traits::*;
    use multi_party_ecdsa::protocols::multi_party_ecdsa::bs_2023::party_i::*;
    pub fn bench_full_keygen_party_one_two(c: &mut Criterion) {
        c.bench_function("keygen t=1 n=2", move |b| {
            b.iter(|| {
                keygen_t_n_parties(1, 2);
            })
        });
    }
    pub fn bench_full_keygen_party_two_three(c: &mut Criterion) {
        c.bench_function("keygen t=2 n=3", move |b| {
            b.iter(|| {
                keygen_t_n_parties(2, 3);
            })
        });
    }
    pub fn keygen_t_n_parties(
        t: u16,
        n: u16,
    ) -> (Vec<Keys>, Vec<SharedKeys>, Vec<GE>, GE, VerifiableSS<GE>) {
        let parames = Parameters {
            threshold: t,
            share_count: n,
        };
        let (t, n) = (t as usize, n as usize);
        let party_keys_vec = (0..n).map(Keys::create).collect::<Vec<Keys>>();

        let mut bc1_vec = Vec::new();
        let mut decom_vec = Vec::new();

        for key in &party_keys_vec {
            let (bc1, decom1) =  key.phase1_broadcast_phase3_proof_of_correct_key_proof_of_correct_h1h2();
            bc1_vec.push(bc1);
            decom_vec.push(decom1);
        }

        let y_vec = (0..n).map(|i| decom_vec[i].y_i).collect::<Vec<GE>>();
        let mut y_vec_iter = y_vec.iter();
        let head = y_vec_iter.next().unwrap();
        let tail = y_vec_iter;
        let y_sum = tail.fold(head.clone(), |acc, x| acc + x);
        let mut vss_scheme_vec = Vec::new();
        let mut secret_shares_vec = Vec::new();
        let mut index_vec = Vec::new();
        for key in &party_keys_vec {
            let (vss_scheme, secret_shares, index) = key
                .phase1_verify_com_phase3_verify_correct_key_verify_dlog_phase2_distribute(
                    &parames, &decom_vec, &bc1_vec,
                )
                .expect("invalid key");
            vss_scheme_vec.push(vss_scheme);
            secret_shares_vec.push(secret_shares);
            index_vec.push(index);
        }
        let vss_scheme_for_test = vss_scheme_vec.clone();

        let party_shares = (0..n)
            .map(|i| {
                (0..n)
                    .map(|j| {
                        let vec_j = &secret_shares_vec[j];
                        vec_j[i]
                    })
                    .collect::<Vec<FE>>()
            })
            .collect::<Vec<Vec<FE>>>();

        let mut shared_keys_vec = Vec::new();
        let mut dlog_proof_vec = Vec::new();
        for i in 0..n {
            let (shared_keys, dlog_proof) = party_keys_vec[i]
                .phase2_verify_vss_construct_keypair_phase3_pok_dlog(
                    &parames,
                    &y_vec,
                    &party_shares[i],
                    &vss_scheme_vec,
                    &index_vec[i] + 1,
                )
                .expect("invalid vss");
            shared_keys_vec.push(shared_keys);
            dlog_proof_vec.push(dlog_proof);
        }

        let pk_vec = (0..n).map(|i| dlog_proof_vec[i].pk).collect::<Vec<GE>>();

        //both parties run:
        Keys::verify_dlog_proofs(&parames, &dlog_proof_vec, &y_vec).expect("bad dlog proof");

        //test
        let xi_vec = (0..=t).map(|i| shared_keys_vec[i].x_i).collect::<Vec<FE>>();
        let x = vss_scheme_for_test[0]
            .clone()
            .reconstruct(&index_vec[0..=t], &xi_vec);
        let sum_u_i = party_keys_vec.iter().fold(FE::zero(), |acc, x| acc + x.u_i);
        assert_eq!(x, sum_u_i);

        (
            party_keys_vec,
            shared_keys_vec,
            pk_vec,
            y_sum,
            vss_scheme_for_test[0].clone(),
        )
    }

    criterion_group! {
    name = keygen;
    config = Criterion::default().sample_size(10);
    targets =
    self::bench_full_keygen_party_one_two,
    self::bench_full_keygen_party_two_three}
}

criterion_main!(bench::keygen);
