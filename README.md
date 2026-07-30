# NuoField（傩场）

NuoField 是一个面向人与 AI Agent 协作的自托管工作空间。

它坚持三条产品原则：

1. Agent 是拥有独立身份、权限、生命周期和审计记录的一等成员。
2. 数据、密钥、记忆、索引、备份和模型出口完全由部署者控制。
3. 国内云模型、本地模型和企业私有推理服务是默认运行生态。

## 当前实现

首个可运行切片已经覆盖：

- 人与 Agent 的独立成员身份；
- 低风险任务直接进入待执行状态；
- 高风险任务必须经人批准；
- 只有被指派的 Agent 可以开始和完成任务；
- 模型调用目标与出口类型进入事件记录；
- 完成任务必须附带证据；
- 所有事件写入部署者本地的追加式 JSONL 文件；
- 审计记录通过 SHA-256 哈希链检测篡改；
- 工作空间状态和完整审计记录可以通过 API 或 CLI 导出。

当前版本是早期技术切片，身份认证、加密签名、细粒度权限和生产级并发存储仍在
后续里程碑中。请勿将服务直接暴露在不可信网络。

## 快速开始

需要 Rust 1.88 或更新版本。

```bash
cargo run -p nuofield-server
```

服务默认监听 `127.0.0.1:3000`，数据写入 `./data/events.jsonl`。

另一个终端可以使用 Agent 优先 CLI：

```bash
cargo run -p nuofield-cli --bin nuofield -- id
cargo run -p nuofield-cli --bin nuofield -- append --file event.json
cargo run -p nuofield-cli --bin nuofield -- export --output export.json
```

事件格式与完整闭环示例见 [协议文档](docs/PROTOCOL.md)。

也可以使用 Docker：

```bash
docker compose up --build
```

## 仓库结构

```text
crates/
  nuofield-core      零 I/O 领域模型、身份与任务状态机
  nuofield-store     部署者自有的追加式事件存储与审计链
  nuofield-server    HTTP API、状态投影与运行入口
  nuofield-cli       面向 Agent 和自动化的命令行入口
docs/
  ARCHITECTURE.md    架构边界与写入管线
  PROTOCOL.md        事件格式、API 与端到端示例
  DEPLOYMENT.md      本地和容器部署
  THREAT_MODEL.md    信任边界、已实现控制与已知缺口
```

## 开发

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

贡献说明见 [CONTRIBUTING.md](CONTRIBUTING.md)，安全问题见
[SECURITY.md](SECURITY.md)。

## 许可证

[MIT](LICENSE) — 可以使用、修改、分发和商用，但必须保留版权与许可声明。
