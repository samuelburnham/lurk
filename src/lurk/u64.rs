use std::array;
use std::borrow::{Borrow, BorrowMut};

use p3_air::AirBuilder;
use p3_field::PrimeField32;

use crate::{
    air::builder::{LookupBuilder, Record, RequireRecord},
    gadgets::{
        bytes::{builder::BytesAirRecordWithContext, record::DummyBytesRecord},
        unsigned::{
            add::{Diff, Sum},
            div_rem::DivRem,
            less_than::IsLessThan,
            mul::Product,
            Word64,
        },
    },
    lair::{chipset::Chipset, execute::QueryRecord},
};

pub type Sum64<T> = Sum<T, 8>;
pub type Diff64<T> = Diff<T, 8>;
pub type DivRem64<T> = DivRem<T, 8>;
pub type IsLessThan64<T> = IsLessThan<T, 8>;
pub type Product64<T> = Product<T, 8>;

#[derive(Clone)]
pub enum U64 {
    Add,
    Sub,
    Mul,
    DivRem,
    LessThan,
}

fn into_u64<F: PrimeField32>(slice: &[F]) -> u64 {
    assert_eq!(slice.len(), 8);
    let buf: [u8; 8] = array::from_fn(|i| slice[i].as_canonical_u32().try_into().unwrap());
    u64::from_le_bytes(buf)
}

impl<F: PrimeField32> Chipset<F> for U64 {
    fn input_size(&self) -> usize {
        16
    }

    fn output_size(&self) -> usize {
        match self {
            U64::DivRem => 16,  // returns (quot, rem)
            U64::LessThan => 1, // returns one bool
            _ => 8,
        }
    }

    fn witness_size(&self) -> usize {
        match self {
            U64::Add => Sum64::<F>::witness_size(),
            U64::Sub => Diff64::<F>::witness_size(),
            U64::Mul => Product64::<F>::witness_size(),
            U64::DivRem => DivRem64::<F>::witness_size(),
            U64::LessThan => IsLessThan64::<F>::witness_size(),
        }
    }

    fn require_size(&self) -> usize {
        match self {
            U64::Add => Sum64::<F>::num_requires(),
            U64::Sub => Diff64::<F>::num_requires(),
            U64::Mul => Product64::<F>::num_requires(),
            U64::DivRem => DivRem64::<F>::num_requires(),
            U64::LessThan => IsLessThan64::<F>::num_requires(),
        }
    }

    fn execute(
        &self,
        input: &[F],
        nonce: u32,
        queries: &mut QueryRecord<F>,
        requires: &mut Vec<Record>,
    ) -> Vec<F> {
        let in1 = into_u64(&input[0..8]);
        let in2 = into_u64(&input[8..16]);
        let bytes = &mut queries.bytes.context(nonce, requires);
        match self {
            U64::Add => {
                let mut witness = Sum64::<F>::default();
                witness.populate(&in1, &in2, bytes);
                witness.iter_result().into_iter().collect()
            }
            U64::Sub => {
                let mut witness = Diff64::<F>::default();
                witness.populate(&in1, &in2, bytes);
                witness.iter_result().into_iter().collect()
            }
            U64::Mul => {
                let mut witness = Product64::<F>::default();
                witness.populate(&in1, &in2, bytes);
                witness.iter_result().into_iter().collect()
            }
            U64::DivRem => {
                let mut witness = DivRem64::<F>::default();
                witness.populate(&in1, &in2, bytes);
                witness.iter_result().into_iter().collect()
            }
            U64::LessThan => {
                let mut witness = IsLessThan64::<F>::default();
                witness.populate_less_than(&in1, &in2, bytes);
                witness.iter_result().into_iter().collect()
            }
        }
    }

    fn populate_witness(&self, input: &[F], witness: &mut [F]) -> Vec<F> {
        let in1 = into_u64(&input[0..8]);
        let in2 = into_u64(&input[8..16]);
        let bytes = &mut DummyBytesRecord;
        match self {
            U64::Add => {
                let witness: &mut Sum64<F> = witness.borrow_mut();
                witness.populate(&in1, &in2, bytes);
                witness.iter_result().into_iter().collect()
            }
            U64::Sub => {
                let witness: &mut Diff64<F> = witness.borrow_mut();
                witness.populate(&in1, &in2, bytes);
                witness.iter_result().into_iter().collect()
            }
            U64::Mul => {
                let witness: &mut Product64<F> = witness.borrow_mut();
                witness.populate(&in1, &in2, bytes);
                witness.iter_result().into_iter().collect()
            }
            U64::DivRem => {
                let witness: &mut DivRem64<F> = witness.borrow_mut();
                witness.populate(&in1, &in2, bytes);
                witness.iter_result().into_iter().collect()
            }
            U64::LessThan => {
                let witness: &mut IsLessThan64<F> = witness.borrow_mut();
                witness.populate_less_than(&in1, &in2, bytes);
                witness.iter_result().into_iter().collect()
            }
        }
    }

    fn eval<AB: AirBuilder<F = F> + LookupBuilder>(
        &self,
        builder: &mut AB,
        is_real: AB::Expr,
        ins: Vec<AB::Expr>,
        witness: &[AB::Var],
        nonce: AB::Expr,
        requires: &[RequireRecord<AB::Var>],
    ) -> Vec<AB::Expr> {
        let in1 = ins[0..8].iter().cloned().collect::<Word64<_>>();
        let in2 = ins[8..16].iter().cloned().collect::<Word64<_>>();
        let mut air_record = BytesAirRecordWithContext::default();
        let out = match self {
            U64::Add => {
                let witness: &Sum64<AB::Var> = witness.borrow();
                let out = witness.eval(builder, in1, in2, &mut air_record, is_real.clone());
                out.map(Into::into).into_iter().collect()
            }
            U64::Sub => {
                let witness: &Diff64<AB::Var> = witness.borrow();
                let out = witness.eval(builder, in1, in2, &mut air_record, is_real.clone());
                out.map(Into::into).into_iter().collect()
            }
            U64::Mul => {
                let witness: &Product64<AB::Var> = witness.borrow();
                let out = witness.eval(builder, &in1, &in2, &mut air_record, is_real.clone());
                out.map(Into::into).into_iter().collect()
            }
            U64::DivRem => {
                let witness: &DivRem64<AB::Var> = witness.borrow();
                let (q, r) = witness.eval(builder, &in1, &in2, &mut air_record, is_real.clone());
                [q, r].into_iter().flatten().map(Into::into).collect()
            }
            U64::LessThan => {
                let witness: &IsLessThan64<AB::Var> = witness.borrow();
                let out =
                    witness.eval_less_than(builder, &in1, &in2, &mut air_record, is_real.clone());
                vec![out]
            }
        };
        air_record.require_all(builder, nonce, requires.iter().cloned());
        out
    }
}

#[cfg(test)]
mod test {
    use p3_baby_bear::BabyBear as F;
    use p3_field::AbstractField;
    use sphinx_core::{stark::StarkMachine, utils::BabyBearPoseidon2};

    use crate::{
        air::debug::debug_chip_constraints_and_queries_with_sharding,
        func,
        lair::{
            execute::{QueryRecord, Shard},
            func_chip::FuncChip,
            lair_chip::{build_chip_vector, build_lair_chip_vector, LairMachineProgram},
            toplevel::Toplevel,
        },
        lurk::chipset::lurk_chip_map,
    };

    #[test]
    fn u64_add_test() {
        sphinx_core::utils::setup_logger();

        let add_func = func!(
        fn add(a: [8], b: [8]): [8] {
            let c: [8] = extern_call(u64_add, a, b);
            return c
        });
        let lurk_chip_map = lurk_chip_map();
        let toplevel = Toplevel::new(&[add_func], lurk_chip_map);

        let add_chip = FuncChip::from_name("add", &toplevel);
        let mut queries = QueryRecord::new(&toplevel);
        let f = F::from_canonical_usize;
        // Little endian
        let args = &[
            f(200),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
            //
            f(56),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
        ];
        let out = toplevel.execute_by_name("add", args, &mut queries);
        assert_eq!(
            out.as_ref(),
            &[f(0), f(1), f(0), f(0), f(0), f(0), f(0), f(0)]
        );

        let lair_chips = build_lair_chip_vector(&add_chip);
        debug_chip_constraints_and_queries_with_sharding(&queries, &lair_chips, None);

        let config = BabyBearPoseidon2::new();
        let machine = StarkMachine::new(
            config,
            build_chip_vector(&add_chip),
            queries.expect_public_values().len(),
        );

        let (pk, _vk) = machine.setup(&LairMachineProgram);
        let shard = Shard::new(&queries);
        machine.debug_constraints(&pk, shard.clone());
    }

    #[test]
    fn u64_sub_test() {
        sphinx_core::utils::setup_logger();

        let sub_func = func!(
        fn sub(a: [8], b: [8]): [8] {
            let c: [8] = extern_call(u64_sub, a, b);
            return c
        });
        let lurk_chip_map = lurk_chip_map();
        let toplevel = Toplevel::new(&[sub_func], lurk_chip_map);

        let sub_chip = FuncChip::from_name("sub", &toplevel);
        let mut queries = QueryRecord::new(&toplevel);
        let f = F::from_canonical_usize;
        // Little endian
        let args = &[
            f(0),
            f(1),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
            //
            f(1),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
        ];
        let out = toplevel.execute_by_name("sub", args, &mut queries);
        assert_eq!(
            out.as_ref(),
            &[f(255), f(0), f(0), f(0), f(0), f(0), f(0), f(0)]
        );

        let lair_chips = build_lair_chip_vector(&sub_chip);
        debug_chip_constraints_and_queries_with_sharding(&queries, &lair_chips, None);

        let config = BabyBearPoseidon2::new();
        let machine = StarkMachine::new(
            config,
            build_chip_vector(&sub_chip),
            queries.expect_public_values().len(),
        );

        let (pk, _vk) = machine.setup(&LairMachineProgram);
        let shard = Shard::new(&queries);
        machine.debug_constraints(&pk, shard.clone());
    }

    #[test]
    fn u64_mul_test() {
        sphinx_core::utils::setup_logger();

        let mul_func = func!(
        fn mul(a: [8], b: [8]): [8] {
            let c: [8] = extern_call(u64_mul, a, b);
            let d: [8] = extern_call(u64_mul, c, c);
            let e: [8] = extern_call(u64_mul, d, d);
            let f: [8] = extern_call(u64_mul, e, e);
            let g: [8] = extern_call(u64_mul, f, f);
            return g
        });
        let lurk_chip_map = lurk_chip_map();
        let toplevel = Toplevel::new(&[mul_func], lurk_chip_map);

        let mul_chip = FuncChip::from_name("mul", &toplevel);
        let mut queries = QueryRecord::new(&toplevel);
        let f = F::from_canonical_usize;
        // Little endian
        let args = &[
            f(2),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
            //
            f(2),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
        ];
        let out = toplevel.execute_by_name("mul", args, &mut queries);
        assert_eq!(
            out.as_ref(),
            &[f(0), f(0), f(0), f(0), f(1), f(0), f(0), f(0)]
        );

        let lair_chips = build_lair_chip_vector(&mul_chip);
        debug_chip_constraints_and_queries_with_sharding(&queries, &lair_chips, None);

        let config = BabyBearPoseidon2::new();
        let machine = StarkMachine::new(
            config,
            build_chip_vector(&mul_chip),
            queries.expect_public_values().len(),
        );

        let (pk, _vk) = machine.setup(&LairMachineProgram);
        let shard = Shard::new(&queries);
        machine.debug_constraints(&pk, shard.clone());
    }

    #[test]
    fn u64_divrem_test() {
        sphinx_core::utils::setup_logger();

        let divrem_func = func!(
        fn divrem(a: [8], b: [8]): [16] {
            let (div: [8], rem: [8]) = extern_call(u64_divrem, a, b);
            return (div, rem)
        });
        let lurk_chip_map = lurk_chip_map();
        let toplevel = Toplevel::new(&[divrem_func], lurk_chip_map);

        let divrem_chip = FuncChip::from_name("divrem", &toplevel);
        let mut queries = QueryRecord::new(&toplevel);
        let f = F::from_canonical_usize;
        // Little endian
        let args = &[
            f(0),
            f(0),
            f(1),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
            //
            f(7),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
        ];
        let out = toplevel.execute_by_name("divrem", args, &mut queries);
        assert_eq!(
            out.as_ref(),
            &[
                f(146),
                f(36),
                f(0),
                f(0),
                f(0),
                f(0),
                f(0),
                f(0),
                //
                f(2),
                f(0),
                f(0),
                f(0),
                f(0),
                f(0),
                f(0),
                f(0)
            ]
        );

        let lair_chips = build_lair_chip_vector(&divrem_chip);
        debug_chip_constraints_and_queries_with_sharding(&queries, &lair_chips, None);

        let config = BabyBearPoseidon2::new();
        let machine = StarkMachine::new(
            config,
            build_chip_vector(&divrem_chip),
            queries.expect_public_values().len(),
        );

        let (pk, _vk) = machine.setup(&LairMachineProgram);
        let shard = Shard::new(&queries);
        machine.debug_constraints(&pk, shard.clone());
    }

    #[test]
    fn u64_lessthan_test() {
        sphinx_core::utils::setup_logger();

        let lessthan_func = func!(
        fn lessthan(a: [8], b: [8]): [1] {
            let c = extern_call(u64_lessthan, a, b);
            return c
        });
        let lurk_chip_map = lurk_chip_map();
        let toplevel = Toplevel::new(&[lessthan_func], lurk_chip_map);

        let lessthan_chip = FuncChip::from_name("lessthan", &toplevel);
        let mut queries = QueryRecord::new(&toplevel);
        let f = F::from_canonical_usize;
        // Little endian
        let args = &[
            f(200),
            f(200),
            f(200),
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
            //
            f(0),
            f(0),
            f(0),
            f(0),
            f(0),
            f(10),
            f(0),
            f(0),
        ];
        let out = toplevel.execute_by_name("lessthan", args, &mut queries);
        assert_eq!(out.as_ref(), &[f(1)]);

        let lair_chips = build_lair_chip_vector(&lessthan_chip);
        debug_chip_constraints_and_queries_with_sharding(&queries, &lair_chips, None);

        let config = BabyBearPoseidon2::new();
        let machine = StarkMachine::new(
            config,
            build_chip_vector(&lessthan_chip),
            queries.expect_public_values().len(),
        );

        let (pk, _vk) = machine.setup(&LairMachineProgram);
        let shard = Shard::new(&queries);
        machine.debug_constraints(&pk, shard.clone());
    }
}