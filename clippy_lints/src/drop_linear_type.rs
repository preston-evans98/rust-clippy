use std::cell::{LazyCell, OnceCell};
use std::sync::OnceLock;

use crate::rustc_lint::LintContext;
use clippy_utils::diagnostics::span_lint_and_help;
use clippy_utils::ty::implements_trait;
use clippy_utils::{get_trait_def_id, match_trait_method, paths};
use rustc_hir::def_id::DefId;
use rustc_hir::*;
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::mir::{Location, TerminatorKind};
use rustc_session::declare_lint_pass;

static NEARLY_LINEAR_DEF_ID: OnceLock<Option<DefId>> = OnceLock::new();

declare_clippy_lint! {
    /// ### What it does
    ///
    /// ### Why is this bad?
    ///
    /// ### Example
    /// ```no_run
    /// // example code where clippy issues a warning
    /// ```
    /// Use instead:
    /// ```no_run
    /// // example code which does not raise clippy warning
    /// ```
    #[clippy::version = "1.84.0"]
    pub DROP_LINEAR_TYPE,
    suspicious,
    "Check for unconsumed 'linear types', which expect to be consumed rather than dropped"
}

declare_lint_pass!(DropLinearType => [DROP_LINEAR_TYPE]);

impl LateLintPass<'_> for DropLinearType {
    fn check_body(&mut self, cx: &LateContext<'_>, body: &Body<'_>) {
        // Find the MIR for the owner of this body. The owner is the function/closure/const
        // whose body this belongs to. https://rustc-dev-guide.rust-lang.org/hir.html#hir-bodies
        let mir = cx.tcx.optimized_mir(cx.tcx.hir().body_owner_def_id(body.id()));

        // Iterate through all basic blocks in the MIR
        for (_bb, bb_data) in mir.basic_blocks.iter().enumerate() {
            // Check the terminator of each basic block
            if bb_data.is_cleanup {
                continue;
            }
            if let Some(terminator) = &bb_data.terminator {
                if let TerminatorKind::Drop { place, .. } = terminator.kind {
                    let ty = place.ty(mir, cx.tcx).ty;
                    if let Some(nearly_linear) =
                        NEARLY_LINEAR_DEF_ID.get_or_init(|| get_trait_def_id(cx.tcx, &paths::NEARLY_LINEAR_PATH))
                    {
                        if implements_trait(cx, ty, *nearly_linear, &[]) {
                            if let Some(decl) = place.as_local().map(|local| mir.local_decls.get(local)).flatten() {
                                let decl_span = decl.source_info.span;
                                cx.span_lint(DROP_LINEAR_TYPE, decl_span, |diag| {
                                    diag.primary_message("dropping an item that should be used");
                                    diag.help("this item should always be consumed before going out of scope. You may have forgotten to call a function that consumes this value.");
                                    diag.span_note(terminator.source_info.span, "Item is dropped here without being used");
                                    diag.note("If you're sure it's safe to drop this value without using it, call `NearlyLinear::done()` on the value before exiting the scope.");
                                });
                            } else {
                                span_lint_and_help(
                                    cx,
                                    DROP_LINEAR_TYPE,
                                    terminator.source_info.span,
                                    "dropping a type that should be used",
                                    None,
                                    "One of the types in this scope was supposed to be used but was dropped. This lint *should* show you which variable was dropped, but was unable to do so because of a bug in the linter. Please report this error.",
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}
