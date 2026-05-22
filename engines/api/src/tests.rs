/// Unit tests for pde-solver-api models and solver infrastructure.
///
/// These tests cover:
///   1. SolveRequest JSON deserialization (various PdeSpec shapes).
///   2. SolveResponse JSON serialization correctness.
///   3. ApiError status code mapping.
///   4. SolverRegistry — list / get / default_id.
///   5. Solver timeout environment variable parsing.

#[cfg(test)]
mod model_tests {
    use crate::models::{
        IcValue, PdeSpec, QuerySpec, SolveRequest, SolveResponse, SolverCategory, SolverInfo,
        SolutionShape, SolveMetadata,
    };

    fn minimal_pde() -> PdeSpec {
        PdeSpec {
            equation: "u_t = 0.1 * laplace(u)".to_string(),
            initial_condition: Some(vec![0.0; 64]),
            boundary_condition: Some("periodic".to_string()),
            parameters: None,
            variables: vec![],
            equations: vec![],
            initial_conditions: Default::default(),
            coef_fields: Default::default(),
            domains: vec![],
            bcs: vec![],
            history: None,
        }
    }

    fn minimal_query() -> QuerySpec {
        QuerySpec {
            x: vec![0.0, 0.5, 1.0],
            y: vec![0.0, 0.5, 1.0],
            t: Some(vec![0.0, 1.0]),
        }
    }

    // ── SolveRequest / PdeSpec deserialization ───────────────────────────────

    #[test]
    fn solve_request_minimal_roundtrip() {
        let req = SolveRequest {
            solver: None,
            pde: minimal_pde(),
            query: minimal_query(),
            options: None,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let decoded: SolveRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.pde.equation, req.pde.equation);
        assert!(decoded.solver.is_none());
    }

    #[test]
    fn solve_request_with_solver_field() {
        let json = r#"{"solver":"classical","pde":{"equation":"u_t=0","variables":[],"equations":[],"initial_conditions":{},"coef_fields":{},"domains":[],"bcs":[]},"query":{"x":[0.0],"y":[0.0],"t":null}}"#;
        let req: SolveRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.solver.as_deref(), Some("classical"));
    }

    #[test]
    fn pde_spec_default_vecs_are_empty() {
        let pde = minimal_pde();
        assert!(pde.variables.is_empty());
        assert!(pde.equations.is_empty());
        assert!(pde.initial_conditions.is_empty());
        assert!(pde.coef_fields.is_empty());
        assert!(pde.domains.is_empty());
        assert!(pde.bcs.is_empty());
        assert!(pde.history.is_none());
    }

    #[test]
    fn ic_value_array_variant_roundtrip() {
        let ic = IcValue::Array(vec![1.0, 2.0, 3.0]);
        let json = serde_json::to_string(&ic).unwrap();
        let decoded: IcValue = serde_json::from_str(&json).unwrap();
        if let IcValue::Array(vals) = decoded {
            assert_eq!(vals, vec![1.0, 2.0, 3.0]);
        } else {
            panic!("expected IcValue::Array");
        }
    }

    #[test]
    fn ic_value_token_variant_roundtrip() {
        let ic = IcValue::Token("zero".to_string());
        let json = serde_json::to_string(&ic).unwrap();
        let decoded: IcValue = serde_json::from_str(&json).unwrap();
        if let IcValue::Token(s) = decoded {
            assert_eq!(s, "zero");
        } else {
            panic!("expected IcValue::Token");
        }
    }

    #[test]
    fn solve_request_with_multi_variable_pde() {
        let json = r#"{
            "pde": {
                "equation": "",
                "variables": ["u", "v"],
                "equations": ["u_t = laplace(u)", "v_t = laplace(v)"],
                "initial_conditions": {"u": [0.0, 1.0], "v": [1.0, 0.0]},
                "coef_fields": {},
                "domains": [],
                "bcs": []
            },
            "query": {"x": [0.0, 1.0], "y": [0.0, 1.0], "t": null}
        }"#;
        let req: SolveRequest = serde_json::from_str(json).expect("deserialize multi-var");
        assert_eq!(req.pde.variables, vec!["u", "v"]);
        assert_eq!(req.pde.equations.len(), 2);
        assert!(req.pde.initial_conditions.contains_key("u"));
        assert!(req.pde.initial_conditions.contains_key("v"));
    }

    // ── SolveResponse serialization ──────────────────────────────────────────

    #[test]
    fn solve_response_serializes_correctly() {
        let resp = SolveResponse {
            solver_used: "classical".to_string(),
            variables: vec!["u".to_string()],
            solution: vec![vec![vec![vec![0.5_f64]]]],
            shape: SolutionShape {
                n_t: 1,
                n_x: 1,
                n_y: 1,
                n_vars: 1,
            },
            metadata: SolveMetadata {
                wall_time_ms: 42,
                backend: "py-pde 0.54 / FDM".to_string(),
                notes: vec!["test note".to_string()],
            },
        };
        let json = serde_json::to_string(&resp).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["solver_used"], "classical");
        assert_eq!(v["shape"]["n_t"], 1);
        assert_eq!(v["shape"]["n_vars"], 1);
        assert_eq!(v["metadata"]["wall_time_ms"], 42);
        assert_eq!(v["variables"][0], "u");
    }

    #[test]
    fn query_spec_optional_t_can_be_null() {
        let json = r#"{"x":[0.0],"y":[0.0],"t":null}"#;
        let q: QuerySpec = serde_json::from_str(json).unwrap();
        assert!(q.t.is_none());
    }

    // ── SolverInfo serialization ─────────────────────────────────────────────

    #[test]
    fn solver_info_category_serializes_snake_case() {
        let info = SolverInfo {
            id: "classical".to_string(),
            name: "Classical FDM".to_string(),
            category: SolverCategory::Classical,
            description: "test".to_string(),
            supported_pde_types: vec!["diffusion".to_string()],
            backend: "py-pde".to_string(),
            available: true,
        };
        let json = serde_json::to_string(&info).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["category"], "classical");
    }

    #[test]
    fn solver_category_ml_serializes_snake_case() {
        let cat = SolverCategory::MachineLearning;
        let json = serde_json::to_string(&cat).unwrap();
        assert_eq!(json, "\"machine_learning\"");
    }
}

#[cfg(test)]
mod registry_tests {
    use crate::solvers::{SolverRegistry};

    #[test]
    fn registry_default_id_is_pdeformer2() {
        assert_eq!(SolverRegistry::default_id(), "pdeformer2");
    }

    #[test]
    fn registry_lists_two_solvers() {
        let reg = SolverRegistry::new();
        let list = reg.list();
        assert_eq!(list.len(), 2, "expected classical + pdeformer2");
        let ids: Vec<&str> = list.iter().map(|s| s.id.as_str()).collect();
        assert!(ids.contains(&"classical"), "classical missing from registry");
        assert!(ids.contains(&"pdeformer2"), "pdeformer2 missing from registry");
    }

    #[test]
    fn registry_get_known_solver() {
        let reg = SolverRegistry::new();
        assert!(reg.get("classical").is_some());
        assert!(reg.get("pdeformer2").is_some());
    }

    #[test]
    fn registry_get_unknown_solver_returns_none() {
        let reg = SolverRegistry::new();
        assert!(reg.get("nonexistent_solver").is_none());
    }

    #[test]
    fn solver_info_classical_has_expected_fields() {
        let reg = SolverRegistry::new();
        let solver = reg.get("classical").unwrap();
        let info = solver.info();
        assert_eq!(info.id, "classical");
        assert!(info.available);
        assert!(!info.supported_pde_types.is_empty());
    }

    #[test]
    fn solver_info_pdeformer2_has_expected_fields() {
        let reg = SolverRegistry::new();
        let solver = reg.get("pdeformer2").unwrap();
        let info = solver.info();
        assert_eq!(info.id, "pdeformer2");
        assert!(info.available);
        assert!(!info.supported_pde_types.is_empty());
    }
}

#[cfg(test)]
mod timeout_env_tests {
    /// Helper that mirrors the solver_timeout_secs() logic used in both
    /// classical.rs and pdeformer2.rs so we can unit-test env-var parsing
    /// without depending on private functions.
    fn parse_solver_timeout(var: Option<&str>) -> u64 {
        var.and_then(|v| v.parse::<u64>().ok()).unwrap_or(300)
    }

    #[test]
    fn default_timeout_is_300() {
        assert_eq!(parse_solver_timeout(None), 300);
    }

    #[test]
    fn valid_env_var_overrides_default() {
        assert_eq!(parse_solver_timeout(Some("60")), 60);
        assert_eq!(parse_solver_timeout(Some("3600")), 3600);
    }

    #[test]
    fn non_numeric_env_var_falls_back_to_default() {
        assert_eq!(parse_solver_timeout(Some("not_a_number")), 300);
        assert_eq!(parse_solver_timeout(Some("")), 300);
        assert_eq!(parse_solver_timeout(Some("0.5")), 300); // f64, not u64
    }

    #[test]
    fn zero_timeout_is_accepted() {
        // 0 is a valid u64; callers can use it as "no timeout" signal if desired
        assert_eq!(parse_solver_timeout(Some("0")), 0);
    }
}

#[cfg(test)]
mod error_tests {
    use crate::error::ApiError;
    use axum::response::IntoResponse;
    use axum::http::StatusCode;

    fn status_of(e: ApiError) -> StatusCode {
        e.into_response().status()
    }

    #[test]
    fn file_not_found_returns_404() {
        assert_eq!(status_of(ApiError::FileNotFound("x".into())), StatusCode::NOT_FOUND);
    }

    #[test]
    fn solver_not_found_returns_404() {
        assert_eq!(status_of(ApiError::SolverNotFound("x".into())), StatusCode::NOT_FOUND);
    }

    #[test]
    fn bad_request_returns_400() {
        assert_eq!(status_of(ApiError::BadRequest("x".into())), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn solver_error_returns_500() {
        assert_eq!(
            status_of(ApiError::SolverError("fail".into())),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn timeout_returns_408() {
        assert_eq!(status_of(ApiError::Timeout(300)), StatusCode::REQUEST_TIMEOUT);
    }

    #[test]
    fn internal_returns_500() {
        let e = ApiError::Internal(anyhow::anyhow!("oops"));
        assert_eq!(status_of(e), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn timeout_error_message_contains_seconds() {
        let e = ApiError::Timeout(42);
        let msg = e.to_string();
        assert!(msg.contains("42"), "timeout message should include seconds: {}", msg);
    }
}
