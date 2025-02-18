// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the "hack" directory of this source tree.

use std::ops::ControlFlow;

use naming_special_names_rust as sn;
use oxidized::aast_defs::FunParam;
use oxidized::aast_defs::Fun_;
use oxidized::aast_defs::Method_;
use oxidized::naming_error::NamingError;
use oxidized::naming_phase_error::NamingPhaseError;

use crate::config::Config;
use crate::Pass;

#[derive(Clone, Copy, Default)]
pub struct ValidateFunParamsPass;

impl Pass for ValidateFunParamsPass {
    fn on_ty_fun__top_down<Ex: Default, En>(
        &mut self,
        elem: &mut Fun_<Ex, En>,
        _: &Config,
        errs: &mut Vec<NamingPhaseError>,
    ) -> ControlFlow<(), ()> {
        self.validate_fun_params(&elem.params, errs)
    }

    fn on_ty_method__top_down<Ex: Default, En>(
        &mut self,
        elem: &mut Method_<Ex, En>,
        _: &Config,
        errs: &mut Vec<NamingPhaseError>,
    ) -> ControlFlow<(), ()> {
        self.validate_fun_params(&elem.params, errs)
    }
}

impl ValidateFunParamsPass {
    fn validate_fun_params<Ex: Default, En>(
        &self,
        params: &Vec<FunParam<Ex, En>>,
        errs: &mut Vec<NamingPhaseError>,
    ) -> ControlFlow<(), ()> {
        let mut seen = std::collections::BTreeSet::<&String>::new();
        for FunParam { name, pos, .. } in params {
            if name == sn::special_idents::PLACEHOLDER {
                continue;
            } else if seen.contains(name) {
                errs.push(NamingPhaseError::Naming(NamingError::AlreadyBound {
                    pos: pos.clone(),
                    name: name.clone(),
                }));
            } else {
                seen.insert(name);
            }
        }
        ControlFlow::Continue(())
    }
}

#[cfg(test)]
mod tests {

    use oxidized::aast::Block;
    use oxidized::aast::FuncBody;
    use oxidized::aast::TypeHint;
    use oxidized::aast::UserAttributes;
    use oxidized::aast::Visibility;
    use oxidized::aast_defs::Pos;
    use oxidized::ast::FunKind;
    use oxidized::ast::Id;
    use oxidized::ast::ParamKind;

    use super::*;
    use crate::transform::Transform;

    fn mk_fun(params: Vec<FunParam<(), ()>>) -> Fun_<(), ()> {
        Fun_ {
            span: Pos::NONE,
            readonly_this: None,
            annotation: (),
            readonly_ret: None,
            ret: TypeHint((), None),
            tparams: vec![],
            where_constraints: vec![],
            params,
            ctxs: None,
            unsafe_ctxs: None,
            body: FuncBody {
                fb_ast: Block(vec![]),
            },
            fun_kind: FunKind::FSync,
            user_attributes: UserAttributes(vec![]),
            external: false,
            doc_comment: None,
        }
    }

    fn mk_method(name: String, params: Vec<FunParam<(), ()>>) -> Method_<(), ()> {
        Method_ {
            span: Pos::NONE,
            annotation: (),
            final_: false,
            abstract_: false,
            static_: true,
            readonly_this: false,
            visibility: Visibility::Public,
            name: Id(Pos::NONE, name),
            tparams: vec![],
            where_constraints: vec![],
            params,
            ctxs: None,
            unsafe_ctxs: None,
            body: FuncBody {
                fb_ast: Block(vec![]),
            },
            fun_kind: FunKind::FSync,
            user_attributes: UserAttributes(vec![]),
            readonly_ret: None,
            ret: TypeHint((), None),
            external: false,
            doc_comment: None,
        }
    }

    fn mk_param(name: String) -> FunParam<(), ()> {
        FunParam {
            name,
            annotation: (),
            type_hint: TypeHint((), None),
            is_variadic: false,
            pos: Pos::NONE,
            expr: None,
            readonly: None,
            callconv: ParamKind::Pnormal,
            user_attributes: UserAttributes(Vec::default()),
            visibility: Some(Visibility::Public),
        }
    }

    #[test]
    fn test_fn_no_args() {
        let cfg = Config::default();
        let mut errs: Vec<NamingPhaseError> = Vec::default();
        let mut pass = ValidateFunParamsPass;

        let mut fun = mk_fun(vec![]);

        fun.transform(&cfg, &mut errs, &mut pass);
        assert!(errs.is_empty())
    }

    #[test]
    fn test_meth_no_args() {
        let cfg = Config::default();
        let mut errs: Vec<NamingPhaseError> = Vec::default();
        let mut pass = ValidateFunParamsPass;

        let mut meth = mk_method("foo".to_string(), vec![]);

        meth.transform(&cfg, &mut errs, &mut pass);
        assert!(errs.is_empty())
    }

    #[test]
    fn test_fn_good_args() {
        let cfg = Config::default();
        let mut errs: Vec<NamingPhaseError> = Vec::default();
        let mut pass = ValidateFunParamsPass;

        let x = mk_param("x".to_string());
        let y = mk_param("y".to_string());
        let mut fun = mk_fun(vec![x, y]);

        fun.transform(&cfg, &mut errs, &mut pass);
        assert!(errs.is_empty())
    }

    #[test]
    fn test_meth_good_args() {
        let cfg = Config::default();
        let mut errs: Vec<NamingPhaseError> = Vec::default();
        let mut pass = ValidateFunParamsPass;

        let x = mk_param("x".to_string());
        let y = mk_param("y".to_string());
        let mut meth = mk_method("foo".to_string(), vec![x, y]);

        meth.transform(&cfg, &mut errs, &mut pass);
        assert!(errs.is_empty())
    }

    #[test]
    fn test_fn_args_multiply_bound() {
        let cfg = Config::default();
        let mut errs: Vec<NamingPhaseError> = Vec::default();
        let mut pass = ValidateFunParamsPass;

        let x = mk_param("x".to_string());
        let mut fun = mk_fun(vec![x.clone(), x]);

        fun.transform(&cfg, &mut errs, &mut pass);
        assert!(matches!(
            &errs[..],
            &[NamingPhaseError::Naming(NamingError::AlreadyBound { .. })]
        ))
    }

    #[test]
    fn test_meth_args_multiply_bound() {
        let cfg = Config::default();
        let mut errs: Vec<NamingPhaseError> = Vec::default();
        let mut pass = ValidateFunParamsPass;

        let x = mk_param("x".to_string());
        let mut meth = mk_method("foo".to_string(), vec![x.clone(), x]);

        meth.transform(&cfg, &mut errs, &mut pass);
        assert!(matches!(
            &errs[..],
            &[NamingPhaseError::Naming(NamingError::AlreadyBound { .. })]
        ))
    }
}
