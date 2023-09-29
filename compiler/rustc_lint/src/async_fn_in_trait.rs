use crate::lints::AsyncFnInTraitDiag;
use crate::LateContext;
use crate::LateLintPass;
use rustc_hir as hir;
use rustc_trait_selection::traits::error_reporting::suggestions::suggest_desugaring_async_fn_to_impl_future_in_trait;

declare_lint! {
    /// The `async_fn_in_trait` lint detects use of `async fn` in the
    /// definition of a publicly-reachable trait.
    ///
    /// ### Example
    ///
    /// ```rust
    /// # #![feature(async_fn_in_trait)]
    /// pub trait Trait {
    ///     async fn method(&self);
    /// }
    /// # fn main() {}
    /// ```
    ///
    /// {{produces}}
    ///
    /// ### Explanation
    ///
    /// When `async fn` is used in a trait definition, the trait does not
    /// promise that the opaque [`Future`] returned by the associated function
    /// or method will implement any [auto traits] such as [`Send`]. This may
    /// be surprising and may make the associated functions or methods on the
    /// trait less useful than intended. On traits exposed publicly from a
    /// crate, this may affect downstream crates whose authors cannot alter
    /// the trait definition.
    ///
    /// For example, this code is invalid:
    ///
    /// ```rust,compile_fail
    /// # #![feature(async_fn_in_trait)]
    /// pub trait Trait {
    ///     async fn method(&self) {}
    /// }
    ///
    /// fn test<T: Trait>(x: T) {
    ///     fn is_send<T: Send>(_: T) {}
    ///     is_send(x.method()); // Not OK.
    /// }
    /// ```
    ///
    /// This lint exists to warn authors of publicly-reachable traits that
    /// they may want to consider desugaring the `async fn` to a normal `fn`
    /// that returns an opaque `impl Future<..> + Send` type.
    ///
    /// For example, instead of:
    ///
    /// ```rust
    /// # #![feature(async_fn_in_trait)]
    /// pub trait Trait {
    ///     async fn method(&self) {}
    /// }
    /// ```
    ///
    /// The author of the trait may want to write:
    ///
    ///
    /// ```rust
    /// # #![feature(return_position_impl_trait_in_trait)]
    /// use core::future::Future;
    /// pub trait Trait {
    ///     fn method(&self) -> impl Future<Output = ()> + Send { async {} }
    /// }
    /// ```
    ///
    /// Conversely, if the trait is used only locally, if only concrete types
    /// that implement the trait are used, or if the trait author otherwise
    /// does not care that the trait will not promise that the returned
    /// [`Future`] implements any [auto traits] such as [`Send`], then the
    /// lint may be suppressed.
    ///
    /// [`Future`]: https://doc.rust-lang.org/core/future/trait.Future.html
    /// [`Send`]: https://doc.rust-lang.org/core/marker/trait.Send.html
    /// [auto traits]: https://doc.rust-lang.org/reference/special-types-and-traits.html#auto-traits
    pub ASYNC_FN_IN_TRAIT,
    Warn,
    "use of `async fn` in definition of a publicly-reachable trait"
}

declare_lint_pass!(
    /// Lint for use of `async fn` in the definition of a publicly-reachable
    /// trait.
    AsyncFnInTrait => [ASYNC_FN_IN_TRAIT]
);

impl<'tcx> LateLintPass<'tcx> for AsyncFnInTrait {
    fn check_trait_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::TraitItem<'tcx>) {
        if let hir::TraitItemKind::Fn(sig, body) = item.kind
            && let hir::IsAsync::Async(async_span) = sig.header.asyncness
        {
            if cx.tcx.features().return_type_notation {
                return;
            }

            let hir::FnRetTy::Return(hir::Ty { kind: hir::TyKind::OpaqueDef(def, ..), .. }) =
                sig.decl.output
            else {
                // This should never happen, but let's not ICE.
                return;
            };
            let sugg = suggest_desugaring_async_fn_to_impl_future_in_trait(
                cx.tcx,
                sig,
                body,
                def.owner_id.def_id,
                " + Send",
            );
            cx.tcx.emit_spanned_lint(ASYNC_FN_IN_TRAIT, item.hir_id(), async_span, AsyncFnInTraitDiag {
                sugg
            });
        }
    }
}
