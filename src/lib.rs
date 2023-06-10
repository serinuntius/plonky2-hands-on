#[cfg(test)]
mod tests {
    use plonky2::{
        field::{goldilocks_field::GoldilocksField, types::Field},
        iop::{
            target::Target,
            witness::{PartialWitness, WitnessWrite},
        },
        plonk::{
            circuit_builder::CircuitBuilder,
            circuit_data::{CircuitConfig, CircuitData},
            config::PoseidonGoldilocksConfig,
            proof::ProofWithPublicInputs,
        },
    };

    type F = GoldilocksField;
    // ２次の拡大体を使う
    const D: usize = 2;

    // HashにPoseidonを利用して、proofをつくる
    type C = PoseidonGoldilocksConfig;

    #[test]
    fn test_plonky2_add() {
        // 回路のサイズや各種設定が入る構造体
        let config = CircuitConfig::standard_recursion_config();
        // 回路の制約を扱う
        let mut builder = CircuitBuilder::<F, D>::new(config);

        let one = F::from_canonical_u64(1);
        let two = F::from_canonical_u64(2);
        // let three = F::from_canonical_u64(3);

        // 回路内の空の変数を定義する(wire)
        // まだテーブルの位置が決まってないので、仮の位置を指定する
        // 最終的にはテーブルの中の位置が固定され、Wireになる
        let a = builder.add_virtual_target();
        let b = builder.add_virtual_target();

        // a + b = c
        let c = builder.add(a, b);

        // targetにwitness(値)を割り当てる
        let mut pw = PartialWitness::new();
        // a = 1
        pw.set_target(a, one);

        // b = 1
        pw.set_target(b, one);

        // c = 2(PartialWitnessで本来は自動で計算してくれるが、明示的に設定する)
        pw.set_target(c, two);

        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();

        // 検証
        data.verify(proof).unwrap();
    }

    struct InnerTarget {
        a: Target,
        b: Target,
        c: Target,
    }

    // inner circuit(再起証明の対象になる回路)を生成する関数
    fn build_inner_circuit() -> (CircuitData<F, C, D>, InnerTarget) {
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(config);

        let one = F::from_canonical_u64(1);
        let two = F::from_canonical_u64(2);
        let a = builder.add_virtual_target();
        let b = builder.add_virtual_target();

        let c = builder.add(a, b);

        let data = builder.build::<C>();
        let target = InnerTarget { a, b, c };
        (data, target)
    }

    fn generate_inner_proof(
        data: &CircuitData<F, C, D>,
        it: &InnerTarget,
    ) -> ProofWithPublicInputs<F, C, D> {
        let mut pw = PartialWitness::new();
        pw.set_target(it.a, F::from_canonical_u64(1));
        pw.set_target(it.b, F::from_canonical_u64(2));
        pw.set_target(it.c, F::from_canonical_u64(3));

        data.prove(pw).unwrap()
    }

    #[test]
    fn test_recursive_proof() {
        let (inner_data, inner_target) = build_inner_circuit();

        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(config);

        let inner_verifier_data = builder.constant_verifier_data(&inner_data.verifier_only);
        let proof_with_pis = builder.add_virtual_proof_with_pis(&inner_data.common);

        let inner_proof = generate_inner_proof(&inner_data, &inner_target);

        // recursion proof
        builder.verify_proof::<C>(&proof_with_pis, &inner_verifier_data, &inner_data.common);

        let mut pw = PartialWitness::<F>::new();

        pw.set_proof_with_pis_target(&proof_with_pis, &inner_proof);

        let data = builder.build::<C>();
        let proof = data.prove(pw).unwrap();

        data.verify(proof).unwrap();
    }
}
