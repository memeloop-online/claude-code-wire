# claude-code-wire

[English README](README.md)

[memeloop-token-center](https://github.com/memeloop-online/memeloop-token-center)
的 wire-shim 插件：把发往 anthropic-claude（Claude 订阅 OAuth）上游的
Anthropic Messages 请求，在网关出口处改写成与官方 Claude Code CLI 完全一致的
线上格式（wire format），让上游看到字节级一致的流量，无论实际客户端是什么。

转换逻辑逐比特对齐 [pi-black](https://github.com/paoloanzn/pi-black) 参考实现：

- **system 块手术**：剥离客户端自带的旧 billing / Agent SDK / legacy 块，
  然后在最前面插入新的 billing 块（含 cc_version 指纹与 cch 校验和）和
  Agent SDK 块；
- **cch 校验和**：对最终序列化 body（model 置空、删除 max_tokens）做 seeded
  XXH64，取低 20 bit 的 5 位 hex。宿主逐字节转发插件输出、绝不重新序列化，
  因此校验和在线上恒自洽；
- **规范请求头**：user-agent、x-app、x-claude-code-session-id、每请求随机的
  x-client-request-id、x-stainless-* SDK 指纹系列。宿主在应用这些头之前会先
  剥离客户端自带的指纹头，非官方 SDK（比如 Python 客户端）的值不会泄漏到上游；
- **可选身份元数据**：metadata.user_id 携带 device_id / account_uuid /
  稳定派生的 session_id。

**fail-closed**：请求无法安全改写、或插件被禁用时，宿主直接拒绝请求——
未改写的请求绝不会被发往订阅 OAuth 上游。

## 构建

    rustup target add wasm32-unknown-unknown
    ./build.sh        # 产出 plugin.wasm
    cargo test        # 原生单元测试（XXH64 向量、指纹、cch 自洽）

从本地 MTC 检出刷新 vendored WIT 定义：

    MTC_REPO=/path/to/memeloop-token-center ./build.sh

## 安装

把 plugin.json + plugin.wasm（+ 本 README）作为一个插件包目录，挂载到宿主的
插件目录（MTC_PLUGIN_DIR，或宿主 plugins/README.md 里的 OCI 安装流程）。
插件要求宿主的 WIT 0.3.0 wire-shim ABI。

## 配置

所有配置项都有对齐 Claude Code 2.1.258 的安全默认值，可全局或按租户覆盖：

| 配置项 | 默认值 | 说明 |
| --- | --- | --- |
| enabled | true | 关闭会拒绝请求（fail-closed），不会放行未改写流量 |
| claude_code_version | 2.1.258 | 嵌入 user-agent 与 cc_version 指纹 |
| entrypoint | sdk-cli | billing 的 cc_entrypoint 值 |
| device_id | — | 64 位小写 hex，取自该账号 ~/.claude.json 的 userID；必须与 account_uuid 成对配置 |
| account_uuid | — | Claude 账号 UUID |
| stainless_* | js / 0.60.0 / MacOS / arm64 / node / v22.14.0 / 600 | x-stainless-* 指纹取值；同一账号应保持稳定 |

配置变更在请求时生效（有界 5 秒缓存），版本号升级无需重启网关。

## 客户端侧遥测（网关拦不到的部分）

Claude Code 客户端会**直连** Statsig/Sentry 遥测服务，不经过网关。如果客户端
本身就是 Claude Code，请务必设置：

    export DISABLE_TELEMETRY=1
    export DISABLE_ERROR_REPORTING=1
    export CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1

或在 DNS/防火墙层屏蔽遥测域名。直接调网关 Anthropic 接口的非 Claude Code
客户端没有这个问题。

## 出口 IP

请求指纹再完美，源 IP 在不支持地区一样会被封号。请给 anthropic-claude 上游
账号配置支持地区的 SOCKS5 出口代理（登录时的 proxy_url，仅接受 socks5h +
私有 IP 字面量）。

## 风险声明

本插件让订阅流量在线上看起来与官方客户端一致。Anthropic 的风控策略随时可能
变化（新版本号、新指纹维度）；claude_code_version 需要跟随官方 Claude Code
版本更新。使用前请自行评估服务条款风险。

线上格式的逐字节细节见 [docs/protocol.zh-CN.md](docs/protocol.zh-CN.md)。
