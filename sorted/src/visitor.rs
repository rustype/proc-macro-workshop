pub(crate) struct MatchVisitor {
    pub errors: Vec<syn::Error>,
}

impl MatchVisitor {
    pub fn new() -> Self {
        Self { errors: vec![] }
    }

    fn push_err(&mut self, err: syn::Error) {
        self.errors.push(err)
    }
}

impl syn::visit_mut::VisitMut for MatchVisitor {
    fn visit_expr_match_mut(&mut self, m: &mut syn::ExprMatch) {
        let attrs = &mut m.attrs;
        // short circuit and unwrap on not empty
        if attrs.is_empty() || !attrs.iter().any(is_valid_sorted) {
            return;
        }
        attrs.retain(|a| !a.path.is_ident("sorted"));
        // if match is not over a single expression
        // add error and return
        if !is_match_expr_path(&m.expr) {
            self.push_err(syn::Error::new_spanned(
                &m.expr,
                "sorted only supports a single expression in `match`",
            ));
            return;
        }
        if let syn::Result::Err(e) = are_arms_sorted(m.arms.iter()) {
            self.push_err(e);
        }
        // println!("{:#?}", m);
        syn::visit_mut::visit_expr_match_mut(self, m)
    }
}

fn is_valid_sorted(attrs: &syn::Attribute) -> bool {
    attrs.path.is_ident("sorted") && attrs.tokens.is_empty()
}

fn is_match_expr_path(expr: &syn::Expr) -> bool {
    if let syn::Expr::Path(_) = expr {
        true
    } else {
        false
    }
}

fn are_arms_sorted<'a, I>(arms: I) -> syn::Result<()>
where
    I: Iterator<Item = &'a syn::Arm>,
{
    let mut acc_ident = vec![];
    let mut wild = None;
    for pat in arms.map(|arm| &arm.pat) {
        println!("{:#?}", pat);
        if let Some(w) = wild {
            // in the case that `_ => {...}` is not the last arm
            // we return an error
            return syn::Result::Err(syn::Error::new_spanned(
                w,
                "_ should sort after all other arms",
            ));
        }
        match pat {
            syn::Pat::TupleStruct(syn::PatTupleStruct {
                path: syn::Path { segments, .. },
                ..
            }) => {
                // there should at least an ident so unwrap should be safe
                let last_seg_ident = join_segments_in_string(segments.pairs());
                for ident in acc_ident.iter() {
                    if ident > &last_seg_ident {
                        return syn::Result::Err(syn::Error::new_spanned(
                            segments,
                            format!("{} should sort before {}", last_seg_ident, ident),
                        ));
                    }
                }
                acc_ident.push(last_seg_ident);
            }
            syn::Pat::Wild(w) => wild = Some(w),
            _ => {
                return syn::Result::Err(syn::Error::new_spanned(pat, "unsupported by #[sorted]"));
            }
        }
    }

    syn::Result::Ok(())
}

fn join_segments_in_string<'a, I>(segments: I) -> String
where
    I: Iterator<Item = syn::punctuated::Pair<&'a syn::PathSegment, &'a syn::token::Colon2>>,
{
    let mut concat_segments = String::new();
    for pair in segments {
        // concat_segments += ;
        match pair {
            syn::punctuated::Pair::Punctuated(segment, _) => {
                concat_segments += format!("{}::", segment.ident.to_string()).as_str();
            }
            syn::punctuated::Pair::End(segment) => {
                concat_segments += format!("{}", segment.ident.to_string()).as_str();
            }
        }
    }
    concat_segments
}
