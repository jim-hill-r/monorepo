//! Ahead-of-time (AOT) compilation of my_lang to native code.
//!
//! [`compile_to_object`] turns my_lang source into a native object file for
//! the host machine, which a C toolchain can link like any other object.
//! [`build_executable`] goes one step further and links a standalone
//! executable whose entry point is the my_lang function `main`.
//!
//! ## Executables
//!
//! The program's `main` must take no parameters. The executable calls it,
//! prints its `i64` result followed by a newline on standard output, and
//! exits with status 0. Linking uses the system C compiler: the `CC`
//! environment variable if set, otherwise `cc`.
//!
//! ## Example
//!
//! ```ignore
//! use std::path::Path;
//! use my_lang::aot;
//!
//! aot::build_executable("function main() { return 6 * 7 }", Path::new("answer"))
//!     .expect("build should succeed");
//! // ./answer prints "42".
//! ```

use std::path::{Path, PathBuf};
use std::process::Command;

use inkwell::module::{Linkage, Module};
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::{AddressSpace, OptimizationLevel, context::Context};

use crate::codegen::{CodeGenerator, CodegenError};
use crate::lexer::Lexer;
use crate::parser::Parser;

/// The my_lang function an executable starts in.
pub const ENTRY_FUNCTION: &str = "main";

/// The name the my_lang `main` is given in an executable, so the C-style
/// `main` wrapper can take its place.
const RENAMED_ENTRY: &str = "my_lang_main";

/// Errors that can occur during AOT compilation.
#[derive(Debug)]
pub enum AotError {
    /// The source code could not be parsed.
    ParseError(String),
    /// Code generation failed.
    CodegenError(CodegenError),
    /// The host target could not be set up for code generation.
    Target(String),
    /// The program has no entry function with this name.
    EntryNotFound(String),
    /// The entry function takes parameters, but an executable has none to
    /// pass it.
    EntryHasParameters {
        /// The entry function's name.
        name: String,
        /// How many parameters it declares.
        count: usize,
    },
    /// The generated LLVM module failed verification.
    Verify(String),
    /// LLVM could not write the object file.
    Emit(String),
    /// The linker could not be run, or it failed.
    Link(String),
}

impl std::fmt::Display for AotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AotError::ParseError(msg) => write!(f, "parse error: {msg}"),
            AotError::CodegenError(e) => write!(f, "codegen error: {e}"),
            AotError::Target(msg) => write!(f, "target setup failed: {msg}"),
            AotError::EntryNotFound(name) => {
                write!(f, "no entry function '{name}' in the program")
            }
            AotError::EntryHasParameters { name, count } => write!(
                f,
                "entry function '{name}' must take no parameters, but takes {count}"
            ),
            AotError::Verify(msg) => write!(f, "invalid LLVM module: {msg}"),
            AotError::Emit(msg) => write!(f, "could not write object file: {msg}"),
            AotError::Link(msg) => write!(f, "link failed: {msg}"),
        }
    }
}

impl std::error::Error for AotError {}

impl From<CodegenError> for AotError {
    fn from(e: CodegenError) -> Self {
        AotError::CodegenError(e)
    }
}

/// Compile `source` to a native object file at `path`, for the host machine.
///
/// Every my_lang function becomes an exported symbol with the C calling
/// convention, taking and returning `i64` (`long long` in C).
///
/// # Errors
///
/// Returns an [`AotError`] if parsing, code generation, verification or
/// writing the object file fails.
pub fn compile_to_object(source: &str, path: &Path) -> Result<(), AotError> {
    let context = Context::create();
    let module = compile_module(&context, source)?;
    emit_object(&module, path)
}

/// Compile `source` into a native executable at `output`.
///
/// The executable runs the my_lang function `main`, prints its result and a
/// newline on standard output, and exits with status 0. The intermediate
/// object file is removed afterwards.
///
/// # Errors
///
/// Returns [`AotError::EntryNotFound`] if there is no `main`,
/// [`AotError::EntryHasParameters`] if `main` takes parameters,
/// [`AotError::Link`] if the C compiler can't be run or fails, or another
/// [`AotError`] if an earlier compilation step fails.
pub fn build_executable(source: &str, output: &Path) -> Result<(), AotError> {
    let context = Context::create();
    let module = compile_module(&context, source)?;
    add_entry_wrapper(&context, &module)?;

    let object = object_path_for(output);
    emit_object(&module, &object)?;
    let linked = link(&object, output);
    let _ = std::fs::remove_file(&object);
    linked
}

/// Lex, parse and generate LLVM IR for `source`.
fn compile_module<'ctx>(context: &'ctx Context, source: &str) -> Result<Module<'ctx>, AotError> {
    let mut parser = Parser::new(Lexer::new(source.to_string()));
    let program = parser
        .parse()
        .map_err(|e| AotError::ParseError(format!("{e:?}")))?;

    let mut codegen = CodeGenerator::new(context, "aot_module");
    codegen.compile_program(&program)?;
    Ok(codegen.take_module())
}

/// Rename the my_lang `main` and add a C `int main(void)` that calls it and
/// prints the result with `printf`.
fn add_entry_wrapper<'ctx>(context: &'ctx Context, module: &Module<'ctx>) -> Result<(), AotError> {
    let entry = module
        .get_function(ENTRY_FUNCTION)
        .ok_or_else(|| AotError::EntryNotFound(ENTRY_FUNCTION.to_string()))?;
    let count = entry.count_params() as usize;
    if count != 0 {
        return Err(AotError::EntryHasParameters {
            name: ENTRY_FUNCTION.to_string(),
            count,
        });
    }
    entry.as_global_value().set_name(RENAMED_ENTRY);

    let i32_type = context.i32_type();
    let ptr_type = context.ptr_type(AddressSpace::default());
    let printf = module.add_function(
        "printf",
        i32_type.fn_type(&[ptr_type.into()], true),
        Some(Linkage::External),
    );

    let main = module.add_function("main", i32_type.fn_type(&[], false), None);
    let builder = context.create_builder();
    builder.position_at_end(context.append_basic_block(main, "entry"));

    let build = |e: inkwell::builder::BuilderError| AotError::from(CodegenError::from(e));
    let result = builder
        .build_call(entry, &[], "result")
        .map_err(build)?
        .try_as_basic_value()
        .basic()
        .ok_or(AotError::CodegenError(CodegenError::ExpressionWithoutValue))?;
    let format = builder
        .build_global_string_ptr("%lld\n", "result_format")
        .map_err(build)?;
    builder
        .build_call(
            printf,
            &[format.as_pointer_value().into(), result.into()],
            "printed",
        )
        .map_err(build)?;
    builder
        .build_return(Some(&i32_type.const_zero()))
        .map_err(build)?;
    Ok(())
}

/// Verify `module` and write it as a host object file to `path`.
fn emit_object(module: &Module<'_>, path: &Path) -> Result<(), AotError> {
    Target::initialize_native(&InitializationConfig::default()).map_err(AotError::Target)?;
    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).map_err(|e| AotError::Target(e.to_string()))?;
    let machine = target
        .create_target_machine(
            &triple,
            &TargetMachine::get_host_cpu_name().to_string(),
            &TargetMachine::get_host_cpu_features().to_string(),
            OptimizationLevel::Default,
            // Position-independent code links into PIE executables, the
            // default on most Linux distributions and on macOS.
            RelocMode::PIC,
            CodeModel::Default,
        )
        .ok_or_else(|| AotError::Target(format!("no target machine for {triple}")))?;

    module.set_triple(&triple);
    module.set_data_layout(&machine.get_target_data().get_data_layout());
    module
        .verify()
        .map_err(|e| AotError::Verify(e.to_string()))?;
    machine
        .write_to_file(module, FileType::Object, path)
        .map_err(|e| AotError::Emit(e.to_string()))
}

/// The intermediate object file for executable `output`: `output` with `.o`
/// added to its file name.
fn object_path_for(output: &Path) -> PathBuf {
    let mut name = output.file_name().unwrap_or_default().to_os_string();
    name.push(".o");
    output.with_file_name(name)
}

/// Link `object` into the executable `output` with the system C compiler.
fn link(object: &Path, output: &Path) -> Result<(), AotError> {
    let compiler = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
    let result = Command::new(&compiler)
        .arg(object)
        .arg("-o")
        .arg(output)
        .output()
        .map_err(|e| AotError::Link(format!("could not run '{compiler}': {e}")))?;
    if result.status.success() {
        Ok(())
    } else {
        Err(AotError::Link(format!(
            "'{compiler}' failed: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_path_appends_extension() {
        assert_eq!(
            object_path_for(Path::new("out/program")),
            PathBuf::from("out/program.o")
        );
        assert_eq!(
            object_path_for(Path::new("app.exe")),
            PathBuf::from("app.exe.o")
        );
    }

    #[test]
    fn errors_display_their_cause() {
        let err = AotError::EntryHasParameters {
            name: "main".to_string(),
            count: 2,
        };
        assert_eq!(
            err.to_string(),
            "entry function 'main' must take no parameters, but takes 2"
        );
    }
}
