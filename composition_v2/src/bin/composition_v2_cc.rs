use libafl_cc::{ClangWrapper, CompilerWrapper, LLVMPasses};
use std::env;

#[cfg(feature = "cov_accounting")]
const GRANULARITY: &str = "FUNC";

pub fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let mut dir = env::current_exe().unwrap();
        let wrapper_name = dir.file_name().unwrap().to_str().unwrap();

        let is_cpp = match wrapper_name[wrapper_name.len()-2..].to_lowercase().as_str() {
            "cc" => false,
            "++" | "pp" | "xx" => true,
            _ => panic!("Could not figure out if c or c++ warpper was called. Expected {:?} to end with c or cxx", dir),
        };

        dir.pop();

        let mut cc = ClangWrapper::new();

        #[cfg(target_os = "linux")]
        cc.add_pass(LLVMPasses::AutoTokens);


        
        let compiler = cc
            .cpp(is_cpp)
            // silence the compiler wrapper output, needed for some configure scripts.
            .silence(true)
            // add arguments only if --libafl or --libafl-no-link are present
            .need_libafl_arg(true)
            .parse_args(&args)
            .expect("Failed to parse the command line")
            .link_staticlib(&dir, env!("CARGO_PKG_NAME"));
        
        #[cfg(any(feature = "ngram4", feature = "ngram8"))]
        #[cfg(any(feature = "cmplog", feature = "value_profile"))]
        compiler.add_arg("-fsanitize-coverage=trace-cmp");

        #[cfg(not(any(feature = "ngram4", feature = "ngram8")))]
        #[cfg(any(feature = "cmplog", feature = "value_profile"))]
        compiler.add_arg("-fsanitize-coverage=trace-pc-guard,trace-cmp");

        #[cfg(not(any(feature = "ngram4", feature = "ngram8")))]
        #[cfg(not(any(feature = "cmplog", feature = "value_profile")))]
        compiler.add_arg("-fsanitize-coverage=trace-pc-guard");

        #[cfg(feature = "cmplog")]
        compiler.add_pass(LLVMPasses::CmpLogRtn);

        #[cfg(feature = "cov_accounting")]
        {
            compiler.add_pass(LLVMPasses::CoverageAccounting);
            compiler.add_passes_arg(format!("-granularity={}", GRANULARITY));
        }

        #[cfg(feature = "ngram4")]
        {
            compiler.add_pass(LLVMPasses::AFLCoverage)
                .add_passes_arg("-ngram")
                .add_passes_arg("4")
                .add_passes_linking_arg("-lm");
        }

        #[cfg(feature = "ngram8")]
        {
            compiler.add_pass(LLVMPasses::AFLCoverage)
                .add_passes_arg("-ngram")
                .add_passes_arg("8")
                .add_passes_linking_arg("-lm");
        }

        if let Some(code) = compiler
            .run()
            .expect("Failed to run the wrapped compiler")
        {
            std::process::exit(code);
        }
    } else {
        panic!("LibAFL CC: No Arguments given");
    }
}
