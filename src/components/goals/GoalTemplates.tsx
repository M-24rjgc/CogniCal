import { GoalTemplate } from '../../types/goal';

export const GOAL_TEMPLATES: GoalTemplate[] = [
  {
    id: 'sequential-project',
    name: '顺序项目',
    description: '必须按顺序完成的任务，一个接一个',
    type: 'sequential',
    taskStructure: [
      {
        title: '规划和研究',
        description: '定义需求并研究解决方案',
        estimatedMinutes: 120,
      },
      {
        title: '设计和架构',
        description: '创建设计文档和架构',
        dependencies: [0],
        estimatedMinutes: 180,
      },
      {
        title: '实现',
        description: '构建解决方案',
        dependencies: [1],
        estimatedMinutes: 480,
      },
      {
        title: '测试和质量保证',
        description: '测试并验证实现',
        dependencies: [2],
        estimatedMinutes: 120,
      },
      {
        title: '部署和文档',
        description: '部署到生产环境并编写文档',
        dependencies: [3],
        estimatedMinutes: 60,
      },
    ],
  },
  {
    id: 'parallel-project',
    name: '并行项目',
    description: '可以同时进行的独立任务',
    type: 'parallel',
    taskStructure: [
      {
        title: '前端开发',
        description: '构建用户界面组件',
        estimatedMinutes: 360,
      },
      {
        title: '后端开发',
        description: '实现 API 和业务逻辑',
        estimatedMinutes: 360,
      },
      {
        title: '数据库设计',
        description: '设计并实现数据库架构',
        estimatedMinutes: 180,
      },
      {
        title: '文档编写',
        description: '编写用户和开发者文档',
        estimatedMinutes: 120,
      },
      {
        title: '集成和测试',
        description: '集成所有组件并测试',
        dependencies: [0, 1, 2],
        estimatedMinutes: 240,
      },
    ],
  },
  {
    id: 'milestone-based',
    name: '里程碑项目',
    description: '围绕关键里程碑和交付成果组织的项目',
    type: 'milestone-based',
    taskStructure: [
      {
        title: '里程碑 1：项目启动',
        description: '初步规划和团队协调',
        estimatedMinutes: 120,
      },
      {
        title: '里程碑 2：MVP 开发',
        description: '构建最小可行产品',
        dependencies: [0],
        estimatedMinutes: 600,
      },
      {
        title: '里程碑 3：Beta 版本发布',
        description: '发布测试版本',
        dependencies: [1],
        estimatedMinutes: 240,
      },
      {
        title: '里程碑 4：正式发布',
        description: '最终发布到生产环境',
        dependencies: [2],
        estimatedMinutes: 180,
      },
    ],
  },
  {
    id: 'learning-goal',
    name: '学习目标',
    description: '学习新技能或技术的结构化方法',
    type: 'sequential',
    taskStructure: [
      {
        title: '基础知识',
        description: '学习基本概念和术语',
        estimatedMinutes: 240,
      },
      {
        title: '动手实践',
        description: '完成教程和练习',
        dependencies: [0],
        estimatedMinutes: 480,
      },
      {
        title: '构建示例项目',
        description: '在实际项目中应用知识',
        dependencies: [1],
        estimatedMinutes: 600,
      },
      {
        title: '高级主题',
        description: '探索高级功能和最佳实践',
        dependencies: [2],
        estimatedMinutes: 360,
      },
    ],
  },
  {
    id: 'product-launch',
    name: '产品发布',
    description: '发布新产品或功能的完整工作流程',
    type: 'milestone-based',
    taskStructure: [
      {
        title: '市场调研',
        description: '研究目标市场和竞争对手',
        estimatedMinutes: 240,
      },
      {
        title: '产品设计',
        description: '创建产品规格和设计',
        dependencies: [0],
        estimatedMinutes: 360,
      },
      {
        title: '开发',
        description: '构建产品',
        dependencies: [1],
        estimatedMinutes: 960,
      },
      {
        title: '营销材料',
        description: '创建营销内容和材料',
        dependencies: [1],
        estimatedMinutes: 240,
      },
      {
        title: 'Beta 测试',
        description: '与测试用户测试并收集反馈',
        dependencies: [2],
        estimatedMinutes: 180,
      },
      {
        title: '发布活动',
        description: '执行发布营销活动',
        dependencies: [3, 4],
        estimatedMinutes: 120,
      },
    ],
  },
  {
    id: 'content-creation',
    name: '内容创作',
    description: '创建和发布内容的工作流程',
    type: 'sequential',
    taskStructure: [
      {
        title: '主题研究',
        description: '研究并选择内容主题',
        estimatedMinutes: 60,
      },
      {
        title: '创建大纲',
        description: '创建详细的内容大纲',
        dependencies: [0],
        estimatedMinutes: 30,
      },
      {
        title: '撰写草稿',
        description: '撰写内容的初稿',
        dependencies: [1],
        estimatedMinutes: 180,
      },
      {
        title: '审阅和编辑',
        description: '审阅并编辑内容以确保质量',
        dependencies: [2],
        estimatedMinutes: 90,
      },
      {
        title: '发布和推广',
        description: '发布内容并在各渠道推广',
        dependencies: [3],
        estimatedMinutes: 60,
      },
    ],
  },
  {
    id: 'event-planning',
    name: '活动策划',
    description: '从头到尾组织和执行活动',
    type: 'parallel',
    taskStructure: [
      {
        title: '定义活动目标',
        description: '设定目标和成功标准',
        estimatedMinutes: 60,
      },
      {
        title: '场地选择',
        description: '研究并预订场地',
        dependencies: [0],
        estimatedMinutes: 120,
      },
      {
        title: '演讲者/嘉宾预订',
        description: '确定并确认演讲者或表演者',
        dependencies: [0],
        estimatedMinutes: 180,
      },
      {
        title: '营销和推广',
        description: '创建宣传材料和活动',
        dependencies: [0],
        estimatedMinutes: 240,
      },
      {
        title: '后勤规划',
        description: '规划餐饮、设备和后勤',
        dependencies: [1],
        estimatedMinutes: 180,
      },
      {
        title: '活动执行',
        description: '举办活动',
        dependencies: [1, 2, 3, 4],
        estimatedMinutes: 480,
      },
      {
        title: '活动后跟进',
        description: '发送感谢信并收集反馈',
        dependencies: [5],
        estimatedMinutes: 120,
      },
    ],
  },
];
