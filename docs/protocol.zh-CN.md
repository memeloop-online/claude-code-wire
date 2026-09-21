# Claude Code 线上格式细节（中文）

[English](protocol.md)

本文档逐条描述插件的输出，对照 pi-black 的 src/claude-code-protocol.ts
（Claude Code 2.1.258）核验。

## 请求体

### system 数组

改写后的请求总是以两个 system 块开头，顺序固定：

1. billing 块（text 块）：

       x-anthropic-billing-header: cc_version=<VERSION>.<FP>; cc_entrypoint=<ENTRYPOINT>; cch=<CCH>;

   - VERSION：claude_code_version 配置（默认 2.1.258）。
   - FP：SHA-256("59cf53e54c78" + selected + VERSION) 的前 3 个 hex 字符，
     其中 selected 是第一条 user 消息文本的 UTF-16 code unit 下标 4、7、20
     处的字符（缺位补 '0'；被截断的代理对字符按 TextEncoder 行为编码为
     U+FFFD）。
   - ENTRYPOINT：entrypoint 配置（默认 sdk-cli）。
   - CCH：见下。
2. Agent SDK 块（text 块）：

       You are a Claude agent, built on Anthropic's Claude Agent SDK.

插入前会先剥离头部已有的 billing + Agent SDK 块对，或单个 legacy
"You are Claude Code, Anthropic's official CLI for Claude." 块。其余 system
块原样保留、顺序不变。

system 字段不是数组时拒绝请求（fail-closed）。

### cch 校验和

在最终序列化之后、对实际发出的字节计算：

1. 解析 body，保持 key 的插入顺序。
2. normalized = 深拷贝；normalized.model = ""；删除 normalized.max_tokens。
3. hash = XXH64(serialize(normalized)，seed = 0x4d659218e32a3268)。
4. cch = (hash 与 0xfffff) 的 5 位小写 hex，左补零。
5. 用该值替换 billing 块里的 cch=00000 占位符。

宿主逐字节转发插件输出，因此按线上字节重算（还原占位符后）的校验方
永远能对上。

### metadata.user_id（可选）

仅当 device_id 与 account_uuid 同时配置时写入：

    metadata.user_id = "{\"device_id\":\"...\",\"account_uuid\":\"...\",\"session_id\":\"...\"}"

session_id 派生规则：SHA-256(tenant_id ++ key_id) 的前 16 字节格式化为
UUIDv4 形状——同一 API key 稳定，不同 key 不同。

## 请求头

由插件设置（宿主会先剥离客户端自带的同名头）：

- user-agent: claude-cli/<VERSION> (external, <ENTRYPOINT>)
- x-app: cli
- x-claude-code-session-id: 派生的 session id
- x-client-request-id: 每请求随机 UUID（同一请求的宿主上游重试间保持稳定）
- x-stainless-lang / -package-version / -os / -arch / -runtime /
  -runtime-version / -timeout：来自配置
- x-stainless-retry-count: 0；宿主重试时改写为从 0 起的尝试序号

anthropic-version 与 anthropic-beta 由宿主核心处理（客户端的 beta 保留并
合并 oauth-2025-04-20）。

## 失败语义

以下情况插件报错、宿主拒绝请求：body 不是 JSON object；缺 model 或
max_tokens；system 不是数组；配置非法（未知字段、身份格式错误）；
enabled 为 false。
