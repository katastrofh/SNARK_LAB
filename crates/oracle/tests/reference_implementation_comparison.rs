#![forbid(unsafe_code)]

use ark_bls12_381::Fr;
use ark_ff::Field;
use multilinear::Multilinear;
use snark_lab_oracle::{
    compute_ipa_evaluation_basis, evaluate_with_ipa_evaluation_basis, fold_ipa_evaluation_vector,
    fold_ipa_polynomial_vector, MultilinearOracle, TransparentOracle,
};

fn one() -> Fr {
    Fr::from(1u64)
}

fn deterministic_table(variables: usize) -> Vec<Fr> {
    let len = 1usize << variables;

    (0..len)
        .map(|i| {
            let x = i as u64;
            Fr::from(x * x + 3 * x + 7)
        })
        .collect()
}

fn deterministic_point(variables: usize) -> Vec<Fr> {
    (0..variables)
        .map(|i| Fr::from((2 * i + 3) as u64))
        .collect()
}

fn reference_evaluation_basis(point: &[Fr]) -> Vec<Fr> {
    let len = 1usize << point.len();
    let mut basis = Vec::with_capacity(len);

    for mask in 0..len {
        let mut weight = one();

        for (variable, coordinate) in point.iter().enumerate() {
            let bit_is_one = ((mask >> variable) & 1) == 1;

            if bit_is_one {
                weight *= *coordinate;
            } else {
                weight *= one() - *coordinate;
            }
        }

        basis.push(weight);
    }

    basis
}

fn reference_multilinear_evaluate(table: &[Fr], point: &[Fr]) -> Fr {
    assert_eq!(table.len(), 1usize << point.len());

    table
        .iter()
        .zip(reference_evaluation_basis(point))
        .map(|(evaluation, basis_value)| *evaluation * basis_value)
        .sum()
}

fn reference_polynomial_fold(values: &[Fr], challenge: Fr) -> Vec<Fr> {
    assert!(values.len() >= 2);
    assert_eq!(values.len() % 2, 0);

    let inverse = challenge.inverse().expect("nonzero challenge");
    let half = values.len() / 2;

    values[..half]
        .iter()
        .zip(values[half..].iter())
        .map(|(left, right)| challenge * *left + inverse * *right)
        .collect()
}

fn reference_evaluation_fold(values: &[Fr], challenge: Fr) -> Vec<Fr> {
    assert!(values.len() >= 2);
    assert_eq!(values.len() % 2, 0);

    let inverse = challenge.inverse().expect("nonzero challenge");
    let half = values.len() / 2;

    values[..half]
        .iter()
        .zip(values[half..].iter())
        .map(|(left, right)| inverse * *left + challenge * *right)
        .collect()
}

#[test]
fn multilinear_evaluation_matches_independent_reference() {
    for variables in 0..=6 {
        let table = deterministic_table(variables);
        let point = deterministic_point(variables);
        let polynomial = Multilinear::new(table.clone()).expect("valid multilinear table");

        let production_value = polynomial.evaluate(&point).expect("production evaluation");
        let reference_value = reference_multilinear_evaluate(&table, &point);

        assert_eq!(production_value, reference_value, "variables={variables}");
    }
}

#[test]
fn ipa_evaluation_basis_matches_independent_reference() {
    for variables in 0..=6 {
        let point = deterministic_point(variables);

        let production_basis =
            compute_ipa_evaluation_basis(&point).expect("production IPA evaluation basis");
        let reference_basis = reference_evaluation_basis(&point);

        assert_eq!(
            production_basis.basis_evaluations, reference_basis,
            "variables={variables}"
        );
    }
}

#[test]
fn ipa_evaluation_inner_product_matches_independent_reference() {
    for variables in 0..=6 {
        let table = deterministic_table(variables);
        let point = deterministic_point(variables);
        let polynomial = Multilinear::new(table.clone()).expect("valid multilinear table");

        let production_value = evaluate_with_ipa_evaluation_basis(&polynomial, &point)
            .expect("production IPA-basis evaluation");
        let reference_value = reference_multilinear_evaluate(&table, &point);

        assert_eq!(production_value, reference_value, "variables={variables}");
    }
}

#[test]
fn ipa_polynomial_vector_fold_matches_independent_reference() {
    let challenge = Fr::from(11u64);

    for variables in 1..=6 {
        let values = deterministic_table(variables);

        let production_fold =
            fold_ipa_polynomial_vector(&values, challenge).expect("production polynomial fold");
        let reference_fold = reference_polynomial_fold(&values, challenge);

        assert_eq!(production_fold, reference_fold, "variables={variables}");
    }
}

#[test]
fn ipa_evaluation_vector_fold_matches_independent_reference() {
    let challenge = Fr::from(11u64);

    for variables in 1..=6 {
        let point = deterministic_point(variables);
        let values = reference_evaluation_basis(&point);

        let production_fold =
            fold_ipa_evaluation_vector(&values, challenge).expect("production evaluation fold");
        let reference_fold = reference_evaluation_fold(&values, challenge);

        assert_eq!(production_fold, reference_fold, "variables={variables}");
    }
}

#[test]
fn transparent_oracle_opening_matches_independent_reference() {
    for variables in 0..=6 {
        let table = deterministic_table(variables);
        let point = deterministic_point(variables);
        let polynomial = Multilinear::new(table.clone()).expect("valid multilinear table");
        let oracle = TransparentOracle::new(polynomial);

        let commitment = oracle.commit();
        let opening = oracle.open(&point).expect("transparent opening");
        let verified = TransparentOracle::verify_opening(&commitment, &point, &opening)
            .expect("transparent verification");

        let reference_value = reference_multilinear_evaluate(&table, &point);

        assert_eq!(opening.value, reference_value, "variables={variables}");
        assert_eq!(verified, reference_value, "variables={variables}");
    }
}
