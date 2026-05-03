// Shared shape — keeps both locales in sync without literal-type conflicts
export interface Translations {
  nav: { knowledge: string; solver: string; skills: string }
  lang: { toggle: string }
  knowledge: {
    title: string; subtitle: string
    tabs: { equations: string; models: string; search: string }
    equations: { selectToView: string }
    models: { selectToView: string }
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
    form: {
      presets: string; solver: string; bc: string
      bcOptions: { unspecified: string; periodic: string; dirichlet: string; neumann: string }
      equation: string; equationPlaceholder: string; equationHint: string; equationHint2: string
      resolution: string; resolutionSuffix: string; sendIc: string; runButton: string
    }
    result: { timeStep: (cur: number, total: number) => string; solutionField: string }
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
    knowledge: '知识库',
    solver: '求解器',
    skills: 'Skills',
  },
  lang: {
    toggle: 'English',
  },

  // ── Knowledge page ──────────────────────────────────────────────────────────
  knowledge: {
    title: '知识库',
    subtitle: '浏览 PDE 方程、AI 模型与数值方法，探索知识图谱中的关系。',
    tabs: {
      equations: '方程',
      models: '模型与方法',
      search: '搜索',
    },
    equations: {
      selectToView: '选择一个方程查看详情',
    },
    models: {
      selectToView: '选择一个模型查看详情',
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
    subtitle: '浏览可用求解器，或提交 PDE 获取数值解。',
    tabs: {
      catalog: '求解器目录',
      solve: '运行求解',
    },
    catalog: {
      noSolvers: '暂无可用求解器，请确认求解器服务正在运行。',
      backend: '后端',
      categoryLabels: { classical: '经典', machine_learning: 'ML', hybrid: '混合' },
    },
    form: {
      presets: '预设',
      solver: '求解器',
      bc: '边界条件',
      bcOptions: {
        unspecified: '— 不指定 —',
        periodic: '周期性',
        dirichlet: 'Dirichlet',
        neumann: 'Neumann',
      },
      equation: '方程',
      equationPlaceholder: '例如：u_t + (u^2)_x + (-0.3*u)_y = 0',
      equationHint: '使用',
      equationHint2: '等符号',
      resolution: '输出分辨率',
      resolutionSuffix: '× 网格',
      sendIc: '发送初始条件（高斯脉冲）',
      runButton: '运行求解器',
    },
    result: {
      timeStep: (cur: number, total: number) => `时间步：${cur} / ${total}`,
      solutionField: '解场',
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
    knowledge: 'Knowledge Base',
    solver: 'Solver',
    skills: 'Skills',
  },
  lang: {
    toggle: '中文',
  },

  knowledge: {
    title: 'Knowledge Base',
    subtitle: 'Explore PDE equations, AI models, and numerical methods in the knowledge graph.',
    tabs: {
      equations: 'Equations',
      models: 'Models & Methods',
      search: 'Search',
    },
    equations: {
      selectToView: 'Select an equation to view details',
    },
    models: {
      selectToView: 'Select a model to view details',
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
    subtitle: 'Browse available solvers or submit a PDE to get a numerical solution.',
    tabs: {
      catalog: 'Solver catalog',
      solve: 'Run a solve',
    },
    catalog: {
      noSolvers: 'No solvers available. Make sure the solver service is running.',
      backend: 'backend',
      categoryLabels: { classical: 'Classical', machine_learning: 'ML', hybrid: 'Hybrid' },
    },
    form: {
      presets: 'Presets',
      solver: 'Solver',
      bc: 'Boundary condition',
      bcOptions: {
        unspecified: '— unspecified —',
        periodic: 'Periodic',
        dirichlet: 'Dirichlet',
        neumann: 'Neumann',
      },
      equation: 'Equation',
      equationPlaceholder: 'e.g. u_t + (u^2)_x + (-0.3*u)_y = 0',
      equationHint: 'Use',
      equationHint2: 'etc.',
      resolution: 'Output resolution',
      resolutionSuffix: '× grid',
      sendIc: 'Send initial condition (Gaussian blob)',
      runButton: 'Run solver',
    },
    result: {
      timeStep: (cur: number, total: number) => `Time step: ${cur} / ${total}`,
      solutionField: 'Solution field',
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
