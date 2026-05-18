// Shared shape — keeps both locales in sync without literal-type conflicts
export interface Translations {
  nav: { home: string; knowledge: string; solver: string; skills: string }
  lang: { toggle: string }
  home: {
    hero: {
      badge: string
      title: string
      subtitle: string
      ctaKnowledge: string
      ctaSolver: string
    }
    vision: {
      title: string
      subtitle: string
    }
    arch: {
      title: string
      subtitle: string
      components: Array<{
        title: string
        desc: string
        tags: string[]
      }>
    }
    roadmap: {
      title: string
      subtitle: string
      items: Array<{ phase: string; title: string; desc: string; status: 'done' | 'active' | 'optional' | 'next' }>
    }
    flow: {
      title: string
      subtitle: string
    }
    stats: Array<{ value: string; label: string }>
  }
  knowledge: {
    title: string; subtitle: string
    tabs: { equations: string; models: string; benchmarks: string; search: string }
    equations: {
      selectToView: string; executableLabel: string
      executableSection: string; literatureSection: string
      aiModelsSubhead: string; numericalMethodsSubhead: string
      noSolvers: string
    }
    models: { selectToView: string }
    benchmarks: {
      selectToView: string
      noBenchmarks: string
      noEntries: string
      protocol: string
      tolerance: string
      lowerIsBetter: string
      higherIsBetter: string
      columns: { rank: string; method: string; bestValue: string; sources: string; confidence: string; latest: string }
      confidence: { verified: string; single: string; disputed: string }
      sourceTypes: { paper_reported: string; self_run: string; third_party_reproduction: string }
      breakdownLabel: string
    }
    search: {
      placeholder: string; button: string
      results: (n: number) => string; noResults: string
    }
  }
  solver: {
    title: string; subtitle: string
    tabs: { catalog: string; solve: string }
    catalog: {
      noSolvers: string; backend: string
      categoryLabels: { classical: string; machine_learning: string; hybrid: string }
    }
    cases: {
      selectCase: string
      solver: string
      resolution: string
      resolutionSuffix: string
      runButton: string
      running: string
      pdeLabel: string
      icLabel: string
      bcLabel: string
      domainLabel: string
      varLabel: string
    }
    result: {
      timeStep: (cur: number, total: number) => string
      solutionField: string
      variable: string
    }
  }
  skills: {
    title: string; subtitle: string; selectSkill: string; selectFile: string
    downloadButton: (name: string) => string; downloading: string
    files: string; noSkills: string; loadError: string
  }
  common: { loading: string; error: string }
}

export const zh: Translations = {
  nav: {
    home: '首页',
    knowledge: '知识库',
    solver: '求解器',
    skills: 'Skills',
  },
  lang: {
    toggle: 'English',
  },

  // ── Home page ────────────────────────────────────────────────────────────────
  home: {
    hero: {
      badge: '基础设施 · 知识 · 求解',
      title: 'PDE Agent',
      subtitle: '为 AI Agent 提供偏微分方程领域的专业服务基础设施——结构化知识图谱、多范式求解器与标准化 Skill 规范，让任意 Agent 框架都能可靠地处理 PDE 问题。',
      ctaKnowledge: '探索知识库',
      ctaSolver: '运行求解器',
    },
    vision: {
      title: '愿景：用 PDE Agent 持续壮大生态',
      subtitle: '我们计划利用 PDE Agent 针对各类典型 PDE 场景自动化生成训练数据、微调专用模型，并将验证后的模型纳入知识库与求解服务，形成"Agent 驱动 → 数据生成 → 模型训练 → 能力回流"的闭环。',
    },
    arch: {
      title: '系统架构',
      subtitle: '三个核心后端服务，共同构成可被任意 Agent 框架调用的 PDE 领域能力层。',
      components: [
        {
          title: 'PDE 知识库',
          desc: '基于 Neo4j 构建的图结构知识库。将方程、数值方法、AI 模型、损失函数、评估指标、基准数据集与研究论文组织为互联的知识图谱，支持图遍历查询与全文搜索。持续接收新模型与论文的增量更新。',
          tags: ['Neo4j', 'SQLite', 'Rust / axum', '9 节点类型', '15 关系类型'],
        },
        {
          title: 'PDE 求解服务',
          desc: '精选先进求解器库，以统一 REST 接口对外暴露。经典 FDM / FEM / FVM 与开源 ML 方法并重，支持任意空间坐标查询与多变量方程组。新模型持续纳入。',
          tags: ['PDEformer-2', 'FDM / FEM', 'FNO / PINNs', 'Rust / axum', '多变量支持'],
        },
        {
          title: 'Agent Skill 规范',
          desc: '层级化 Skill 文档告诉 Agent 有哪些服务、何时使用、如何调用。SKILL.md 将用户意图映射到具体 API 端点，子文档覆盖请求体构造、响应解析与典型场景范例。',
          tags: ['SKILL.md', '场景映射', 'API 规范', '可打包下载'],
        },
      ],
    },
    roadmap: {
      title: '演进路线',
      subtitle: '从基础设施搭建，到开源模型持续集成，再到 Agent 驱动的闭环训练生态。',
      items: [
        {
          phase: 'Phase 1',
          title: '基础设施',
          desc: '知识图谱 + 求解服务 + Skill 规范 + 前端界面，完成可被外部 Agent 调用的核心能力层。',
          status: 'done',
        },
        {
          phase: 'Phase 2',
          title: '开源 PDE AI 模型集成 & 知识库持续更新',
          desc: '持续接入开源 PDE AI 模型（PDEformer-2、FNO、DeepONet 等），完善多变量、复杂域、混合边界条件的推理支持；同步扩充知识库中的模型节点、论文与关系数据。',
          status: 'active',
        },
        {
          phase: 'Phase 3',
          title: 'Agent 驱动数据生成（可选）',
          desc: '利用 PDE Agent 针对 Burgers、NS、浅水波等典型场景自动化生成大规模训练数据，借助知识库中的方程关系进行场景扩展。此阶段为可选路径，视需求决定是否推进。',
          status: 'optional',
        },
        {
          phase: 'Phase 4',
          title: '专用模型训练与能力回流',
          desc: '基于生成数据微调各场景专用模型，验证后纳入知识库（作为新 AIModel 节点）和求解服务（作为新后端），形成能力自增长闭环。',
          status: 'next',
        },
      ],
    },
    flow: {
      title: 'Agent 驱动闭环',
      subtitle: 'PDE Agent 不只是工具的使用者，也是知识与求解能力的构建者。',
    },
    stats: [
      { value: '9', label: '知识图谱节点类型' },
      { value: '15', label: '关系类型' },
      { value: '9', label: 'PDE 求解案例' },
      { value: '3', label: '核心后端服务' },
    ],
  },

  // ── Knowledge page ──────────────────────────────────────────────────────────
  knowledge: {
    title: '知识库',
    subtitle: '浏览 PDE 方程、AI 模型与数值方法，探索知识图谱中的关系。',
    tabs: {
      equations: '方程',
      models: '模型与方法',
      benchmarks: '榜单',
      search: '搜索',
    },
    equations: {
      selectToView: '选择一个方程查看详情',
      executableLabel: '本地可执行',
      executableSection: '本地可执行',
      literatureSection: '仅文献存在',
      aiModelsSubhead: 'AI / ML 模型',
      numericalMethodsSubhead: '数值方法',
      noSolvers: '尚未关联求解器。',
    },
    models: {
      selectToView: '选择一个模型查看详情',
    },
    benchmarks: {
      selectToView: '选择一个评测查看排行榜',
      noBenchmarks: '暂无评测口径，先去后端 seed 一些。',
      noEntries: '该评测下还没有任何测量结果。提交一条 BenchResult 再回来看。',
      protocol: '协议',
      tolerance: '容差',
      lowerIsBetter: '越小越好',
      higherIsBetter: '越大越好',
      columns: { rank: '#', method: '方法', bestValue: '最佳值', sources: '独立来源', confidence: '置信度', latest: '最近一次' },
      confidence: { verified: '已验证', single: '单一来源', disputed: '存在分歧' },
      sourceTypes: { paper_reported: '论文', self_run: '本地', third_party_reproduction: '第三方' },
      breakdownLabel: '来源分布',
    },
    search: {
      placeholder: '例如：Navier-Stokes、FNO、heat equation…',
      button: '搜索',
      results: (n: number) => `${n} 条结果`,
      noResults: '未找到匹配节点，请尝试其他关键词。',
    },
  },

  // ── Solver page ─────────────────────────────────────────────────────────────
  solver: {
    title: 'PDE 求解器',
    subtitle: '浏览可用求解器，或选择示例案例使用 PDEformer-2 进行推理。',
    tabs: {
      catalog: '求解器目录',
      solve: '运行求解',
    },
    catalog: {
      noSolvers: '暂无可用求解器，请确认求解器服务正在运行。',
      backend: '后端',
      categoryLabels: { classical: '经典', machine_learning: 'ML', hybrid: '混合' },
    },
    cases: {
      selectCase: '选择示例案例',
      solver: '求解器',
      resolution: '输出分辨率',
      resolutionSuffix: '× 网格',
      runButton: '运行求解器',
      running: '推理中…',
      pdeLabel: 'PDE 形式',
      icLabel: '初始条件',
      bcLabel: '边界条件',
      domainLabel: '定义域',
      varLabel: '变量',
    },
    result: {
      timeStep: (cur: number, total: number) => `时间步：${cur} / ${total}`,
      solutionField: '解场',
      variable: '变量',
    },
  },

  // ── Skills page ─────────────────────────────────────────────────────────────
  skills: {
    title: 'Skills',
    subtitle: '浏览 Agent skill 文档，可打包下载供 Agent 框架直接使用。',
    selectSkill: '选择一个 skill 包查看文档',
    selectFile: '从左侧选择文件预览',
    downloadButton: (name: string) => `下载 ${name}`,
    downloading: '打包中…',
    files: '文件',
    noSkills: '未找到 skill 包。',
    loadError: '加载失败',
  },

  // ── Common ──────────────────────────────────────────────────────────────────
  common: {
    loading: '加载中…',
    error: '发生错误',
  },
}

export const en: Translations = {
  nav: {
    home: 'Home',
    knowledge: 'Knowledge Base',
    solver: 'Solver',
    skills: 'Skills',
  },
  lang: {
    toggle: '中文',
  },

  // ── Home page ────────────────────────────────────────────────────────────────
  home: {
    hero: {
      badge: 'Infrastructure · Knowledge · Solving',
      title: 'PDE Agent',
      subtitle: 'Domain-specific infrastructure for AI agents tackling partial differential equations — a structured knowledge graph, multi-paradigm solvers, and standardised Skill specs that any agent framework can call.',
      ctaKnowledge: 'Explore Knowledge Base',
      ctaSolver: 'Run Solver',
    },
    vision: {
      title: 'Vision: A Self-Growing Ecosystem Driven by PDE Agent',
      subtitle: 'We plan to use the PDE Agent to automate training-data generation for diverse PDE scenarios, fine-tune scenario-specific models, and feed the validated models back into the knowledge base and solver service — closing the loop of "Agent drives → data generated → model trained → capability returned".',
    },
    arch: {
      title: 'System Architecture',
      subtitle: 'Three core backend services forming a PDE capability layer that any agent framework can consume.',
      components: [
        {
          title: 'PDE Knowledge Base',
          desc: 'A graph-structured knowledge base built on Neo4j. Equations, numerical methods, AI models, loss functions, metrics, benchmark datasets, and research papers are organised into an interconnected knowledge graph with graph-traversal queries and full-text search. Continuously updated with new models and papers.',
          tags: ['Neo4j', 'SQLite', 'Rust / axum', '9 node types', '15 relation types'],
        },
        {
          title: 'PDE Solver Service',
          desc: 'A curated library of open-source solvers exposed through a unified REST interface. Classical FDM / FEM / FVM and ML methods are treated equally; multi-variable equation systems and arbitrary coordinate queries are supported. New models are added incrementally.',
          tags: ['PDEformer-2', 'FDM / FEM', 'FNO / PINNs', 'Rust / axum', 'Multi-variable'],
        },
        {
          title: 'Agent Skill Specs',
          desc: 'Hierarchical Skill documents tell agents what services exist, when to use them, and how to call them. SKILL.md maps user intent to specific API endpoints; sub-documents cover request construction, response parsing, and annotated scenario examples.',
          tags: ['SKILL.md', 'Intent mapping', 'API spec', 'Downloadable'],
        },
      ],
    },
    roadmap: {
      title: 'Roadmap',
      subtitle: 'From infrastructure to continuous open-source model integration to an agent-driven closed-loop training ecosystem.',
      items: [
        {
          phase: 'Phase 1',
          title: 'Infrastructure',
          desc: 'Knowledge graph + solver service + Skill specs + frontend. The core capability layer ready for external agent consumption.',
          status: 'done',
        },
        {
          phase: 'Phase 2',
          title: 'Open-Source PDE AI Model Integration & Knowledge Base Updates',
          desc: 'Continuously integrating open-source PDE AI models (PDEformer-2, FNO, DeepONet, …) with support for multi-variable, complex-domain, and mixed-BC inference; simultaneously expanding model nodes, papers, and relations in the knowledge base.',
          status: 'active',
        },
        {
          phase: 'Phase 3',
          title: 'Agent-Driven Data Generation (Optional)',
          desc: 'Use the PDE Agent to automatically generate large-scale training data for canonical scenarios (Burgers, NS, shallow water, …) and leverage knowledge-graph equation relations to expand scenario coverage. This phase is optional and will proceed based on need.',
          status: 'optional',
        },
        {
          phase: 'Phase 4',
          title: 'Model Training & Capability Feedback',
          desc: 'Fine-tune scenario-specific models on the generated data, then inject them back into the knowledge base (as new AIModel nodes) and the solver service (as new backends) — completing the self-growing capability loop.',
          status: 'next',
        },
      ],
    },
    flow: {
      title: 'Agent-Driven Closed Loop',
      subtitle: 'The PDE Agent is not just a consumer of tools — it is also a builder of knowledge and solving capability.',
    },
    stats: [
      { value: '9', label: 'Knowledge graph node types' },
      { value: '15', label: 'Relation types' },
      { value: '9', label: 'PDE solver cases' },
      { value: '3', label: 'Core backend services' },
    ],
  },

  knowledge: {
    title: 'Knowledge Base',
    subtitle: 'Explore PDE equations, AI models, and numerical methods in the knowledge graph.',
    tabs: {
      equations: 'Equations',
      models: 'Models & Methods',
      benchmarks: 'Leaderboards',
      search: 'Search',
    },
    equations: {
      selectToView: 'Select an equation to view details',
      executableLabel: 'runnable here',
      executableSection: 'Runnable here',
      literatureSection: 'Literature only',
      aiModelsSubhead: 'AI / ML Models',
      numericalMethodsSubhead: 'Numerical Methods',
      noSolvers: 'No solvers linked yet.',
    },
    models: {
      selectToView: 'Select a model to view details',
    },
    benchmarks: {
      selectToView: 'Select a benchmark to view its leaderboard',
      noBenchmarks: 'No benchmarks defined yet. Seed some via the backend.',
      noEntries: 'No measurements recorded for this benchmark yet. Submit a BenchResult and reload.',
      protocol: 'Protocol',
      tolerance: 'Tolerance',
      lowerIsBetter: 'lower is better',
      higherIsBetter: 'higher is better',
      columns: { rank: '#', method: 'Method', bestValue: 'Best value', sources: 'Independent', confidence: 'Confidence', latest: 'Latest' },
      confidence: { verified: 'Verified', single: 'Single source', disputed: 'Disputed' },
      sourceTypes: { paper_reported: 'paper', self_run: 'self-run', third_party_reproduction: '3rd-party' },
      breakdownLabel: 'Sources',
    },
    search: {
      placeholder: 'e.g. Navier-Stokes, FNO, heat equation…',
      button: 'Search',
      results: (n: number) => `${n} result${n !== 1 ? 's' : ''}`,
      noResults: 'No nodes found. Try different keywords.',
    },
  },

  solver: {
    title: 'PDE Solver',
    subtitle: 'Browse available solvers or pick an example case to run PDEformer-2 inference.',
    tabs: {
      catalog: 'Solver catalog',
      solve: 'Run a solve',
    },
    catalog: {
      noSolvers: 'No solvers available. Make sure the solver service is running.',
      backend: 'backend',
      categoryLabels: { classical: 'Classical', machine_learning: 'ML', hybrid: 'Hybrid' },
    },
    cases: {
      selectCase: 'Select an example case',
      solver: 'Solver',
      resolution: 'Output resolution',
      resolutionSuffix: '× grid',
      runButton: 'Run solver',
      running: 'Running…',
      pdeLabel: 'PDE form',
      icLabel: 'Initial condition',
      bcLabel: 'Boundary condition',
      domainLabel: 'Domain',
      varLabel: 'Variables',
    },
    result: {
      timeStep: (cur: number, total: number) => `Time step: ${cur} / ${total}`,
      solutionField: 'Solution field',
      variable: 'Variable',
    },
  },

  skills: {
    title: 'Skills',
    subtitle: 'Browse agent skill documents and download skill packages for use in any agent framework.',
    selectSkill: 'Select a skill package to browse its docs',
    selectFile: 'Select a file on the left to preview',
    downloadButton: (name: string) => `Download ${name}`,
    downloading: 'Packing…',
    files: 'Files',
    noSkills: 'No skill packages found.',
    loadError: 'Failed to load',
  },

  common: {
    loading: 'Loading…',
    error: 'An error occurred',
  },
}

export type Locale = 'en' | 'zh'
