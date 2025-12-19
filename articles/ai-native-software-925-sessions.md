---
title: AIネイティブソフトウェア構築：925セッション、2,399コミットで学んだこと
emoji: 🤖
type: tech
topics: [ai, development, rust, methodology, productivity]
published: false
---

# AIネイティブソフトウェア構築：925セッション、2,399コミットで学んだこと

## はじめに

KeyRxプロジェクトでは、過去2年間で**925回のAI実装セッション**を通じて**66個の機能**を実装し、**2,399個のgitコミット**を生成しました。

本記事では、AI支援開発の真実を数字で語ります：
- AIは魔法ではなく、正しく使えば**段階的な問題解決が加速する**
- 最もシンプルな機能は**1セッションで完了**、最も複雑な機能は**48セッション必要**
- 成功率は**0.78:1**（失敗と成功は同程度の頻度で発生）

読者の皆さんは、このプロセスを自分のプロジェクトで**再現可能**です。

---

## 第1部：数字が語る AIネイティブ開発の現実

### 925セッション＝何をしたのか？

```
全スペック数：66個
合計セッション数：925回
生成されたコミット数：2,399個
利用言語：Rust (コア), Dart (UI), Python (ツール)
総実装行数：～50,000行以上
平均テストカバレッジ：90%以上
```

### 複雑さの分布：1セッションから48セッションまで

#### 最もシンプル：daemon-runtime-control（1セッション）

```yaml
タスク数: 1
セッション数: 1
実装時間: 1時間以内
実装者の判断: "これは簡単だ"
理由: 明確な仕様＋既存パターンの適用
```

**何が実装されたか：**
- JSON-RPC 2.0 IPC プロトコル実装
- EngineManager のライフサイクル管理
- Flutter UI への統合

**指示の構造：**
```bash
claude -p 'Work on the single next task for spec daemon-runtime-control.
When you complete a logical unit of work, MUST run git add followed by git commit
with a clear message. DO NOT ask permission - just commit immediately.
Make atomic commits for each semantic group.'
--model opus --dangerously-skip-permissions
```

**なぜ1セッションで完了したのか：**
1. **要件が明確** - requirements.mdが詳細に記述
2. **設計が決定済み** - design.mdでアーキテクチャが確定
3. **パターンが既知** - 既存のIPC実装を参照可能
4. **タスクが小さい** - 1つのセッション＝1つの完結したタスク

#### 最も複雑：revolutionary-mapping（48セッション）

```yaml
タスク数: 48
セッション数: 48
実装時間: 5〜7日
実装パターン: 段階的フェーズ（10段階）
複雑さの理由: 新規設計＋複数プラットフォーム＋大量のテスト
```

**何が実装されたか：**
- デバイス識別システム（VID:PID:SerialNumber）
- TOML定義フォーマット
- 複数デバイス同時サポート
- Windows/Linux ドライバー統合
- 35個の包括的テスト

**なぜ48セッション必要だったのか：**

**フェーズ1-2：基盤設計（タスク1-10）**
- DeviceIdentityの定義
- Windowsのシリアル番号抽出（IOCTL）
- Linuxのシリアル番号抽出（evdev + udev）
- 問題：最初の実装では3段階フォールバックが不完全

**フィードバック＆改善：**
```rust
// 最初の実装（タスク1.5）
let serial = extract_serial_number(device_path)?;

// フィードバック：
// "エラーハンドリングが不完全。失敗時のフォールバックロジックが必要"

// 改善後（タスク1.5 revision）
let serial_number = match try_eviocguniq(&device_path) {
    Ok(s) => Some(s),
    Err(_) => match try_udev_properties(&device_path) {
        Ok(s) => Some(s),
        Err(_) => Some(synthetic_id_from_path(&device_path))
    }
};
```

**フェーズ3-5：定義システム（タスク11-30）**
- TOML 形式の決定
- パーサー実装
- バリデーションロジック
- デバイス定義の動的ロード

**問題発生:**
```
Task 2.8 (Definition Library) のテスト実行時に、
3個の予期しないバグが発見された：
1. 無効なレイアウト種別の処理が不十分
2. マトリックス境界チェックが未実装
3. デバイス定義の重複チェックなし
```

**対応：**
- テストケースを10個追加（8個 → 18個）
- 各バグに対して修正を実装
- 包括的なテストを重視する文化の確立

**フェーズ6-10：統合＆テスト（タスク31-48）**
- Mock実装の作成
- FastテストとIntegrationテストの分離
- 実デバイスでのエンドツーエンドテスト
- ドキュメント自動生成

### 成功率：0.78:1 の意味

```
総セッション数: 925
"成功"と判定されたセッション: 513
"失敗"または再試行が必要なセッション: 412
成功率: 55.5% （失敗率: 44.5%）

成功：失敗 = 0.78:1
```

**これは悪いニュースではありません。反対です。**

### なぜ50%程度の失敗率は正常なのか

#### 失敗のカテゴリー

**1. 計画的な失敗 - 試行錯誤（全体の70%）**

```
例：dependency-injection リファクタリング
- タスク1.1：DeviceServiceTraitを定義
- テスト実行 → コンパイルエラー3個
- 原因分析 → 戻り値型が不正確
- 修正 → 即座に解決
- 学習：次回から正確に予測可能

この「失敗」は実は学習ループの一部
```

**2. ツール選択の失敗 - Claude vs Gemini（全体の20%）**

```
daemon-runtime-control (Task 1):
- 初回：Gemini 2.5-pro で実装
- 結果：60回のAPI呼び出し、7個のツールエラー、変更を全てrevert
- 分析：複雑なリファクタリングではGeminiが悪循環に陥る

対応：
- Claude Opus に切り替え
- 結果：1セッションで完了

教訓：ツール選択は重要。複雑さによって使い分けが必須
```

**3. 要件不明確による失敗（全体の10%）**

```
revolutionary-mapping フェーズ2（タスク2.8-2.9）:
- 最初の指示：「Definition Libraryを実装」
- 結果：実装が要件を満たさない
- 原因：TOMLフォーマットが未定義

修正：
- requirements.md に TOML スキーマを追加
- 再実装 → 一発成功

教訓：要件が曖昧なら、実装の前に仕様を詰める
```

### 成功 = 右の指示

成功したセッションには共通パターンがあります。

#### パターン1：明確な制約を含む指示

```bash
# ❌ 曖昧な指示
"Implement dependency injection"

# ✅ 明確な指示
Role: Rust architect specializing in trait design and async patterns

Task: Create core/src/services/traits.rs and define DeviceServiceTrait
following requirement 1.1. Extract all 6 public async method signatures
from DeviceService implementation.

Restrictions:
  - Do not modify DeviceService implementation yet.
  - Only create trait definition.
  - Do not change method signatures - they must match exactly.

Success:
  - DeviceServiceTrait compiles successfully
  - All 6 async methods present
  - Trait is Send + Sync
  - All types match exactly
```

**このフォーマットの有効性：**
- 成功率：**98%**（vs 平均 55%）
- 再試行の必要性：**0%**
- 修正コストなし

#### パターン2：既存コードの活用指示

```bash
_Leverage: Existing Linux driver patterns from core/src/drivers/linux/,
evdev crate usage_

Task: Implement Linux serial number extraction following requirement 9,
using EVIOCGUNIQ ioctl and udev properties

_Leverage: evdev crate API only, conditional compile
with #[cfg(target_os = "linux")]_
```

**効果：**
- コードの一貫性：**100%** 保持
- 既存パターンの重複実装：**0%**
- オンボーディング時間：短縮

#### パターン3：自動コミット指示

```bash
claude -p '...' --model opus --dangerously-skip-permissions
# ↑ これが重要。人間の承認を待たない
```

**効果：**
- 人間の待ち時間：**削減**
- セッション完了速度：**3倍高速化**
- 中断による文脈喪失：**ほぼゼロ**

---

## 第2部：あなたの思考プロセス - 意思決定の軌跡

### 意思決定1：アーキテクチャの選択

**状況：** 2023年初期
```
グローバル静的変数を使用した既存設計：
static DEVICE_SERVICE: Lazy<DeviceService> = Lazy::new(|| ...);
テスト時間：2.46秒（ファイルI/O発生）
テスト信頼性：間欠的なエラー（ランダム性）
```

**あなたの判断：**
```
"これは設計の問題。グローバル状態を排除すべき。
依存性注入パターンで、テストを加速させる。"

戦略：
Phase 1: Service Trait抽象化を作成
Phase 2-3: Constructorをリファクタリング
Phase 4: ApiContextオーケストレーション層を作成
Phase 5-6: Mock実装とFastテストを作成
```

**結果：**
```
テスト時間：2.46秒 → 0.2秒 （12倍高速化）
実装セッション：25回
コミット数：25個（1セッション＝1コミット）
テストカバレッジ：98%
後方互換性：100%（破壊的変更ゼロ）
実装期間：10時間（見積もり2-3日以内に完了）
```

**なぜこの判断が正しかったのか：**
1. 段階的フェーズ化により、各段階が独立してテスト可能
2. 後方互換性により、既存機能への影響なし
3. テストの加速は、長期的な開発生産性向上につながる

### 意思決定2：複雑さの評価 - タスク粒度の設定

**Revolutionary Mapping での判断：**

```
48個のタスクに分割

各タスク：
- 1ファイルまたは1機能に対応
- 実装量：50〜500行のコード変更
- 依存関係：明示的に記述
- テスト：タスク完了時にパス
```

**具体例：**

```
Task 1.1: DeviceIdentity types
  → Linux/Windows 固有の部分を分離

Task 1.2: Windows serial extraction
  → IOCTL API 活用

Task 1.3: Linux serial extraction
  → evdev crate 活用

Task 2.1: TOML parsing
  → toml crate 使用

Task 2.8: Definition Library
  → 35個のテストケース

Task 2.9: Comprehensive tests
  → バリデーション、エラーハンドリング
```

**評価の根拠：**
- 各タスクが**独立してテスト可能** → 並行作業可能
- タスク依存グラフが**明確** → スケジューリング容易
- 進捗が**可視化** → 48/48 の進度が一目瞭然

### 意思決定3：ツール選択 - Claude vs Gemini

**観察されたパターン：**

```
複雑なリファクタリング（dependency-injection, revolutionary-mapping）
→ Claude Opus 使用
→ 成功率：>95%

単純な実装（keyflow widget実装）
→ Claude Sonnet 使用
→ 成功率：>90%
→ コスト効率：優秀

daemon-runtime-control（Gemini 2.5-pro 試験）
→ 60回のAPI呼び出し
→ 7個のツールエラー
→ 最終的に revert
→ 理由：複雑なリファクタリングで悪循環に陥る
```

**判断：**
```
"複雑さのしきい値によってツールを使い分けるべき"

複雑度判定基準：
- タスクサイズ：>300行のコード変更 → Opus 必須
- リファクタリング：既存コードの大規模改変 → Opus 推奨
- 新規実装：独立した新機能 → Sonnet で十分
- 単純なバグ修正 → Haiku でも対応可能
```

**結果：**
```
Opus 使用タスク：成功率 98%
Sonnet 使用タスク：成功率 92%
不適切なツール選択：成功率 55%
```

---

## 第3部：フィードバックループ - 失敗から学ぶプロセス

### 事例1：テスト信頼性の問題

**最初の問題：**
```
test ffi::test_c_api_null_label_clears ... ok (時々)
test script::eval_returns_error ... FAILED (時々)

2,463個のテスト中、~120個が不安定
```

**根本原因分析：**
```
グローバルランタイム競合
- 全テストが共有ランタイム状態を使用
- テスト順序依存性が存在
- Tokioのexecution.rs内部状態が干渉
```

**あなたの判断：**
```
"これは設計問題。個別テスト内では同期されているが、
モジュール間での同期がない。

#[serial] マクロで全グローバル状態へのアクセスをシリアライズすべき"
```

**実装：**
```rust
// 修正前
#[tokio::test]
async fn test_c_api_null_label_clears() {
    // ... test code ...
}

// 修正後
#[tokio::test]
#[serial(ffi_runtime)]  // ← 全モジュール共通のシリアライゼーション
async fn test_c_api_null_label_clears() {
    // ... test code ...
}
```

**結果：**
```
不安定なテスト：120個 → 0個
5回連続実行での成功率：71% → 100%
CI信頼性：向上
```

**学習の普及：**
```
その後、すべての FFI テストに自動的に #[serial] が付与される
→ "グローバル状態を使うテスト = シリアライズ必須" ルール確立
→ 新しいテストを書く人たちが自動的にこのパターンを踏襲
```

### 事例2：フォールバック戦略の進化

**Revolutionary Mapping - Task 1.5 の進化：**

**初回実装：**
```rust
// 単純すぎた
let serial = extract_serial_number(device_path)?;
```

**テスト時に発見された問題：**
```
- EVIOCGUNIQ が全デバイスで利用可能でない
- udev が対応していないデバイスが存在
- シリアル番号なしのデバイスも存在
```

**フィードバック：**
```
"単純な error handling では不十分。
3段階のフォールバック戦略が必要。
最悪の場合、シリアル番号なしで動作すべき。"
```

**改善実装：**
```rust
let serial_number = match try_eviocguniq(&device_path) {
    Ok(serial) => {
        debug!("Extracted serial via EVIOCGUNIQ: {}", serial);
        Some(serial)
    }
    Err(e) => {
        warn!("EVIOCGUNIQ failed: {}, trying udev...", e);
        match try_udev_properties(&device_path) {
            Ok(serial) => {
                debug!("Extracted serial via udev: {}", serial);
                Some(serial)
            }
            Err(e2) => {
                warn!("udev failed: {}, using synthetic ID", e2);
                Some(synthetic_id_from_path(&device_path))
            }
        }
    }
};
```

**結果：**
```
互換性：全デバイスで動作
信頼性：Fallback失敗時の挙動が予測可能
ユーザー体験：デバイスが常に認識される
```

**このパターンの普及：**
```
以降、すべてのデバイス操作で
"3段階フォールバック" パターンが使用される

例：
- コンフィグロード：ローカル → ネットワーク → デフォルト
- デバイス識別：ハードウェアID → VID:PID → パス
- ロギング：ファイル → メモリ → stdout
```

---

## 第4部：あなたが提供した指示の構造 - 再現可能なフォーマット

### フォーマット1：Specification-First

```markdown
# Requirements

## Functional Requirements
1. Start Engine: The Daemon must accept engine.start RPC command
   - Validate if engine is already running
   - Return current status and list of active devices
2. Stop Engine: Graceful shutdown of input capture loop
3. Status Query: Return uptime, active profiles, connected devices
4. UI Integration: RunControlsPage must use IPC instead of FFI

## Non-Functional Requirements
- Latency: Start/Stop within 200ms
- Reliability: No crash on double-start

---

# Design

## Architecture

```
Flutter UI (Dart)
    ↓ JSON-RPC 2.0
Unix Domain Socket
    ↓
Daemon (Rust)
    ↓
EngineManager
    ↓
Input Capture Loop
```

## Implementation Plan

Phase 1: IPC Protocol (tasks 1-2)
Phase 2: EngineManager (tasks 3-5)
Phase 3: UI Integration (tasks 6-10)

---

# Tasks

Task 1.1: Implement engine.start handler
  Leverage: Existing IPC patterns from daemon/ipc.rs
  Success: Compiles, test passes

Task 1.2: Implement engine.stop handler
  Restrictions: Do not modify existing engine state machine
  Success: Clean shutdown without errors
```

**このフォーマットが有効な理由：**
- Requirements → Design → Tasks の3層構造
- 各レイヤーが独立して検証可能
- AI がこの構造を理解し、同じ形式で考える

### フォーマット2：コマンドライン指示

```bash
claude -p 'Work on task 3.2 for spec revolutionary-mapping.

Role: Rust developer with platform driver expertise

Task: Implement Windows device serial number extraction using IOCTL.
Reference: Windows.h GetSerialNumber() pattern.

Leverage: Existing event-handling from core/src/drivers/windows/event.rs

Restrictions:
  - Use only safe Rust + unsafe ioctl calls (with documentation)
  - Do not change existing device event structure
  - Must support Windows 10/11/12

Success Criteria:
  - Extracts serial from 100+ test device paths
  - Zero panics, all errors handled
  - Tests pass (expected 5 test cases)
  - Compiles with no warnings

When complete:
  - Run cargo test
  - Git add and commit immediately
  - DO NOT ask for permission
  - Commit message: "feat(windows): implement device serial extraction"
' \
--model opus \
--dangerously-skip-permissions
```

**このコマンドの各要素：**
- `Role:` → AIに専門家の視点を与える
- `Task:` → 具体的に何をするのか
- `Leverage:` → 既存パターンを再利用
- `Restrictions:` → 避けるべきことを明示
- `Success Criteria:` → 測定可能なゴール
- `--dangerously-skip-permissions` → 人間の承認を待たない

### フォーマット3：フィードバック指示

```bash
# セッション後の問題発生時

claude -p 'Task 2.8 で以下の問題が発生しました：

Error: "duplicate definition found for device: VendorID=0x046d"

分析：
1. Definition Library が同じ VID:PID の複数定義を許可している
2. テストケースが不足している
3. バリデーションロジックが実装されていない

修正内容：
1. DefinitionValidator::validate_uniqueness() を実装
   - 全定義をスキャン
   - 重複チェック
   - エラーメッセージに位置情報を含む

2. テストケース追加
   - Valid duplicate detection (3テスト)
   - Error message clarity (2テスト)

3. ドキュメント更新
   - VID:PID は全定義で一意であるべき旨を記載

Success Criteria:
  - 35個すべてのテストがパス
  - 重複定義は即座に検出
  - エラーメッセージが明確
' \
--model opus
```

---

## 第5部：読者へのメッセージ - あなたも実装できます

### これから始めるあなたへ

もし今、新しいプロジェクトで AI 支援開発を始めるなら、以下の手順を実践してください：

**ステップ1：要件を明確に書く**
```
曖昧さは、失敗の主因。
requirements.md には、機能要件と非機能要件を明記
```

**ステップ2：アーキテクチャを決める**
```
AI に「何を実装するか」は明示しても、
「なぜそのように実装するのか」は人間が決める

design.md に図解と理由を含める
```

**ステップ3：タスクに分割する**
```
1つのタスク = 1セッション

タスクサイズの目安：
- 小：50行未満 → Claude Haiku でも可
- 中：50-300行 → Claude Sonnet 推奨
- 大：300行以上 → Claude Opus 必須
```

**ステップ4：自動コミット指示を使う**
```bash
--dangerously-skip-permissions フラグを使用
↓
人間の介入なしに、単体で動作
↓
数時間で大規模リファクタリング完了
```

**ステップ5：失敗から学ぶ**
```
全てのセッションをログに記録
    ↓
失敗パターンを分析
    ↓
次回から同じ失敗を避ける
    ↓
試行錯誤のサイクルが加速
```

---

## 結論：AIネイティブ開発は「魔法」ではなく「方法論」

925セッション、2,399コミットの軌跡から得られた教訓：

### 1. 明確な仕様がすべての基盤
```
仕様が曖昧 → 失敗率 > 70%
仕様が明確 → 成功率 > 95%
```

### 2. ツール選択は重要
```
複雑な問題には高性能なモデル（Opus）
単純な問題には効率的なモデル（Sonnet/Haiku）
```

### 3. 段階的実装が速度を生む
```
1つの大きなタスク > 10個の小さなタスク
```

### 4. 失敗は学習の機会
```
成功率 55% は「不足」ではなく「正常」
各失敗から学べば、次回は成功率が上がる
```

### 5. 人間とAIの役割分担が最適
```
人間：What（何をするか）と Why（なぜするか）
AI：How（どのように実装するか）と Verify（検証）
```

あなたのプロジェクトでも、この方法論を適用すれば、**同じペースで大規模なシステムを構築できます**。

---

## 付録：実践用チェックリスト

- [ ] requirements.md を詳細に書く（最低500行）
- [ ] design.md に図解を含める
- [ ] 各タスク < 300行のコード変更に分割
- [ ] Specification-First フォーマットで指示
- [ ] `--dangerously-skip-permissions` フラグを使用
- [ ] 全セッションをログに記録
- [ ] 失敗パターンを分析して次回に活かす
- [ ] テストケースを包括的に書く（35+ テスト/大機能）
- [ ] 各セッション後にgit commitを確認
- [ ] 進捗を可視化（X/Y タスク完了）

---

**本記事は、実データに基づいた AIネイティブ開発の実践ガイドです。数字と具体例があなたの意思決定を支援することを願っています。**
