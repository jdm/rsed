// Copyright 2012-2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern mod syntax;
extern mod rustc;
//extern mod extra;

//use rustc;
use self::rustc::{driver/*, middle*/};

use self::syntax::ast;
use self::syntax::diagnostic;
use self::syntax::fold::ast_fold;
use self::syntax::parse;
use self::syntax::print::pprust;
//use self::syntax;

use std::io;
use std::os;
use std::vec;

use visit::Renamer;

pub mod visit;

pub struct RSedContext {
    crate: ast::Crate,
    /*tycx: middle::ty::ctxt,*/
    sess: driver::session::Session
}

local_data_key!(pub ctxtkey: @RSedContext)

/// Parses, resolves, and typechecks the given crate
fn get_ast_and_resolve(cpath: &Path, libs: ~[Path]) -> RSedContext {
    use syntax::codemap::dummy_spanned;
    use rustc::driver::driver::*;

    let parsesess = parse::new_parse_sess(None);
    let input = file_input(cpath.clone());

    let sessopts = @driver::session::options {
        binary: @"rsed",
        maybe_sysroot: Some(@os::self_exe_path().unwrap().pop()),
        addl_lib_search_paths: @mut libs,
        .. (*rustc::driver::session::basic_options()).clone()
    };


    let diagnostic_handler = syntax::diagnostic::mk_handler(None);
    let span_diagnostic_handler =
        syntax::diagnostic::mk_span_handler(diagnostic_handler, parsesess.cm);

    let sess = driver::driver::build_session_(sessopts,
                                              parsesess.cm,
                                              @diagnostic::DefaultEmitter as
                                                @diagnostic::Emitter,
                                              span_diagnostic_handler);

    let mut cfg = build_configuration(sess);
    cfg.push(@dummy_spanned(ast::MetaWord(@"stage2")));

    let crate = phase_1_parse_input(sess, cfg.clone(), &input);
    /*crate = phase_2_configure_and_expand(sess, cfg, crate);
    let analysis = phase_3_run_analysis_passes(sess, &crate);*/

    //debug!("crate: %?", crate);
    RSedContext { crate: crate, /*tycx: analysis.ty_cx,*/ sess: sess }
}

pub fn run(libs: ~[Path], path: &Path, from: ~[&str], to: &str) {
    let ctxt = get_ast_and_resolve(path, libs);

    debug!("%? -> %?", from, to);
    
    /*debug!("defmap:");
    for (k, v) in ctxt.tycx.def_map.iter() {
        debug!("%?: %?", k, v);
    }*/
    //local_data::set(ctxtkey, ctxt);

    let renamer = Renamer::new(from.map(|x| x.to_owned()), to);
    let new_crate = renamer.fold_crate(ctxt.crate.clone());
    pprust::print_crate(ctxt.sess.codemap,
                        ctxt.sess.intr(),
                        ctxt.sess.span_diagnostic,
                        &new_crate,
                        path.to_str().to_managed(),
                        io::file_reader(path).unwrap(),
                        io::stdout(),
                        @pprust::no_ann::new() as @pprust::pp_ann,
                        false);
}

pub fn main() {
    let args = os::args();
    match args {
        [_, crate, from, to, libs] => {
            let modules: ~[&str] = from.split_str_iter("::").collect();
            let components: ~[&str] = do vec::flat_map(modules) |x| {
                x.clone().split_iter('.').collect()
            };
            run(~[Path(libs.clone())], &Path(crate.clone()), components, to);
        }
        _ => fail!("usage: crate.rs pattern name path/to/libs")
    }
}