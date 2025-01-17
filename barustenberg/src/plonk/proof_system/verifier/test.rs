use ark_bn254::{Fq12, G1Projective};
use ark_ff::One;

use super::*;

impl<Fq: Field, Fr: Field + FftField, G1Affine: AffineRepr, H: BarretenHasher, S: Settings<H>>
    Verifier<'_, Fq, Fr, G1Affine, H, S>
{
    pub fn generate_verifier(circuit_proving_key: Rc<RefCell<ProvingKey<'_, Fr, G1Affine>>>) -> Self {
        let mut poly_coefficients: Vec<&mut [Fr]> = vec![&mut []; 8];
        poly_coefficients[0] = circuit_proving_key
            .borrow()
            .polynomial_store
            .get(&"q_1".to_owned())
            .unwrap()
            .borrow_mut()
            .coefficients
            .as_mut_slice();
        poly_coefficients[1] = circuit_proving_key
            .borrow()
            .polynomial_store
            .get(&"q_2".to_owned())
            .unwrap()
            .borrow_mut()
            .coefficients
            .as_mut_slice();
        poly_coefficients[2] = circuit_proving_key
            .borrow()
            .polynomial_store
            .get(&"q_3".to_owned())
            .unwrap()
            .borrow_mut()
            .coefficients
            .as_mut_slice();
        poly_coefficients[3] = circuit_proving_key
            .borrow()
            .polynomial_store
            .get(&"q_m".to_owned())
            .unwrap()
            .borrow_mut()
            .coefficients
            .as_mut_slice();
        poly_coefficients[4] = circuit_proving_key
            .borrow()
            .polynomial_store
            .get(&"q_c".to_owned())
            .unwrap()
            .borrow_mut()
            .coefficients
            .as_mut_slice();
        poly_coefficients[5] = circuit_proving_key
            .borrow()
            .polynomial_store
            .get(&"sigma_1".to_owned())
            .unwrap()
            .borrow_mut()
            .coefficients
            .as_mut_slice();
        poly_coefficients[6] = circuit_proving_key
            .borrow()
            .polynomial_store
            .get(&"sigma_2".to_owned())
            .unwrap()
            .borrow_mut()
            .coefficients
            .as_mut_slice();
        poly_coefficients[7] = circuit_proving_key
            .borrow()
            .polynomial_store
            .get(&"sigma_3".to_owned())
            .unwrap()
            .borrow_mut()
            .coefficients
            .as_mut_slice();

        let mut commitments = vec![G1Affine::default(); 8];
        let mut state = PippengerRuntimeState::new(circuit_proving_key.borrow().circuit_size);

        for i in 0..8 {
            commitments[i] = G1Affine::from_projective(state.pippenger(
                &poly_coefficients[i],
                circuit_proving_key.borrow().reference_string.borrow().get_monomial_points(),
                circuit_proving_key.borrow().circuit_size,
            ));
        }

        // TODOL: this number of points in arbitrary and needs to be checked with the reference string
        let crs = Arc::new(FileReferenceString::new(32, "../srs_db/ignition"));
        let circuit_verification_key = Arc::new(VerificationKey::new(
            circuit_proving_key.borrow().circuit_size,
            circuit_proving_key.borrow().num_public_inputs,
            crs,
            circuit_proving_key.borrow().composer_type,
        ));

        circuit_verification_key
            .commitments
            .insert("Q_1", commitments[0]);
        circuit_verification_key
            .commitments
            .insert("Q_2", commitments[1]);
        circuit_verification_key
            .commitments
            .insert("Q_3", commitments[2]);
        circuit_verification_key
            .commitments
            .insert("Q_M", commitments[3]);
        circuit_verification_key
            .commitments
            .insert("Q_C", commitments[4]);

        circuit_verification_key
            .commitments
            .insert("SIGMA_1", commitments[5]);
        circuit_verification_key
            .commitments
            .insert("SIGMA_2", commitments[6]);
        circuit_verification_key
            .commitments
            .insert("SIGMA_3", commitments[7]);

        let verifier = Verifier::new(
            Some(circuit_verification_key),
            ComposerType::StandardComposer::create_manifest(0),
        );

        let kate_commitment_scheme = Box::new(KateCommitmentScheme::<
            H,
            crate::plonk::proof_system::types::polynomial_manifest::PolynomialIndex,
        >::new());
        verifier.commitment_scheme = kate_commitment_scheme;
        verifier
    }

    pub fn verify_proof(self, proof: &Proof) -> Result<bool, &'static str> {
        // This function verifies a PLONK proof for given program settings.
        // A PLONK proof for standard PLONK is of the form:
        //
        // π_SNARK =   { [a]_1,[b]_1,[c]_1,[z]_1,[t_{low}]_1,[t_{mid}]_1,[t_{high}]_1,[W_z]_1,[W_zω]_1 \in G,
        //                a_eval, b_eval, c_eval, sigma1_eval, sigma2_eval, sigma3_eval,
        //                  q_l_eval, q_r_eval, q_o_eval, q_m_eval, q_c_eval, z_eval_omega \in F }
        //
        // Proof π_SNARK must first be added to the transcript with the other program_settings.
        self.key.program_width = self.settings.program_width();

        // Initialize the transcript.
        let mut transcript = Transcript::new_from_transcript(
            proof.proof_data.as_ref(),
            self.manifest.clone(),
            S.num_challenge_bytes(),
        );

        // Add circuit size and public input size to the transcript.
        transcript.add_element("circuit_size", proof.key.circuit_size.to_be_bytes());
        transcript.add_element(
            "public_input_size",
            proof.key.num_public_inputs.to_be_bytes(),
        );

        // Compute challenges using Fiat-Shamir heuristic.
        transcript.apply_fiat_shamir("init");
        transcript.apply_fiat_shamir("eta");
        transcript.apply_fiat_shamir("beta");
        transcript.apply_fiat_shamir("alpha");
        transcript.apply_fiat_shamir("z");

        // Deserialize alpha and zeta from the transcript.
        let alpha = transcript.get_challenge_field_element("alpha", None);
        let zeta = transcript.get_challenge_field_element("z", None);

        todo!("fail here- are you sure this is the right function?");
        // Compute the evaluations of the Lagrange polynomials and the vanishing polynomial.
        let lagrange_evals = &self.key.domain.evaluate_all_lagrange_coefficients(zeta);

        // Compute quotient polynomial evaluation at zeta.
        let mut t_numerator_eval = Fr::default();
        S::compute_quotient_evaluation_contribution(
            &self.key,
            alpha,
            &transcript,
            &mut t_numerator_eval,
        );
        let t_eval = t_numerator_eval * lagrange_evals.vanishing_poly.inverse();
        transcript.add_element("t", t_eval.to_buffer());

        // Compute nu and separator challenges.
        transcript.apply_fiat_shamir("nu");
        transcript.apply_fiat_shamir("separator");
        let separator_challenge = transcript.get_challenge_field_element("separator");

        // Verify the commitments using Kate commitment scheme.
        self.commitment_scheme.batch_verify(
            &transcript,
            &mut self.kate_g1_elements,
            &mut self.kate_fr_elements,
            &self.key,
        );

        // Append scalar multiplication inputs.
        S::append_scalar_multiplication_inputs(
            &self.key,
            alpha,
            &transcript,
            &mut self.kate_fr_elements,
        );

        // Get PI_Z and PI_Z_OMEGA from the transcript.
        let pi_z = transcript.get_group_element("PI_Z");
        let pi_z_omega = transcript.get_group_element("PI_Z_OMEGA");

        // Check if PI_Z and PI_Z_OMEGA are valid points.
        if !pi_z.on_curve() || pi_z.is_point_at_infinity() {
            return Err("opening proof group element PI_Z not a valid point".into());
        }
        if !pi_z_omega.on_curve() || pi_z_omega.is_point_at_infinity() {
            return Err("opening proof group element PI_Z_OMEGA not a valid point".into());
        }

        // get kate_g1_elements: HashMap<u64, G1Affine> and kate_fr_elements: HashMap<u64, Fr>
        let mut kate_g1_elements: HashMap<String, G1Affine> = HashMap::new();
        let mut kate_fr_elements: HashMap<String, Fr> = HashMap::new();

        // Initialize vectors for scalars and elements
        let mut scalars: Vec<Fr> = Vec::new();
        let mut elements: Vec<G1Affine> = Vec::new();

        // Iterate through the kate_g1_elements and accumulate scalars and elements
        for (key, element) in &kate_g1_elements {
            if element.is_on_curve() && !element.is_zero() {
                if let Some(scalar) = kate_fr_elements.get(key) {
                    scalars.push(*scalar);
                    elements.push(*element);
                }
            }
        }

        // Resize elements vector to make room for Pippenger point table
        let n = elements.len();
        elements.resize(2 * n, G1Affine::zero());

        // Generate Pippenger point table
        generate_pippenger_point_table(&mut elements[..]);

        // Create Pippenger runtime state
        let mut state = PippengerRuntimeState::new(n);

        // Perform Pippenger multi-scalar multiplication
        let p0 = state.pippenger(&scalars, &elements);

        // Calculate P[1]
        let p1 =
            -((G1Projective::from(pi_z_omega) * separator_challenge) + G1Projective::from(pi_z));

        // Check if recursive proof is present
        if let Some(recursive_proof_indices) = self.key.recursive_proof_public_input_indices {
            assert_eq!(recursive_proof_indices.len(), 16);

            let inputs = transcript.get_field_element_vector("public_inputs");

            //  Recover Fq values from public inputs
            let recover_fq_from_public_inputs =
                |idx0: usize, idx1: usize, idx2: usize, idx3: usize| {
                    let l0 = inputs[idx0];
                    let l1 = inputs[idx1];
                    let l2 = inputs[idx2];
                    let l3 = inputs[idx3];

                    let limb = l0
                        + (l1 << NUM_LIMB_BITS_IN_FIELD_SIMULATION)
                        + (l2 << (NUM_LIMB_BITS_IN_FIELD_SIMULATION * 2))
                        + (l3 << (NUM_LIMB_BITS_IN_FIELD_SIMULATION * 3));
                    Fq::from(limb)
                };

            // Get recursion_separator_challenge
            let recursion_separator_challenge =
                transcript.get_challenge_field_element("separator").square();

            // Recover x0, y0, x1, and y1
            let x0 = recover_fq_from_public_inputs(
                recursive_proof_indices[0],
                recursive_proof_indices[1],
                recursive_proof_indices[2],
                recursive_proof_indices[3],
            );
            let y0 = recover_fq_from_public_inputs(
                recursive_proof_indices[4],
                recursive_proof_indices[5],
                recursive_proof_indices[6],
                recursive_proof_indices[7],
            );
            let x1 = recover_fq_from_public_inputs(
                recursive_proof_indices[8],
                recursive_proof_indices[9],
                recursive_proof_indices[10],
                recursive_proof_indices[11],
            );
            let y1 = recover_fq_from_public_inputs(
                recursive_proof_indices[12],
                recursive_proof_indices[13],
                recursive_proof_indices[14],
                recursive_proof_indices[15],
            );

            // Update P[0] and P[1] with recursive proof values
            let p0 = p0 + (G1Projective::new(x0, y0, Fq::one()) * recursion_separator_challenge);
            let p1 = p1 + (G1Projective::new(x1, y1, Fq::one()) * recursion_separator_challenge);
        }

        // Normalize P[0] and P[1]
        let p_affine = [G1Affine::from(p0), G1Affine::from(p1)];

        // Perform final pairing check
        let result = reduced_ate_pairing_batch_precomputed(
            &p_affine,
            &self.key.reference_string.get_precomputed_g2_lines(),
            // TODO this num_points was NOT provided in the original code.
            p_affine.len(),
        );

        // Check if result equals Fq12::one()
        Ok(result == Fq12::one())
        // Err("opening proof group element PI_Z not a valid point".into());
    }
}

fn generate_test_data<
    'a,
    Fq: Field + FftField,
    Fr: Field + FftField,
    G1Affine: AffineRepr,
    H: BarretenHasher,
>(
    n: usize,
) -> Prover<
    'a,
    Fq,
    Fr,
    G1Affine,
    H,
    StandardSettings<H>,
    KateCommitmentScheme<H, StandardSettings<H>>,
> {
    // create some constraints that satisfy our arithmetic circuit relation
    let crs = Rc::new(FileReferenceString::new(n + 1, "../srs_db/ignition"));
    let key = Rc::new(ProvingKey::new(n, 0, crs, ComposerType::Standard));

    let mut rand = rand::thread_rng();

    let mut w_l = Polynomial::new(n);
    let mut w_r = Polynomial::new(n);
    let mut w_o = Polynomial::new(n);
    let mut q_l = Polynomial::new(n);
    let mut q_r = Polynomial::new(n);
    let mut q_o = Polynomial::new(n);
    let mut q_c: Polynomial<Fr> = Polynomial::new(n);
    let mut q_m = Polynomial::new(n);

    let mut t0;
    for i in 0..n / 4 {
        w_l.coefficients[2 * i] = Fr::rand(&mut rand);
        w_r.coefficients[2 * i] = Fr::rand(&mut rand);
        w_o.coefficients[2 * i] = w_l.coefficients[2 * i] * w_r.coefficients[2 * i];
        w_o.coefficients[2 * i] += w_l.coefficients[2 * i];
        w_o.coefficients[2 * i] += w_r.coefficients[2 * i];
        w_o.coefficients[2 * i] += Fr::one();
        q_l.coefficients[2 * i] = Fr::one();
        q_r.coefficients[2 * i] = Fr::one();
        q_o.coefficients[2 * i] = -Fr::one();
        q_c.coefficients[2 * i] = Fr::one();
        q_m.coefficients[2 * i] = Fr::one();

        w_l.coefficients[2 * i + 1] = Fr::rand(&mut rand);
        w_r.coefficients[2 * i + 1] = Fr::rand(&mut rand);
        w_o.coefficients[2 * i + 1] = Fr::rand(&mut rand);

        t0 = w_l.coefficients[2 * i + 1] + w_r.coefficients[2 * i + 1];
        q_c[2 * i + 1] = t0 + w_o[2 * i + 1];
        q_c[2 * i + 1] = -q_c[2 * i + 1];
        q_l[2 * i + 1] = Fr::one();
        q_r[2 * i + 1] = Fr::one();
        q_o[2 * i + 1] = Fr::one();
        q_m[2 * i + 1] = Fr::zero();
    }

    let shift = n / 2;
    w_l.coefficients[shift..].copy_from_slice(&w_l.coefficients[..shift]);
    w_r.coefficients[shift..].copy_from_slice(&w_r.coefficients[..shift]);
    w_o.coefficients[shift..].copy_from_slice(&w_o.coefficients[..shift]);
    q_m.coefficients[shift..].copy_from_slice(&q_m.coefficients[..shift]);
    q_l.coefficients[shift..].copy_from_slice(&q_l.coefficients[..shift]);
    q_r.coefficients[shift..].copy_from_slice(&q_r.coefficients[..shift]);
    q_o.coefficients[shift..].copy_from_slice(&q_o.coefficients[..shift]);
    q_c.coefficients[shift..].copy_from_slice(&q_c.coefficients[..shift]);

    let mut sigma_1_mapping: Vec<u32> = vec![0; n];
    let mut sigma_2_mapping: Vec<u32> = vec![0; n];
    let mut sigma_3_mapping: Vec<u32> = vec![0; n];

    // create basic permutation - second half of witness vector is a copy of the first half
    for i in 0..(n / 2) {
        sigma_1_mapping[shift + i] = i as u32;
        sigma_2_mapping[shift + i] = (i as u32) + (1 << 30);
        sigma_3_mapping[shift + i] = (i as u32) + (1 << 31);
        sigma_1_mapping[i] = (i + shift) as u32;
        sigma_2_mapping[i] = ((i + shift) as u32) + (1 << 30);
        sigma_3_mapping[i] = ((i + shift) as u32) + (1 << 31);
    }

    // make last permutation the same as identity permutation
    // we are setting the permutation in the last 4 gates as identity permutation since
    // we are cutting out 4 roots as of now.

    let num_roots_cut_out_of_the_vanishing_polynomial = 4;
    for j in 0..num_roots_cut_out_of_the_vanishing_polynomial {
        let index = (shift - 1 - j) as u32;
        sigma_1_mapping[shift - 1 - j] = index;
        sigma_2_mapping[shift - 1 - j] = index + (1 << 30);
        sigma_3_mapping[shift - 1 - j] = index + (1 << 31);
        sigma_1_mapping[n - 1 - j] = (n - 1 - j) as u32;
        sigma_2_mapping[n - 1 - j] = ((n - 1 - j) as u32) + (1 << 30);
        sigma_3_mapping[n - 1 - j] = ((n - 1 - j) as u32) + (1 << 31);
    }

    let mut sigma_1 = Polynomial::new(key.circuit_size);
    let mut sigma_2 = Polynomial::new(key.circuit_size);
    let mut sigma_3 = Polynomial::new(key.circuit_size);

    compute_permutation_lagrange_base_single(&mut sigma_1, &sigma_1_mapping, &key.small_domain);
    compute_permutation_lagrange_base_single(&mut sigma_2, &sigma_2_mapping, &key.small_domain);
    compute_permutation_lagrange_base_single(&mut sigma_3, &sigma_3_mapping, &key.small_domain);

    let sigma_1_lagrange_base = sigma_1.clone();
    let sigma_2_lagrange_base = sigma_2.clone();
    let sigma_3_lagrange_base = sigma_3.clone();

    key.polynomial_store
        .insert(&"sigma_1_lagrange".to_string(), sigma_1_lagrange_base);
    key.polynomial_store
        .insert(&"sigma_2_lagrange".to_string(), sigma_2_lagrange_base);
    key.polynomial_store
        .insert(&"sigma_3_lagrange".to_string(), sigma_3_lagrange_base);

    key.small_domain.ifft_inplace(&mut sigma_1);
    key.small_domain.ifft_inplace(&mut sigma_2);
    key.small_domain.ifft_inplace(&mut sigma_3);

    const WIDTH: usize = 4;
    let sigma_1_fft = Polynomial::new_from(sigma_1, key.circuit_size * WIDTH);
    let sigma_2_fft = Polynomial::new_from(sigma_2, key.circuit_size * WIDTH);
    let sigma_3_fft = Polynomial::new_from(sigma_3, key.circuit_size * WIDTH);

    sigma_1_fft.coset_fft(&key.large_domain);
    sigma_2_fft.coset_fft(&key.large_domain);
    sigma_3_fft.coset_fft(&key.large_domain);

    key.polynomial_store.insert(&"sigma_1".to_string(), sigma_1);
    key.polynomial_store.insert(&"sigma_2".to_string(), sigma_2);
    key.polynomial_store.insert(&"sigma_3".to_string(), sigma_3);

    key.polynomial_store
        .insert(&"sigma_1_fft".to_string(), sigma_1_fft);
    key.polynomial_store
        .insert(&"sigma_2_fft".to_string(), sigma_2_fft);
    key.polynomial_store
        .insert(&"sigma_3_fft".to_string(), sigma_3_fft);

    key.polynomial_store
        .insert(&"w_1_lagrange".to_string(), w_l);
    key.polynomial_store
        .insert(&"w_2_lagrange".to_string(), w_r);
    key.polynomial_store
        .insert(&"w_3_lagrange".to_string(), w_o);

    key.small_domain.ifft_inplace(&mut q_l);
    key.small_domain.ifft_inplace(&mut q_r);
    key.small_domain.ifft_inplace(&mut q_o);
    key.small_domain.ifft_inplace(&mut q_m);
    key.small_domain.ifft_inplace(&mut q_c);

    let q_1_fft = Polynomial::new_from(q_l, n * 4);
    let q_2_fft = Polynomial::new_from(q_r, n * 4);
    let q_3_fft = Polynomial::new_from(q_o, n * 4);
    let q_m_fft = Polynomial::new_from(q_m, n * 4);
    let q_c_fft = Polynomial::new_from(q_c, n * 4);

    q_1_fft.coset_fft(&key.large_domain);
    q_2_fft.coset_fft(&key.large_domain);
    q_3_fft.coset_fft(&key.large_domain);
    q_m_fft.coset_fft(&key.large_domain);
    q_c_fft.coset_fft(&key.large_domain);

    key.polynomial_store.insert(&"q_1".to_string(), q_l);
    key.polynomial_store.insert(&"q_2".to_string(), q_r);
    key.polynomial_store.insert(&"q_3".to_string(), q_o);
    key.polynomial_store.insert(&"q_m".to_string(), q_m);
    key.polynomial_store.insert(&"q_c".to_string(), q_c);

    key.polynomial_store.insert(&"q_1_fft".to_string(), q_1_fft);
    key.polynomial_store.insert(&"q_2_fft".to_string(), q_2_fft);
    key.polynomial_store.insert(&"q_3_fft".to_string(), q_3_fft);
    key.polynomial_store.insert(&"q_m_fft".to_string(), q_m_fft);
    key.polynomial_store.insert(&"q_c_fft".to_string(), q_c_fft);

    let permutation_widget: Box<ProverPermutationWidget<'_>> =
        Box::new(ProverPermutationWidget::<3>::new(key.clone()));

    let widget: Box<ProverArithmeticWidget<'_, StandardSettings>> = Box::new(
        ProverArithmeticWidget::<_, StandardSettings>::new(key.clone()),
    );

    let kate_commitment_scheme = Box::new(KateCommitmentScheme::<StandardSettings>::new());

    let state = Prover::new(
        key,
        Some(ComposerType::StandardComposer::create_manifest(0)),
        None,
    );
    state.random_widgets.push(permutation_widget);
    state.transition_widgets.push(widget);
    state.commitment_scheme = kate_commitment_scheme;
    state
}

use std::{rc::Rc, cell::RefCell};

use crate::{
    ecc::{reduced_ate_pairing_batch_precomputed, PippengerRuntimeState},
    plonk::{
        composer::composer_base::ComposerType,
        proof_system::{
            commitment_scheme::KateCommitmentScheme,
            constants::NUM_LIMB_BITS_IN_FIELD_SIMULATION,
            prover::Prover,
            proving_key::ProvingKey,
            types::prover_settings::StandardSettings,
            utils::permutation::compute_permutation_lagrange_base_single,
            widgets::{
                random_widgets::permutation_widget::ProverPermutationWidget,
                transition_widgets::arithmetic_widget::ProverArithmeticWidget,
            },
        },
    },
    polynomials::Polynomial,
    srs::reference_string::file_reference_string::FileReferenceString,
    transcript::Transcript,
};

#[test]
fn verify_arithmetic_proof_small() {
    let n = 8;

    let state = generate_test_data(n);
    let verifier = Verifier::generate_verifier(&state.key);

    // Construct proof
    let proof = state.construct_proof();

    // Verify proof
    let result = verifier.verify_proof(&proof).unwrap();

    assert!(result);
}

#[test]
fn verify_arithmetic_proof() {
    let n = 1 << 14;

    let state = generate_test_data(n);
    let verifier = Verifier::generate_verifier(&state.key);

    // Construct proof
    let proof = state.construct_proof();

    // Verify proof
    let result = verifier.verify_proof(&proof).unwrap();

    assert!(result);
}

#[test]
#[should_panic]
fn verify_damaged_proof() {
    let n = 8;

    let state = generate_test_data(n);
    let verifier = Verifier::generate_verifier(&state.key);

    // Create empty proof
    let proof = Proof::default();

    // Verify proof
    verifier.verify_proof(&proof).unwrap();
}
