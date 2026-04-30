// Shared shape — keeps both locales in sync without literal-type conflicts
export interface Translations {
  nav: { knowledge: string; solver: string; skills: string }
  lang: { toggle: string }
  knowledge: {
    title: string; subtitle: string
    tabs: { search: string; recent: string; methods: string }
    search: {
      placeholder: string; button: string
      allPdeTypes: string; allMethods: string; allDomains: string
      results: (n: number) => string; noResults: string
    }
    recent: { filterLabel: string; allDomains: string; noResults: string }
    methods: {
      tabs: { browse: string; compare: string; recommend: string }
      allCategories: string; selectToView: string; relatedMethods: string
      categoryLabels: { classical: string; ml: string; hybrid: string }
      compare: { methodA: string; methodB: string; vs: string; button: string; notFound: string }
      recommend: {
        pdeType: string; domain: string; domainPlaceholder: string
        constraints: string; button: string; selectPde: string; score: string
      }
    }
    paper: { etAl: string; showMore: string; showLess: string; viewPaper: string }
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
    subtitle: '搜索 PDE 论文、浏览数值方法、获取求解器推荐。',
    tabs: {
      search: '论文搜索',
      recent: '最新论文',
      methods: '方法',
    },
    search: {
      placeholder: '例如：物理信息神经网络 反问题',
      button: '搜索',
      allPdeTypes: '全部 PDE 类型',
      allMethods: '全部方法',
      allDomains: '全部领域',
      results: (n: number) => `${n} 条结果`,
      noResults: '未找到论文，请尝试放宽查询条件。',
    },
    recent: {
      filterLabel: '按领域过滤：',
      allDomains: '全部领域',
      noResults: '未找到论文。',
    },
    methods: {
      tabs: {
        browse: '浏览',
        compare: '对比',
        recommend: '推荐',
      },
      allCategories: '全部类别',
      selectToView: '选择一个方法查看详情',
      relatedMethods: '相关方法',
      categoryLabels: { classical: '经典', ml: 'ML', hybrid: '混合' },
      compare: {
        methodA: '方法 A',
        methodB: '方法 B',
        vs: '对比',
        button: '对比',
        notFound: '未找到一个或两个方法',
      },
      recommend: {
        pdeType: 'PDE 类型',
        domain: '应用领域（可选）',
        domainPlaceholder: '例如 fluid_dynamics',
        constraints: '约束条件',
        button: '获取推荐',
        selectPde: '请选择…',
        score: '得分',
      },
    },
    paper: {
      etAl: '等',
      showMore: '展开',
      showLess: '收起',
      viewPaper: '查看论文',
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
    subtitle: 'Search PDE papers, explore numerical methods, and get solver recommendations.',
    tabs: {
      search: 'Search papers',
      recent: 'Recent papers',
      methods: 'Methods',
    },
    search: {
      placeholder: 'e.g. physics-informed neural networks inverse problem',
      button: 'Search',
      allPdeTypes: 'All PDE types',
      allMethods: 'All methods',
      allDomains: 'All domains',
      results: (n: number) => `${n} result${n !== 1 ? 's' : ''}`,
      noResults: 'No papers found. Try broadening your query.',
    },
    recent: {
      filterLabel: 'Filter by domain:',
      allDomains: 'All domains',
      noResults: 'No papers found.',
    },
    methods: {
      tabs: {
        browse: 'Browse',
        compare: 'Compare',
        recommend: 'Recommend',
      },
      allCategories: 'All categories',
      selectToView: 'Select a method to view details',
      relatedMethods: 'Related methods',
      categoryLabels: { classical: 'Classical', ml: 'ML', hybrid: 'Hybrid' },
      compare: {
        methodA: 'Method A',
        methodB: 'Method B',
        vs: 'vs',
        button: 'Compare',
        notFound: 'One or both methods not found',
      },
      recommend: {
        pdeType: 'PDE type',
        domain: 'Domain (optional)',
        domainPlaceholder: 'e.g. fluid_dynamics',
        constraints: 'Constraints',
        button: 'Get recommendations',
        selectPde: 'Select…',
        score: 'score',
      },
    },
    paper: {
      etAl: 'et al.',
      showMore: 'Show more',
      showLess: 'Show less',
      viewPaper: 'View paper',
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
