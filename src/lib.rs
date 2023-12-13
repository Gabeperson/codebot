use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::files::Files;
use codespan_reporting::files::SimpleFiles;
use numbat::diagnostic::ErrorDiagnostic;
use numbat::markup as m;
use numbat::module_importer::BuiltinModuleImporter;
use numbat::pretty_print::PrettyPrint;
use numbat::resolver::CodeSource;
use numbat::Context;
use numbat::InterpreterSettings;
use std::sync::{Arc, Mutex};

pub fn run(s: String) -> String {
    let mut context = Context::new(BuiltinModuleImporter::default());
    context.load_currency_module_on_demand(true);
    let to_be_printed: Arc<Mutex<Vec<m::Markup>>> = Arc::new(Mutex::new(vec![]));
    let tbc = to_be_printed.clone();
    let mut settings = InterpreterSettings {
        print_fn: Box::new(move |s: &m::Markup| {
            tbc.lock().unwrap().push(s.clone());
        }),
    };
    _ = context
        .interpret_with_settings(&mut settings, "use prelude", CodeSource::Internal)
        .unwrap();
    let (result, registry) = {
        let registry = context.dimension_registry().clone();
        (
            context.interpret_with_settings(&mut settings, &s, CodeSource::Text),
            registry,
        )
    };

    match result {
        Ok((statements, interp_result)) => {
            let tbp = to_be_printed.lock().unwrap();
            let mut to_return = String::new();
            for s in tbp.iter() {
                to_return.push_str(&s.to_string());
                to_return.push('\n');
                // println!("{}", s.to_string());
            }
            // return to_return;
            let result_markup = interp_result.to_markup(statements.last(), &registry, false);
            let expr = result_markup.to_string();
            if !expr.trim().is_empty() {
                let last_statement = statements
                    .last()
                    .and_then(|s| s.as_expression())
                    .map(|e| e.get_type())
                    .map(|t| t.to_string())
                    .unwrap_or("".into());
                let r = statements.last().unwrap().as_expression().unwrap();
                let pp = r.pretty_print();
                to_return.push_str(&format!(
                    "\n`{pp}` = {} [{}]",
                    result_markup.to_string().trim(),
                    last_statement
                ));
            }
            to_return
        }
        Err(e) => {
            let files = &context.resolver().files;
            // context.print_diagnostic(e);
            // println!("{}", e.to_string());
            fn handle_error(
                d: &Diagnostic<usize>,
                files: &SimpleFiles<String, String>,
                s: &str,
                e: String,
            ) -> String {
                let label = &d.labels[0];
                let location = files
                    .location(label.file_id, label.range.start)
                    .expect("This should exist");
                let offending_line = s.lines().nth(location.line_number - 1).unwrap();
                let num_spaces = location.column_number - 1;
                let spaces = String::from_utf8(vec![b' '; num_spaces]).unwrap();
                format!(
                    "{}:{}: error :\n{}\n{}^\n{}",
                    location.line_number, location.column_number, offending_line, spaces, e
                )
            }
            // e.to_string()
            match e {
                numbat::NumbatError::ResolverError(e) => {
                    let diagnostic = &e.diagnostics()[0];
                    handle_error(diagnostic, files, &s, e.to_string())
                }
                numbat::NumbatError::NameResolutionError(e) => {
                    let diagnostic = &e.diagnostics()[0];
                    handle_error(diagnostic, files, &s, e.to_string())
                }
                numbat::NumbatError::TypeCheckError(e) => {
                    let diagnostic = &e.diagnostics()[0];
                    handle_error(diagnostic, files, &s, e.to_string())
                }
                numbat::NumbatError::RuntimeError(e) => {
                    let diagnostic = &e.diagnostics()[0];
                    handle_error(diagnostic, files, &s, e.to_string())
                }
            }
        }
    }
}
