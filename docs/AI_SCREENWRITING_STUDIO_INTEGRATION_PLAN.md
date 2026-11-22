# AI 编剧工作站：集成实施计划

> 状态：Phase 0 已启动；本文档是集成分支的实施基线，不代表后续功能已经完成。
>
> 仓库：`lingjunhao002-ship-it/scriptcraft`
> 上游：`NoJuanNobody/scriptcraft`
> 集成分支：`integration/ai-screenwriting-studio`
> 目标平台：Windows 10/11 x64
> 基线：ScriptCraft v0.5.0，已审查提交 `fce1d3c57a1964742d989d6488df638fc97f089d`

## 1. 工程原则

1. 保留 ScriptCraft 作为原生桌面编辑器和剧本文档引擎，不迁移到 Electron、Tauri 或 Web UI。
2. 保留上游 `main` 的可同步性；所有产品功能只进入集成分支及其 feature 分支。
3. 优先复用 ScriptCraft 的命令引擎、MCP、CLI、Fountain/FDX、分页和 PDF 能力。
4. AI Runtime 与 Rust 编辑器分进程运行，使用本机 IPC/MCP；不把模型供应商或 Agent 框架耦合进剧本核心。
5. Agent 修改必须可追踪、可预览、可撤销；默认先生成建议，用户批准后再写入。
6. 用户 Skill 与平台内置 Skill 统一使用版本化清单、权限声明和工具白名单。
7. 保持原有 Final Draft/Fountain 兼容性；中文排版采用独立 Profile，不改变美式 Profile 的既有行为。
8. 第三方代码先做许可证、依赖、安全和维护状态审查，再迁移；不整仓复制。

## 2. 已验证的仓库基线

### 2.1 仓库与分支

- Fork 关系：`lingjunhao002-ship-it/scriptcraft` 是 `NoJuanNobody/scriptcraft` 的 GitHub Fork。
- 默认分支：`main`。
- 已创建 `integration/ai-screenwriting-studio`，从 `main` 建立。
- README、Cargo.toml、Cargo.lock、CI、MIT/Apache 许可证和 NOTICE 在两个分支的 Blob SHA 一致；集成分支尚未改动原始业务代码。
- 不向 `main` 直接提交产品改造。

### 2.2 技术结构

- Rust 2024 / Cargo Workspace，声明最低 Rust 版本 1.95。
- 桌面 UI：egui / eframe。
- 核心模块包括文档模型、几何、字体、Fountain、FDX、布局、PDF、导入、拼写、编辑引擎、协作、MCP 和 egui UI。
- 桌面入口：`apps/scriptcraft`；CLI：`apps/scriptcraft-cli`。
- Workspace 通过 `crates/*`、`apps/*` 和 `xtask` 组织。

### 2.3 可复用的 Agent 接口

现有 MCP 工具包括：

- `list_commands`
- `execute`
- `batch`
- `inspect_script`
- `get_fountain`
- `set_fountain`
- `reports`
- `screenshot`

MCP 的 `execute` 可以调用 ScriptCraft 命令注册表中的命令。Track Changes 已有 `track.setAuthor`、`track.toggle`、`track.list`、`track.accept`、`track.reject`、`track.acceptAll`、`track.rejectAll` 等命令。

重要约束：`set_fountain` 会整体替换剧本，只能用于明确的整稿导入/恢复流程；普通 Agent 编辑应优先使用局部命令和 Track Changes。

桌面控制通道当前绑定 `127.0.0.1`，通过随机密钥文件校验请求。Windows 上密钥文件 ACL、进程生命周期、端口冲突和异常退出恢复仍需专项验证。

## 3. 目标架构

```text
┌────────────────────────────────────────────────────────────┐
│                 ScriptCraft Desktop (Rust / egui)           │
│                                                            │
│  Project Workspace │ Script Editor │ AI Workbench          │
│  Story Bible       │ Outline       │ Review / Track Changes│
│        │                 │                  │               │
│        └─────────────────┴──────────────────┘               │
│                           │                                 │
│               Rust AI Bridge / Process Manager              │
└───────────────────────────┬────────────────────────────────┘
                            │ local IPC / MCP
┌───────────────────────────▼────────────────────────────────┐
│                  AI Runtime (Python)                       │
│                                                            │
│  LangGraph Orchestrator                                    │
│  ├── Chief Writer Agent                                    │
│  ├── Story Analyst / Structure Agent                       │
│  ├── Character Agent                                       │
│  ├── Dialogue Agent                                        │
│  ├── Script Doctor / Continuity Agent                      │
│  ├── Adaptation Pipeline                                   │
│  ├── Skill Registry / Loader                               │
│  └── Evaluators / Provenance                               │
│                                                            │
│  Model Provider Adapter │ Task Checkpoint │ Audit Log       │
└───────────────────────────┬────────────────────────────────┘
                            │ MCP client
┌───────────────────────────▼────────────────────────────────┐
│       scriptcraft-cli mcp --connect (existing interface)    │
│       ScriptCraft command engine / document model           │
└────────────────────────────────────────────────────────────┘
```

### 3.1 进程与通信

- Rust 桌面程序负责窗口、文档、编辑状态和用户审批。
- Python Runtime 负责 Agent 编排、模型调用、Skill 执行策略、小说改编和任务恢复。
- Runtime 通过 MCP 客户端连接 `scriptcraft-cli mcp --connect`，使用现有控制通道操作当前桌面会话。
- Rust 与 Runtime 之间采用仅本机可访问的进程通信；优先评估子进程 stdin/stdout JSON-RPC，禁止默认开放公网监听端口。
- 模型 API Key 使用 Windows Credential Manager 或等价安全存储，不写入项目文件、日志或 Skill 内容。
- 每个任务记录调用的模型、Skill 版本、工具、输入文档指纹、输出和用户审批结果。

### 3.2 项目数据布局（建议）

```text
MyScreenplayProject/
├── project.yaml
├── scripts/
│   ├── current.fountain
│   └── exports/
├── sources/
│   └── novels/
├── story-bible/
│   ├── characters/
│   ├── locations/
│   ├── world.md
│   └── timeline.md
├── outlines/
├── adaptations/
│   ├── plans/
│   ├── episodes/
│   └── provenance/
├── skills/
│   └── project-local/
└── .ai-studio/
    ├── state.sqlite
    ├── checkpoints/
    └── audit/
```

设计原则：剧本正文仍由 ScriptCraft 文件格式承载；项目数据库保存任务、索引、关系、锚点和执行状态。可读的 Story Bible 使用 Markdown/YAML 等可迁移格式，避免将项目知识锁死在单一数据库中。

## 4. 集成模块与迁移清单

| 模块 | 目标位置 | 策略 | 优先级 |
|---|---|---|---|
| 原生编辑器、文档模型、命令引擎 | 现有 `crates/*` | 原样保留 | P0 |
| Fountain / FDX / PDF | 现有 `crates/fountain`、`fdx`、`pdf` | 原样保留并加回归测试 | P0 |
| MCP / CLI / 控制通道 | 现有 `crates/mcp`、`apps/scriptcraft-cli`、桌面控制模块 | 复用，补安全与集成测试 | P0 |
| AI 进程管理和 UI 桥接 | 新增 Rust bridge 模块，扩展 `apps/scriptcraft`、`crates/ui-egui` | 新建薄适配层 | P1 |
| AI Runtime | `ai-runtime/` | 新建 Python 项目 | P1 |
| Skill Registry / Loader | `ai-runtime/skills/` | 新建统一清单、版本、权限和工具白名单 | P1 |
| Project Workspace | 新增项目模块与 `project.yaml` / SQLite | 新建 | P1 |
| Story Bible | `ai-runtime/story_bible/` + 项目数据目录 | 新建 | P1 |
| Anchor Resolver | Runtime 与文档桥接层 | 新建 | P1 |
| 中文字体与排版 Profile | `crates/fonts`、`geom`、`layout`、`pdf`、`ui-egui` | 扩展原模块 | P1 |
| 小说解析与改编 | `ai-runtime/adaptation/` | 从 Novel2Script 选择性移植 | P2 |
| Windows 安装与更新 | `.github/workflows`、安装器工程 | 新建 | P3 |

## 5. Agent 修改协议

### 5.1 安全修改流程

1. 读取当前文档版本、场景结构和目标段落。
2. 生成任务计划和拟修改范围。
3. 解析 Skill 权限，校验允许使用的工具。
4. 将修改写入独立建议或通过 Track Changes 标记作者。
5. 重新读取文档，检查目标是否仍然匹配。
6. 展示差异、理由和影响范围。
7. 用户接受后保留修改；拒绝则撤销对应修改。
8. 记录任务、Skill、模型、工具调用和审批审计信息。

### 5.2 锚点与并发

当前段落索引不能作为跨任务稳定 ID。第一阶段采用复合锚点：

- 场景标题与场景序号
- 段落元素类型
- 目标文本摘要/哈希
- 前后文摘要
- 任务开始时的文档指纹

提交前重新解析并验证锚点。若匹配不唯一、文档版本发生冲突或目标文本已变化，必须停止自动应用并请求用户确认。后续再评估给文档模型引入稳定 ID。

## 6. Skill 设计

### 6.1 统一 Skill Manifest

建议每个 Skill 至少包含：

- `id`、`name`、`version`、`description`
- 适用任务和输入/输出 schema
- Prompt / 指令文件
- 允许调用的 Agent 与工具
- 读写权限、网络权限和文件范围
- 依赖、许可证、来源和校验摘要

### 6.2 来源和安全策略

- 平台内置 Skill：随应用发布，带版本与签名/摘要。
- 用户 Skill：导入后进入项目或用户级目录，默认不可信。
- 第一版只执行声明式指令和白名单工具；不自动运行 Skill 中的任意 Python、PowerShell、EXE 或下载脚本。
- 任何文件写入、联网、批量修改和整稿替换都需要单独权限与用户确认。
- Skill 升级保留历史版本，任务检查点记录实际使用的版本。

## 7. 小说改编迁移策略

候选上游：`baoerger/novel-to-script`（MIT）。只选择性迁移解析、分块、信息抽取、场景识别、人物整合、时间线、分场生成和质量评估模块；不直接照搬其端到端任务调度器。

建议的改编图：

1. 导入与编码/章节检测
2. 章节分块和摘要
3. 人物、地点、组织、道具抽取
4. 人物去重和关系图谱
5. 事件时间线与因果链
6. 故事结构、主题和改编策略
7. 季/集/幕/场大纲
8. 场景生成与对白生成
9. 连贯性、人物声音、节奏和格式质检
10. 人工审阅、修订和导出

输出先进入独立 Adaptation Draft，不覆盖正式剧本。通过适配器转换成 Fountain/ScriptCraft 命令；保留每场、每个关键事件与原著章节/段落之间的 provenance 映射。

迁移前必须重新检查上游当前 HEAD、完整依赖清单、提示词、数据处理、许可证、网络行为和安全问题；MIT 许可证不等于其所有依赖都自动兼容。

## 8. 中文排版方案

已建立独立 `MainlandChina` 档案和首版规则文档 `docs/CHINESE_SCREENPLAY_PROFILE.md`。档案为可覆盖的产品默认值，不宣称行业只有一种固定模板。当前已新增 CJK 度量与初版行禁则换行模块，并通过命令注册 `script.profile.set`；完整屏幕/PDF渲染尚未接通。至少验证：

- 合法授权的 CJK 字体、字体回退与嵌入
- 中日韩字符宽度、标点挤压/禁则和 UAX #14 换行规则
- 场景标题、人物、对白、动作、括号、转场的中文行业格式
- 分页、页码、页眉、场景跨页和长对白
- Fountain/FDX 往返兼容
- PDF 复制文本、字体嵌入和视觉回归

不允许只通过给 UI 加中文翻译就宣称中文剧本排版完成。美式 Courier 分页行为必须有基准测试保护。

## 9. 分支策略

- `main`：上游同步基线，不直接开发产品功能。
- `integration/ai-screenwriting-studio`：产品集成主线。
- 以下 feature 分支均从集成分支创建，完成后以 PR 合并回集成分支：

| 分支 | 任务 |
|---|---|
| `feature/baseline-ci-license` | Windows CI、依赖许可证、SBOM |
| `feature/project-workspace` | 项目模型、文件布局、任务状态 |
| `feature/agent-mcp-bridge` | Runtime 启停、MCP 客户端、编辑器桥接 |
| `feature/skill-runtime` | Skill Manifest、导入、权限与版本 |
| `feature/story-bible` | 人物、世界观、时间线与检索 |
| `feature/cjk-screenplay-profile` | 字体、度量、换行、分页、PDF |
| `feature/novel-adaptation` | 小说解析、改编图和适配器 |
| `feature/windows-packaging` | Python Runtime 打包、安装、升级、签名 |

## 10. 测试与验收门槛

### 基线门槛

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets`
- `cargo test --workspace`
- `cargo build --workspace --all-targets`
- `cargo xtask ci`（以仓库当前 xtask 实际支持的命令为准）
- GitHub Actions 的 Windows runner 构建和测试通过。

### 产品门槛

- 旧有 Fountain / FDX 导入导出往返测试不回退。
- Agent 修改能被 Track Changes 标记、接受、拒绝和撤销。
- 任务中断后能从检查点恢复，不重复提交已审批修改。
- 文档发生变化时锚点失效检测有效。
- Skill 越权工具调用被拒绝；默认不执行任意脚本。
- 中文长文本、中文标点、混排数字/英文和跨页对白通过黄金样例测试。
- PDF 字体嵌入和文本可复制。
- Windows 安装、启动、升级、卸载、路径含中文和权限受限场景通过验证。

## 11. 许可证与供应链门槛

- ScriptCraft Workspace 声明 `MIT OR Apache-2.0`，仓库保留两份许可证、NOTICE 和 ATTRIBUTION。
- Courier Badi 字体采用 SIL OFL 1.1；词库及样例素材另有各自声明。
- Novel2Script 候选项目声明 MIT；移植时保留版权和许可证文件。
- 对 Cargo.lock 与 Python 锁文件做逐项许可证审查；生成 SBOM；拒绝未批准的 copyleft、来源不明或高风险依赖。
- 任何新增字体、图标、样例剧本、Prompt 或模型文件都必须记录来源、许可和再分发条件。

## 12. 当前状态与下一步

### 已完成

- [x] Fork 已验证，来源为 `NoJuanNobody/scriptcraft`。
- [x] 验证当前账号对 Fork 具有 Admin / Push 权限。
- [x] 创建 `integration/ai-screenwriting-studio`。
- [x] 验证集成分支与 `main` 的关键基线文件一致。
- [x] 完成第一轮 Rust Workspace、MCP、Track Changes、Windows CI 和许可证文件审查。
- [x] 建立本集成计划文档。

### 尚未完成

- [ ] 在 Windows runner 上执行实际构建并记录基线结果。
- [ ] 对全部 Cargo 依赖做许可证审查并生成 SBOM。
- [ ] 审查 Novel2Script 当前 HEAD 和完整 Python 依赖。
- [x] 选定中国大陆中文影视剧本为首版档案，并建立可覆盖的格式规范。
- [ ] 固定并嵌入 Noto Sans SC / Noto Serif SC 字体及许可证材料。
- [ ] 实现项目工作空间和 AI Runtime 最小纵向切片。
- [ ] 实现 Agent 建议、Track Changes 和用户审批闭环。

### 第一条可运行纵向切片

先实现一个可验证的小闭环，而不是同时铺开所有 Agent：

1. 打开一个现有 ScriptCraft 文档。
2. AI Runtime 通过 MCP 读取场景与人物报告。
3. 用户输入“改写第二场的对白，使人物语气更克制”。
4. Agent 生成局部修改计划和差异。
5. 通过 Track Changes 写入，作者标记为 `AI Dialogue Agent`。
6. UI 展示差异，用户接受或拒绝。
7. 任务和审计记录保存到项目目录。

只有该闭环在 Windows 上通过测试后，才扩展到多 Agent 和自动小说改编。
