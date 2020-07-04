//! FIXME: write short doc here

use crate::completion::{CompletionContext, Completions};

/// Completes constats and paths in patterns.
pub(super) fn complete_pattern(acc: &mut Completions, ctx: &CompletionContext) {
    if !ctx.is_pat_binding_or_const {
        return;
    }
    if ctx.record_pat_syntax.is_some() {
        return;
    }

    // FIXME: ideally, we should look at the type we are matching against and
    // suggest variants + auto-imports
    ctx.scope().process_all_names(&mut |name, res| {
        match &res {
            hir::ScopeDef::ModuleDef(def) => match def {
                hir::ModuleDef::Adt(hir::Adt::Enum(..))
                | hir::ModuleDef::Adt(hir::Adt::Struct(..))
                | hir::ModuleDef::EnumVariant(..)
                | hir::ModuleDef::Const(..)
                | hir::ModuleDef::Module(..) => (),
                _ => return,
            },
            hir::ScopeDef::MacroDef(_) => (),
            _ => return,
        };

        acc.add_resolution(ctx, name.to_string(), &res)
    });
}

#[cfg(test)]
mod tests {
    use crate::completion::{test_utils::do_completion, CompletionItem, CompletionKind};
    use insta::assert_debug_snapshot;

    fn complete(code: &str) -> Vec<CompletionItem> {
        do_completion(code, CompletionKind::Reference)
    }

    #[test]
    fn completes_enum_variants_and_modules() {
        let completions = complete(
            r"
            enum E { X }
            use self::E::X;
            const Z: E = E::X;
            mod m {}

            static FOO: E = E::X;
            struct Bar { f: u32 }

            fn foo() {
               match E::X {
                   <|>
               }
            }
            ",
        );
        assert_debug_snapshot!(completions, @r###"
        [
            CompletionItem {
                label: "Bar",
                source_range: 137..137,
                delete: 137..137,
                insert: "Bar",
                kind: Struct,
            },
            CompletionItem {
                label: "E",
                source_range: 137..137,
                delete: 137..137,
                insert: "E",
                kind: Enum,
            },
            CompletionItem {
                label: "X",
                source_range: 137..137,
                delete: 137..137,
                insert: "X",
                kind: EnumVariant,
                detail: "()",
            },
            CompletionItem {
                label: "Z",
                source_range: 137..137,
                delete: 137..137,
                insert: "Z",
                kind: Const,
            },
            CompletionItem {
                label: "m",
                source_range: 137..137,
                delete: 137..137,
                insert: "m",
                kind: Module,
            },
        ]
        "###);
    }

    #[test]
    fn completes_in_simple_macro_call() {
        let completions = complete(
            r"
            macro_rules! m { ($e:expr) => { $e } }
            enum E { X }

            fn foo() {
               m!(match E::X {
                   <|>
               })
            }
            ",
        );
        assert_debug_snapshot!(completions, @r###"
        [
            CompletionItem {
                label: "E",
                source_range: 90..90,
                delete: 90..90,
                insert: "E",
                kind: Enum,
            },
            CompletionItem {
                label: "m!(…)",
                source_range: 90..90,
                delete: 90..90,
                insert: "m!($0)",
                kind: Macro,
                lookup: "m!",
                detail: "macro_rules! m",
            },
        ]
        "###);
    }
}
