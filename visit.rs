use syntax::ast;
use syntax::ast::{Ident, item, Path};
use syntax::fold;
use syntax::parse::token::{ident_to_str, str_to_ident};

use std::str;

struct RenamerData {
    current_target: uint,
    targets: ~[~str],
    to: ~str,
    rename_next_ident: bool,
    rename_eventual_ident: bool,
    finished: bool
}

pub struct Renamer {
    state: @mut RenamerData
}

impl Renamer {
    pub fn new(from: ~[~str], to: &str) -> Renamer {
        Renamer {
            state: @mut RenamerData {
                current_target: 1, //skip top-level crate
                targets: from,
                to: to.to_owned(),
                rename_next_ident: false,
                rename_eventual_ident: false,
                finished: false
            }
        }
    }
}

fn ident_eq(s: &str, i: &Ident) -> bool {
    str::eq_slice(s, ident_to_str(i))
}

impl fold::ast_fold for Renamer {
    fn fold_ident(&self, i: Ident) -> Ident {
        if self.state.rename_next_ident ||
           (self.state.rename_eventual_ident && ident_eq(*self.state.targets.last(), &i)) {
            self.state.rename_next_ident = false;
            self.state.rename_eventual_ident = false;
            self.state.finished = true;
            str_to_ident(self.state.to)
        } else {
            i
        }
    }

    fn fold_item(&self, i: @item) -> Option<@item> {
        if !self.state.finished &&
           ident_eq(self.state.targets[self.state.current_target], &i.ident) {
            debug!("found a chain item");
            self.state.current_target += 1;
            if self.state.current_target == self.state.targets.len() {
                debug!("renaming ident");
                self.state.rename_next_ident = true;
            }
        }
        fold::noop_fold_item(i, self)
    }

    fn fold_path(&self, p: &Path) -> Path {
        let path_matches = {
            let n = (self.state.targets.len() - p.segments.len()).max(&0);
            let mut zipped = p.segments.iter().zip(self.state.targets.iter().skip(n));
            zipped.all(|(seg, target)| ident_eq(*target, &seg.identifier))
        };

        self.state.rename_eventual_ident = path_matches;

        ast::Path {
            span: self.new_span(p.span),
            global: p.global,
            segments: p.segments.map(|segment| ast::PathSegment {
                identifier: self.fold_ident(segment.identifier),
                lifetime: segment.lifetime,
                types: segment.types.map(|typ| self.fold_ty(typ)),
            })
        }
    }
}
