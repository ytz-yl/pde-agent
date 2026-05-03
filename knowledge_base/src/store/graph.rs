/// Neo4j graph connection management and schema initialisation.
///
/// This module handles:
///   - Creating the Neo4j Graph pool (neo4rs)
///   - Creating constraints and indexes on first run
///   - Seeding initial knowledge nodes

use anyhow::{Context, Result};
use neo4rs::Graph;

use crate::store::schema::{
    LABEL_AI_MODEL, LABEL_CONDITION, LABEL_EQUATION, LABEL_LOSS_FUNCTION,
    LABEL_METRIC, LABEL_NUMERICAL_METHOD,
    REL_EVALUATED_BY, REL_SOLVES, REL_TRAINED_BY, REL_TESTED_ON,
};

// ── Connection ────────────────────────────────────────────────────────────────

/// Create a Neo4j Graph connection pool from environment variables.
///
/// Environment variables:
///   NEO4J_URI      Bolt URI (default: bolt://localhost:7687)
///   NEO4J_USER     Username (default: neo4j)
///   NEO4J_PASSWORD Password (required)
pub async fn connect() -> Result<Graph> {
    let uri = std::env::var("NEO4J_URI")
        .unwrap_or_else(|_| "bolt://localhost:7687".into());
    let user = std::env::var("NEO4J_USER")
        .unwrap_or_else(|_| "neo4j".into());
    let password = std::env::var("NEO4J_PASSWORD")
        .unwrap_or_else(|_| "password".into());

    let graph = Graph::new(&uri, &user, &password)
        .await
        .context("connect to neo4j")?;

    tracing::info!("connected to neo4j at {}", uri);
    Ok(graph)
}

// ── Schema initialisation ─────────────────────────────────────────────────────

/// Create uniqueness constraints and indexes for all node labels.
/// Safe to call multiple times (IF NOT EXISTS).
pub async fn init_schema(graph: &Graph) -> Result<()> {
    let constraints: &[&str] = &[
        "CREATE CONSTRAINT equation_id IF NOT EXISTS FOR (n:Equation)         REQUIRE n.id IS UNIQUE",
        "CREATE CONSTRAINT condition_id IF NOT EXISTS FOR (n:Condition)        REQUIRE n.id IS UNIQUE",
        "CREATE CONSTRAINT theorem_id IF NOT EXISTS FOR (n:Theorem)            REQUIRE n.id IS UNIQUE",
        "CREATE CONSTRAINT numerical_method_id IF NOT EXISTS FOR (n:NumericalMethod) REQUIRE n.id IS UNIQUE",
        "CREATE CONSTRAINT ai_model_id IF NOT EXISTS FOR (n:AIModel)           REQUIRE n.id IS UNIQUE",
        "CREATE CONSTRAINT loss_function_id IF NOT EXISTS FOR (n:LossFunction) REQUIRE n.id IS UNIQUE",
        "CREATE CONSTRAINT metric_id IF NOT EXISTS FOR (n:Metric)              REQUIRE n.id IS UNIQUE",
        "CREATE CONSTRAINT dataset_id IF NOT EXISTS FOR (n:Dataset)            REQUIRE n.id IS UNIQUE",
        "CREATE CONSTRAINT paper_id   IF NOT EXISTS FOR (n:Paper)              REQUIRE n.id IS UNIQUE",
    ];

    for cypher in constraints {
        graph
            .run(neo4rs::query(cypher))
            .await
            .with_context(|| format!("init constraint: {}", cypher))?;
    }

    tracing::info!("neo4j schema constraints applied");
    Ok(())
}

// ── Seed data ─────────────────────────────────────────────────────────────────

/// Insert seed knowledge nodes and relations if they do not yet exist.
/// Uses MERGE so it is idempotent.
pub async fn seed_data(graph: &Graph) -> Result<()> {
    seed_equations(graph).await?;
    seed_conditions(graph).await?;
    seed_numerical_methods(graph).await?;
    seed_ai_models(graph).await?;
    seed_loss_functions(graph).await?;
    seed_metrics(graph).await?;
    seed_datasets(graph).await?;
    seed_relations(graph).await?;
    tracing::info!("seed data applied");
    Ok(())
}

async fn seed_equations(graph: &Graph) -> Result<()> {
    let equations: &[(&str, &str, &str, &[&str], bool, &str)] = &[
        // (id, name, pde_type, variables, time_dependent, description)
        ("heat_equation",   "Heat Equation",             "parabolic",  &["t", "x"],          true,  "Parabolic PDE describing heat diffusion. du/dt = alpha * laplacian(u)."),
        ("wave_equation",   "Wave Equation",             "hyperbolic", &["t", "x"],          true,  "Hyperbolic PDE describing wave propagation. d2u/dt2 = c2 * laplacian(u)."),
        ("poisson",         "Poisson Equation",          "elliptic",   &["x", "y"],          false, "Elliptic PDE. laplacian(u) = f. Arises in electrostatics, fluid pressure."),
        ("navier_stokes",   "Navier-Stokes Equations",   "mixed",      &["t", "x", "y", "z"],true,  "Incompressible viscous fluid flow equations. du/dt + (u·∇)u = -∇p + ν·laplacian(u)."),
        ("burgers",         "Burgers Equation",          "hyperbolic", &["t", "x"],          true,  "Nonlinear PDE combining diffusion and nonlinear convection."),
        ("schrodinger",     "Schrödinger Equation",      "parabolic",  &["t", "x"],          true,  "Quantum mechanics wave function evolution."),
        ("allen_cahn",      "Allen-Cahn Equation",       "parabolic",  &["t", "x"],          true,  "Phase field model for interface dynamics."),
    ];

    for &(id, name, pde_type, vars, time_dep, desc) in equations {
        let vars_str = format!("{:?}", vars);
        graph.run(neo4rs::query(&format!(
            "MERGE (n:{label} {{id: $id}}) \
             ON CREATE SET n.name = $name, n.pde_type = $pde_type, n.variables = $vars, \
                           n.time_dependent = $time_dep, n.description = $desc, n.tags = [] \
             ON MATCH SET  n.name = $name",
            label = LABEL_EQUATION
        ))
        .param("id", id)
        .param("name", name)
        .param("pde_type", pde_type)
        .param("vars", vars_str.as_str())
        .param("time_dep", time_dep)
        .param("desc", desc))
        .await
        .with_context(|| format!("seed equation {}", id))?;
    }
    Ok(())
}

async fn seed_conditions(graph: &Graph) -> Result<()> {
    let conditions: &[(&str, &str, &str, &str)] = &[
        // (id, name, type, form)
        ("dirichlet_bc",     "Dirichlet Boundary Condition", "boundary",   "u = g on ∂Ω"),
        ("neumann_bc",       "Neumann Boundary Condition",   "boundary",   "∂u/∂n = g on ∂Ω"),
        ("periodic_bc",      "Periodic Boundary Condition",  "boundary",   "u(0) = u(L)"),
        ("zero_ic",          "Zero Initial Condition",       "initial",    "u(x,0) = 0"),
        ("bounded_domain",   "Bounded Domain",               "domain",     "Ω is bounded in R^n"),
        ("smooth_coeffs",    "Smooth Coefficients",          "regularity", "Coefficients in C^∞"),
    ];

    for &(id, name, ctype, form) in conditions {
        graph.run(neo4rs::query(&format!(
            "MERGE (n:{label} {{id: $id}}) \
             ON CREATE SET n.name = $name, n.condition_type = $ctype, n.form = $form",
            label = LABEL_CONDITION
        ))
        .param("id", id)
        .param("name", name)
        .param("ctype", ctype)
        .param("form", form))
        .await
        .with_context(|| format!("seed condition {}", id))?;
    }
    Ok(())
}

async fn seed_numerical_methods(graph: &Graph) -> Result<()> {
    let methods: &[(&str, &str, &str, u32, &str)] = &[
        // (id, name, type, order, description)
        ("fdm", "Finite Difference Method",  "grid_based",     2, "Approximates derivatives by finite differences on structured grids."),
        ("fem", "Finite Element Method",     "mesh_based",     2, "Variational formulation on unstructured meshes. Handles complex geometries."),
        ("fvm", "Finite Volume Method",      "mesh_based",     2, "Integral form of conservation laws on control volumes. Widely used in CFD."),
        ("spectral", "Spectral Methods",     "spectral_based", 0, "Global basis functions (Fourier, Chebyshev). Exponential convergence for smooth solutions."),
    ];

    for &(id, name, mtype, order, desc) in methods {
        graph.run(neo4rs::query(&format!(
            "MERGE (n:{label} {{id: $id}}) \
             ON CREATE SET n.name = $name, n.method_type = $mtype, n.order = $order, \
                           n.description = $desc, n.tags = []",
            label = LABEL_NUMERICAL_METHOD
        ))
        .param("id", id)
        .param("name", name)
        .param("mtype", mtype)
        .param("order", order as i64)
        .param("desc", desc))
        .await
        .with_context(|| format!("seed numerical method {}", id))?;
    }
    Ok(())
}

async fn seed_ai_models(graph: &Graph) -> Result<()> {
    let models: &[(&str, &str, &str, &str, &str, &str)] = &[
        // (id, name, architecture, training_type, paper_ref, description)
        ("pinn",       "Physics-Informed Neural Network", "MLP",         "physics_informed",  "Raissi et al. 2019",  "Embeds PDE residuals into the loss of a neural network. Mesh-free, good for inverse problems."),
        ("deeponet",   "Deep Operator Network",           "MLP",         "operator_learning", "Lu et al. 2021",      "Learns mappings between function spaces via branch/trunk nets."),
        ("fno",        "Fourier Neural Operator",         "FNO",         "operator_learning", "Li et al. 2021",      "Learns solution operators in Fourier space. Fast inference, resolution-invariant."),
        ("pdeformer",  "PDEformer",                       "Transformer", "supervised",        "anonymous 2024",      "Transformer-based universal PDE solver using symbolic DAG representation."),
        ("deepxde_net","DeepXDE Network",                 "MLP",         "physics_informed",  "Lu et al. 2021b",     "PINN variant using DeepXDE framework. Supports residual-based adaptive refinement."),
    ];

    for &(id, name, arch, training, paper, desc) in models {
        graph.run(neo4rs::query(&format!(
            "MERGE (n:{label} {{id: $id}}) \
             ON CREATE SET n.name = $name, n.architecture = $arch, \
                           n.training_type = $training, n.paper_ref = $paper, \
                           n.description = $desc, n.input_vars = [], n.output_vars = [], n.tags = []",
            label = LABEL_AI_MODEL
        ))
        .param("id", id)
        .param("name", name)
        .param("arch", arch)
        .param("training", training)
        .param("paper", paper)
        .param("desc", desc))
        .await
        .with_context(|| format!("seed ai model {}", id))?;
    }
    Ok(())
}

async fn seed_loss_functions(graph: &Graph) -> Result<()> {
    let losses: &[(&str, &str, &str, &str)] = &[
        // (id, name, type, description)
        ("pde_residual_loss",  "PDE Residual Loss",       "physics",    "Minimise the PDE residual at collocation points."),
        ("boundary_loss",      "Boundary Condition Loss", "boundary",   "Penalise violation of boundary conditions."),
        ("data_mse_loss",      "Data MSE Loss",           "data_driven","Mean squared error against training data."),
        ("combined_pinn_loss", "Combined PINN Loss",      "combined",   "Weighted sum of PDE residual + boundary + initial condition losses."),
    ];

    for &(id, name, ltype, desc) in losses {
        graph.run(neo4rs::query(&format!(
            "MERGE (n:{label} {{id: $id}}) \
             ON CREATE SET n.name = $name, n.loss_type = $ltype, n.description = $desc",
            label = LABEL_LOSS_FUNCTION
        ))
        .param("id", id)
        .param("name", name)
        .param("ltype", ltype)
        .param("desc", desc))
        .await
        .with_context(|| format!("seed loss function {}", id))?;
    }
    Ok(())
}

async fn seed_metrics(graph: &Graph) -> Result<()> {
    let metrics: &[(&str, &str, &str, &str)] = &[
        // (id, name, type, description)
        ("l2_error",        "L2 Relative Error",   "accuracy",        "Relative L2 norm of error: ||u_pred - u_true|| / ||u_true||."),
        ("linf_error",      "L∞ Error",            "accuracy",        "Maximum absolute pointwise error."),
        ("mse",             "Mean Squared Error",  "accuracy",        "Average of squared differences between prediction and ground truth."),
        ("inference_time",  "Inference Time",      "efficiency",      "Wall-clock time to generate a solution."),
        ("training_time",   "Training Time",       "efficiency",      "Total time to train the model."),
        ("param_count",     "Parameter Count",     "efficiency",      "Number of trainable parameters."),
        ("generalisation",  "Generalisation Error","generalisation",  "Error on out-of-distribution test samples."),
    ];

    for &(id, name, mtype, desc) in metrics {
        graph.run(neo4rs::query(&format!(
            "MERGE (n:{label} {{id: $id}}) \
             ON CREATE SET n.name = $name, n.metric_type = $mtype, n.description = $desc",
            label = LABEL_METRIC
        ))
        .param("id", id)
        .param("name", name)
        .param("mtype", mtype)
        .param("desc", desc))
        .await
        .with_context(|| format!("seed metric {}", id))?;
    }
    Ok(())
}

async fn seed_datasets(graph: &Graph) -> Result<()> {
    let datasets: &[(&str, &str, &str, &str)] = &[
        // (id, name, dimension, description)
        ("burgers_1d",      "Burgers 1D Dataset",         "1D", "Standard Burgers equation benchmark used in FNO paper (viscosity=0.01)."),
        ("navier_stokes_2d","Navier-Stokes 2D Dataset",   "2D", "2D Kolmogorov flow benchmark from FNO paper."),
        ("heat_2d",         "Heat Equation 2D Dataset",   "2D", "Heat equation on unit square with Dirichlet BC."),
        ("darcy_flow",      "Darcy Flow Dataset",         "2D", "Steady-state Darcy flow with random permeability fields."),
    ];

    for &(id, name, dim, desc) in datasets {
        graph.run(neo4rs::query(&format!(
            "MERGE (n:Dataset {{id: $id}}) \
             ON CREATE SET n.name = $name, n.dimension = $dim, n.description = $desc"
        ))
        .param("id", id)
        .param("name", name)
        .param("dim", dim)
        .param("desc", desc))
        .await
        .with_context(|| format!("seed dataset {}", id))?;
    }
    Ok(())
}

async fn seed_relations(graph: &Graph) -> Result<()> {
    // AIModel --SOLVES--> Equation
    let solves: &[(&str, &str, &str, &str)] = &[
        // (from_label, from_id, to_label, to_id)
        (LABEL_AI_MODEL, "pinn",       LABEL_EQUATION, "heat_equation"),
        (LABEL_AI_MODEL, "pinn",       LABEL_EQUATION, "wave_equation"),
        (LABEL_AI_MODEL, "pinn",       LABEL_EQUATION, "poisson"),
        (LABEL_AI_MODEL, "pinn",       LABEL_EQUATION, "burgers"),
        (LABEL_AI_MODEL, "deeponet",   LABEL_EQUATION, "burgers"),
        (LABEL_AI_MODEL, "deeponet",   LABEL_EQUATION, "heat_equation"),
        (LABEL_AI_MODEL, "fno",        LABEL_EQUATION, "navier_stokes"),
        (LABEL_AI_MODEL, "fno",        LABEL_EQUATION, "burgers"),
        (LABEL_AI_MODEL, "fno",        LABEL_EQUATION, "darcy_flow"),
        (LABEL_AI_MODEL, "pdeformer",  LABEL_EQUATION, "heat_equation"),
        (LABEL_AI_MODEL, "pdeformer",  LABEL_EQUATION, "wave_equation"),
        (LABEL_AI_MODEL, "pdeformer",  LABEL_EQUATION, "burgers"),
        (LABEL_AI_MODEL, "deepxde_net",LABEL_EQUATION, "heat_equation"),
        (LABEL_AI_MODEL, "deepxde_net",LABEL_EQUATION, "poisson"),
        (LABEL_NUMERICAL_METHOD, "fdm",   LABEL_EQUATION, "heat_equation"),
        (LABEL_NUMERICAL_METHOD, "fdm",   LABEL_EQUATION, "wave_equation"),
        (LABEL_NUMERICAL_METHOD, "fdm",   LABEL_EQUATION, "burgers"),
        (LABEL_NUMERICAL_METHOD, "fem",   LABEL_EQUATION, "poisson"),
        (LABEL_NUMERICAL_METHOD, "fem",   LABEL_EQUATION, "heat_equation"),
        (LABEL_NUMERICAL_METHOD, "fem",   LABEL_EQUATION, "navier_stokes"),
        (LABEL_NUMERICAL_METHOD, "fvm",   LABEL_EQUATION, "navier_stokes"),
        (LABEL_NUMERICAL_METHOD, "spectral", LABEL_EQUATION, "wave_equation"),
        (LABEL_NUMERICAL_METHOD, "spectral", LABEL_EQUATION, "burgers"),
    ];

    for &(from_label, from_id, to_label, to_id) in solves {
        graph.run(neo4rs::query(&format!(
            "MATCH (a:{fl} {{id: $from_id}}), (b:{tl} {{id: $to_id}}) \
             MERGE (a)-[:{rel}]->(b)",
            fl = from_label, tl = to_label, rel = REL_SOLVES
        ))
        .param("from_id", from_id)
        .param("to_id", to_id))
        .await
        .with_context(|| format!("seed SOLVES {}->{}", from_id, to_id))?;
    }

    // AIModel --TRAINED_BY--> LossFunction
    let trained_by: &[(&str, &str)] = &[
        ("pinn",        "combined_pinn_loss"),
        ("deepxde_net", "combined_pinn_loss"),
        ("deeponet",    "data_mse_loss"),
        ("fno",         "data_mse_loss"),
        ("pdeformer",   "data_mse_loss"),
    ];

    for &(model_id, loss_id) in trained_by {
        graph.run(neo4rs::query(&format!(
            "MATCH (a:{fl} {{id: $mid}}), (b:{tl} {{id: $lid}}) \
             MERGE (a)-[:{rel}]->(b)",
            fl = LABEL_AI_MODEL, tl = LABEL_LOSS_FUNCTION, rel = REL_TRAINED_BY
        ))
        .param("mid", model_id)
        .param("lid", loss_id))
        .await
        .with_context(|| format!("seed TRAINED_BY {}->{}", model_id, loss_id))?;
    }

    // LossFunction --REPRESENTS--> Equation (which PDE the loss encodes)
    let represents: &[(&str, &str)] = &[
        ("pde_residual_loss",  "heat_equation"),
        ("combined_pinn_loss", "heat_equation"),
    ];
    for &(loss_id, eq_id) in represents {
        graph.run(neo4rs::query(
            "MATCH (a:LossFunction {id: $lid}), (b:Equation {id: $eid}) \
             MERGE (a)-[:REPRESENTS]->(b)"
        )
        .param("lid", loss_id)
        .param("eid", eq_id))
        .await
        .with_context(|| format!("seed REPRESENTS {}->{}", loss_id, eq_id))?;
    }

    // AIModel --EVALUATED_BY--> Metric
    let eval_metrics = &["l2_error", "mse", "inference_time"];
    for model_id in &["pinn", "deeponet", "fno", "pdeformer", "deepxde_net"] {
        for metric_id in eval_metrics {
            graph.run(neo4rs::query(&format!(
                "MATCH (a:{fl} {{id: $mid}}), (b:{tl} {{id: $metid}}) \
                 MERGE (a)-[:{rel}]->(b)",
                fl = LABEL_AI_MODEL, tl = LABEL_METRIC, rel = REL_EVALUATED_BY
            ))
            .param("mid", *model_id)
            .param("metid", *metric_id))
            .await
            .with_context(|| format!("seed EVALUATED_BY {}->{}", model_id, metric_id))?;
        }
    }

    // AIModel --TESTED_ON--> Dataset
    let tested_on: &[(&str, &str)] = &[
        ("pinn",       "burgers_1d"),
        ("fno",        "navier_stokes_2d"),
        ("fno",        "burgers_1d"),
        ("fno",        "darcy_flow"),
        ("deeponet",   "burgers_1d"),
        ("pdeformer",  "burgers_1d"),
        ("pdeformer",  "heat_2d"),
    ];

    for &(model_id, dataset_id) in tested_on {
        graph.run(neo4rs::query(&format!(
            "MATCH (a:{fl} {{id: $mid}}), (b:Dataset {{id: $did}}) \
             MERGE (a)-[:{rel}]->(b)",
            fl = LABEL_AI_MODEL, rel = REL_TESTED_ON
        ))
        .param("mid", model_id)
        .param("did", dataset_id))
        .await
        .with_context(|| format!("seed TESTED_ON {}->{}", model_id, dataset_id))?;
    }

    // Equation --HAS_CONDITION--> Condition (common pairings)
    let eq_conditions: &[(&str, &str)] = &[
        ("heat_equation", "dirichlet_bc"),
        ("heat_equation", "zero_ic"),
        ("poisson",       "dirichlet_bc"),
        ("poisson",       "neumann_bc"),
        ("wave_equation", "periodic_bc"),
        ("wave_equation", "zero_ic"),
        ("navier_stokes", "dirichlet_bc"),
        ("burgers",       "periodic_bc"),
    ];

    for &(eq_id, cond_id) in eq_conditions {
        graph.run(neo4rs::query(
            "MATCH (a:Equation {id: $eid}), (b:Condition {id: $cid}) \
             MERGE (a)-[:HAS_CONDITION]->(b)"
        )
        .param("eid", eq_id)
        .param("cid", cond_id))
        .await
        .with_context(|| format!("seed HAS_CONDITION {}->{}", eq_id, cond_id))?;
    }

    // Dataset --BASED_ON--> Equation
    let dataset_equations: &[(&str, &str)] = &[
        ("burgers_1d",       "burgers"),
        ("navier_stokes_2d", "navier_stokes"),
        ("heat_2d",          "heat_equation"),
        ("darcy_flow",       "poisson"),
    ];

    for &(ds_id, eq_id) in dataset_equations {
        graph.run(neo4rs::query(
            "MATCH (a:Dataset {id: $dsid}), (b:Equation {id: $eid}) \
             MERGE (a)-[:BASED_ON]->(b)"
        )
        .param("dsid", ds_id)
        .param("eid", eq_id))
        .await
        .with_context(|| format!("seed BASED_ON {}->{}", ds_id, eq_id))?;
    }

    Ok(())
}
