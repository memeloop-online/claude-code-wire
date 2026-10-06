# Claude 插件安全边界与历史审计

## 当前版本说明（2026-10-06）

**当前 v1.0.1 不是下方历史中间版的“无条件拒绝”实现。** 当前 `src/lib.rs:32` 的 finalize 只删除顶层 `metadata.user_id`；没有该字段或配置显式关闭时返回原始请求字节。删除发生时重新序列化 JSON，不能承诺此分支字节不变。模型、system、messages、tools、工具关联及其他字段保留，不注入身份、计费、会话或请求头。

可核对的正式制品为 `ghcr.io/memeloop-online/claude-code-wire@sha256:3e4160580005b15c9db1e3bd419df79ee65de1d86a8fe1d4ec5aac4c88fe5906`，manifest 版本1.0.1/WIT0.3.0、`capabilities: []`。源码或本文更新都不等于该制品已在某个宿主生效；必须分别记录宿主的实际 inventory/revision。不要用旧v1.0.0或一个新构建文件替代已经核对的digest。

### 谁能看到什么、谁负责出口

| 边界 | 当前实现与限制 |
| --- | --- |
| WASM插件 | 接收宿主提供的最终请求体、租户/主体/key标识及非凭据头快照；请求中主动附带的源码、工具参数和秘密仍然可见。插件没有读取用户磁盘的接口，不等于请求里没有本地文件内容。 |
| 插件外联与持久化 | 当前guest源码不调用HTTP、KV、日志或文件接口，manifest无capabilities。受验MTC wire-shim linker拒绝HTTP/KV/random调用及guest日志，未注册WASI文件系统/网络接口；不把宿主自身的联网、数据库或归档权限说成插件权限。 |
| 宿主OAuth与账号 | 登录、token交换/刷新/撤销、profile、models及推理出口由MTC宿主管理，不经过这个finalize插件。宿主存储访问/刷新令牌；删除metadata.user_id不会删除账号凭据或隐藏上游账号身份。 |
| 客户端与浏览器 | 客户端自行读取目录、运行工具、上传上下文，或通过另一条连接发送遥测，均不受该插件控制。用户仍须决定客户端目录/工具权限及浏览器登录出口；不能承诺绝无遥测或本地文件风险。 |
| 宿主归档 | 只净化某次上游请求，不会追溯清除原始请求、响应、日志或宿主归档。不能将插件启用等同于无内容留存。 |

对应宿主源码可按受验提交独立核对：[wire-shim imports](https://github.com/memeloop-online/memeloop-token-center/blob/f391f1bf63293568b257933923a250b06276611b/src/plugin.rs#L2003)、[Claude OAuth端点与scope](https://github.com/memeloop-online/memeloop-token-center/blob/f391f1bf63293568b257933923a250b06276611b/src/oauth/claude.rs#L23)、[账号代理与远端DNS边界](https://github.com/memeloop-online/memeloop-token-center/blob/f391f1bf63293568b257933923a250b06276611b/src/network.rs#L213)。OAuth的远端DNS路径与通用推理的目的地验证不是同一函数，不能据此承诺所有请求都不做本地DNS查询。

### 用户本人登录前的判定

登录与插件启用是不同操作。插件不是OAuth登录的前置条件，也不会替用户授权；若验收后宿主已恢复empty inventory，不能告诉用户请求净化仍在运行。用户只应使用已核验的MTC入口和既有合法操作身份，在该入口发起自己的登录，再在所返回的官方授权页面确认账号及完整scope，将本次产生的 `code#state` 只提交回同一登录会话。不要将code、session token、访问/刷新令牌发给审计人员或贴进日志。

宿主明确选择的账号代理只约束宿主请求，不能替代浏览器出口配置。未配置代理的路径可能使用宿主直连；明确代理失败不得按“兼容”理由切换到另一个未批准出口。是否能到达授权/token/profile/model服务需要独立环境证据，不能以插件CI或安装成功代替。

同样，HTTP200不证明调用者具有合法观测身份。角色停用占位值不是凭据；不得将它用于metrics或其他接口。若曾使用这种值取得指标，该部分证据必须标为 `AUTH_INVALID`、保留原始响应并撤销对应通过结论，待宿主拒绝占位值及合法身份验证后仅补缺失观测，不为此重复已经成功的发布/恢复生命周期。

本说明是源码/边界修正，不声明完成用户OAuth、真实订阅流量、所有节点加载或客户端遥测验收。GHA编译测试、签名制品核对、集群stage、host发布/回滚和gateway最终wire是不同层证据，须分别记录。

## 以下为2026-09-30历史审计，不是当前验收结论

## 范围与来源

- 独立仓库：`memeloop-online/claude-code-wire`，审计基线 `7c3d15e`（`v1.0.0`，`master`）。本 PR 只改该仓库；未改 MTC 主仓库、GitOps 或生产发布候选 `da263`。
- 原版 `plugin.wasm` 是已跟踪二进制，基线无法仅凭源码证明其与 `src/lib.rs` 一致。旧 release 和旧安装包必须视为未验收制品；本 PR 删除仓库中的二进制，并停用自动发布工作流。

## 已证实风险

1. 基线 `src/lib.rs` 明确把任意客户端请求改写为 Claude Code 风格：注入 billing/Agent SDK 系统块、校验值、`metadata.user_id` 与 `user-agent`、`x-app`、`x-claude-code-session-id`、`x-stainless-*` 等指纹。`README.md` 明示让上游无法区分来源，并指导以 SOCKS 代理选择区域。此行为属于客户端身份伪装和风控规避，不应继续运行。
2. `wire-shim-v1.finalize` 收到完整序列化请求体和非凭据头快照，因此插件能看到用户提示、工具参数以及客户端主动放入请求的源码。基线代码没有本地文件读取或外联调用；但“不会读取本地源码”不能推导为“请求内绝无源码”。
3. 基线 manifest 只声明 `random` 能力，无 `http`、`kv`、`log` 能力。MTC 宿主按能力限制这些 host 调用；WASM guest 没有普通本地文件系统接口。静态审计未发现插件自身的遥测或外传路径，但旧二进制与运行中制品尚未做隔离验证，不能作无泄漏保证。
4. OAuth 登录与 SOCKS5H 代理位于 MTC 宿主，不在此插件内。宿主 `src/api/upstreams/oauth_claude.rs` 校验代理 URL；`src/oauth/claude.rs` 的 OAuth 网络调用沿用所选代理或默认出口。插件修改不能证明全程代理不回退、远端 DNS、无敏感日志；这些须在隔离环境按真实部署版本验收。不得由审计人员代用户登录或获取凭据。
5. 旧文档声称配置可在约 5 秒缓存后热生效，属于运行时配置路径，不等于源代码热加载。宿主插件包会被读取为组件并可通过插件生命周期加载新修订；必须验收每个节点的组件 digest、版本和旧制品撤除，不能仅看 Git HEAD。

## 历史中间版的边界（已被v1.0.1替代）

- 当时中间版的 `finalize` 无条件拒绝；当前v1.0.1已改为本文开头说明的有限净化，不能沿用“总是拒绝”测试结论。
- manifest 移除 `random` 和所有身份/指纹配置；移除旧协议说明、预编译 WASM 和自动 release 工作流。
- CI 仅证明新源码可测试、可构建；不触及生产或用户账号。旧 `v1.0.0` tag/release 不会因 PR 自动消失，必须单独盘点与隔离。

## 历史待验项目（状态只反映当时，不可当作当前通过/未通过）

- [ ] PR 未合并、未部署；生产 `da263`、GitOps digest、各节点插件包及旧 release 均已盘点并隔离旧制品。
- [ ] GitHub Actions 的 native test 和 WASM build 对本 PR 的**确切 commit SHA** 全绿；审阅 CI 日志、产物 SHA256、manifest 能力与编译输入，确认没有额外网络/文件读取代码。
- [ ] 在无真实账号、无真实源码的隔离环境，用合成请求验证新组件总是拒绝且宿主不回退转发；无伪造身份头和上游流量。
- [ ] 对拟授权的宿主版本独立验证 OAuth 授权 URL、回调 state/PKCE、令牌存储/日志遮蔽、代理校验、SOCKS5H 远端 DNS、失败时无直连回退，并由用户本人在官方页面完成授权。
- [ ] 对隔离环境抓取 DNS/网络出口与日志，确认请求内容、源码片段、凭据和客户端遥测只流向用户明确批准的目的地；客户端直连遥测需要单独核查，不受此插件控制。
- [ ] 记录所有节点的实际组件 digest、加载/修订事件和配置快照；确认没有旧二进制残留或热切换到旧版的路径，再由用户决定是否授权。不要把 CI 通过等同于可使用订阅。
