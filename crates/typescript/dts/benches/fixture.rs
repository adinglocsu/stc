#![feature(box_syntax)]
#![feature(test)]

extern crate test;

use rnode::RNode;
use slog::Logger;
use stc_ts_ast_rnode::RModule;
use stc_ts_dts::cleanup_module_for_dts;
use stc_ts_file_analyzer::analyzer::Analyzer;
use stc_ts_file_analyzer::analyzer::NoopLoader;
use stc_ts_file_analyzer::env::Env;
use stc_ts_file_analyzer::mode::Single;
use stc_ts_file_analyzer::validator::ValidateWith;
use stc_ts_file_analyzer::Lib;
use stc_ts_types::module_id;
use stc_ts_utils::StcComments;
use std::{
    fs::read_to_string,
    hint::black_box,
    path::{Path, PathBuf},
    sync::Arc,
};
use swc_common::input::SourceFileInput;
use swc_common::GLOBALS;
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::Parser;
use swc_ecma_parser::Syntax;
use swc_ecma_parser::{JscTarget, TsConfig};
use swc_ecma_transforms::resolver::ts_resolver;
use swc_ecma_visit::FoldWith;
use test::Bencher;

#[bench]
fn bench_001(b: &mut Bencher) {
    run_bench(
        b,
        Path::new("tests")
            .join("fixture")
            .join("classes")
            .join("mixin")
            .join("access-modifiers")
            .join("test-1-2-4.ts"),
    );
}

#[bench]
fn bench_002(b: &mut Bencher) {
    run_bench(
        b,
        Path::new("tests")
            .join("fixture")
            .join("expression")
            .join("type-guards")
            .join("this")
            .join("test-1-1.ts"),
    );
}

#[bench]
fn bench_003(b: &mut Bencher) {
    run_bench(
        b,
        Path::new("tests")
            .join("fixture")
            .join("key-of")
            .join("indexed-access")
            .join("test-6-9.ts"),
    );
}

#[bench]
fn bench_004(b: &mut Bencher) {
    run_bench(
        b,
        Path::new("tests")
            .join("fixture")
            .join("libs")
            .join("rxjs")
            .join("internal")
            .join("util")
            .join("Immediate-1-1.ts"),
    );
}

#[bench]
fn bench_005(b: &mut Bencher) {
    fixture(b, "types::conditional::infer::test-2::index.ts");
}

#[bench]
fn bench_006(b: &mut Bencher) {
    fixture(b, "types::generic::context::inference::test-2-4.ts");
}

#[bench]
fn bench_007(b: &mut Bencher) {
    fixture(b, "libs::rxjs::pipe.ts");
}

#[bench]
fn bench_008(b: &mut Bencher) {
    fixture(b, "types::generic::context::inference::test-4.ts");
}

#[bench]
fn bench_009(b: &mut Bencher) {
    fixture(b, "types::mapped::inference::test-1-4.ts");
}

#[bench]
fn bench_010(b: &mut Bencher) {
    fixture(b, "types::mapped::inference::test-1-6-1.ts");
}

#[bench]
fn bench_011(b: &mut Bencher) {
    fixture(b, "types::mapped::types::array-tuple::test-5-1-3.ts");
}

#[bench]
fn bench_012(b: &mut Bencher) {
    fixture(b, "types::mapped::types::array-tuple::test-5-1-4.ts");
}

#[bench]
fn bench_013(b: &mut Bencher) {
    fixture(b, "types::mapped::types::array-tuple::test-5-1-5.ts");
}

#[bench]
fn bench_014(b: &mut Bencher) {
    fixture(b, "types::mapped::types::array-tuple::test-5-1.ts");
}

#[bench]
fn bench_015(b: &mut Bencher) {
    fixture(b, "types::mapped::types::array-tuple::test-1.ts");
}

#[bench]
fn bench_016(b: &mut Bencher) {
    fixture(b, "types::mapped::types::conformance-2::test-1.ts");
}

#[bench]
fn bench_017(b: &mut Bencher) {
    fixture(b, "types::mapped::types::array-tuple::test-5.ts");
}

fn fixture(b: &mut Bencher, s: &str) {
    let mut path = PathBuf::new().join("tests").join("fixture");
    for s in s.split("::") {
        path.push(s);
    }

    run_bench(b, path)
}

fn run_bench(b: &mut Bencher, path: PathBuf) {
    let size = read_to_string(&path).unwrap().as_bytes().len();
    b.bytes = size as _;

    ::testing::run_test2(false, |cm, _| {
        let fm = cm.load_file(&path).unwrap();
        let env = Env::simple(
            Default::default(),
            JscTarget::Es2020,
            &Lib::load("es2020.full"),
        );
        let stable_env = env.shared().clone();
        let generator = module_id::Generator::default();
        let path = Arc::new(path.clone());

        let comments = StcComments::default();

        let lexer = Lexer::new(
            Syntax::Typescript(TsConfig {
                tsx: path.to_string_lossy().contains("tsx"),
                decorators: true,
                ..Default::default()
            }),
            JscTarget::Es2020,
            SourceFileInput::from(&*fm),
            Some(&comments),
        );
        let mut parser = Parser::new_from(lexer);
        let module = parser.parse_module().unwrap();
        let module = GLOBALS.set(stable_env.swc_globals(), || {
            module.fold_with(&mut ts_resolver(stable_env.marks().top_level_mark()))
        });
        let module = RModule::from_orig(module);

        b.iter(|| {
            let mut storage = Single {
                parent: None,
                id: generator.generate(&path).1,
                path: path.clone(),
                info: Default::default(),
            };

            let mut module = module.clone();
            {
                let mut analyzer = Analyzer::root(
                    Logger::root(slog::Discard, slog::o!()),
                    env.clone(),
                    cm.clone(),
                    box &mut storage,
                    &NoopLoader,
                );
                GLOBALS.set(stable_env.swc_globals(), || {
                    module.validate_with(&mut analyzer).unwrap();
                });
            }

            {
                cleanup_module_for_dts(&mut module.body, &storage.info.exports);
            }

            black_box(storage);
            black_box(module);
        });

        Ok(())
    })
    .unwrap();
}