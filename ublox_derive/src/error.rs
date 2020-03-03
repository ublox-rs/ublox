use std::{fmt::Write, path::Path};
use syn::Error;

pub fn panic_on_parse_error((src_path, src_cnt): (&Path, &str), err: &Error) -> ! {
    let span = err.span();
    let start = span.start();
    let end = span.end();

    let mut code_problem = String::new();
    let nlines = end.line - start.line + 1;
    for (i, line) in src_cnt
        .lines()
        .skip(start.line - 1)
        .take(nlines)
        .enumerate()
    {
        code_problem.push_str(&line);
        code_problem.push('\n');
        if i == 0 && start.column > 0 {
            write!(&mut code_problem, "{:1$}", ' ', start.column).expect("write to String failed");
        }
        let code_problem_len = if i == 0 {
            if i == nlines - 1 {
                end.column - start.column
            } else {
                line.len() - start.column - 1
            }
        } else if i != nlines - 1 {
            line.len()
        } else {
            end.column
        };
        writeln!(&mut code_problem, "{:^^1$}", '^', code_problem_len).expect("Not enought memory");
        if i == end.line {
            break;
        }
    }

    panic!(
        "parsing of {name} failed\nerror: {err}\n{code_problem}\nAt {name}:{line_s}:{col_s}",
        name = src_path.display(),
        err = err,
        code_problem = code_problem,
        line_s = start.line,
        col_s = start.column,
    );
}
