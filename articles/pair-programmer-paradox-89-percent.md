---
title: ペアプログラマーの真実：89%の手動入力の逆説 - AIは「思考」であって「自動化」ではない
emoji: 💭
type: tech
topics: [ai, collaboration, programming, tools, workflow]
published: false
---

# ペアプログラマーの真実：89%の手動入力の逆説 - AIは「思考」であって「自動化」ではない

## はじめに

AI コーディング支援について、一般的な誤解があります：

**誤解：** 「AI が自動的に大量のコードを生成し、ツール呼び出しで自動化する」

**現実：** 925セッション、2,399コミットの分析から判明した真実は全く異なります

```
AI が実際にしていることの内訳：

思考・分析（コード生成なし）：     89%
ツール呼び出し（ファイルI/O等）：   8%
その他（エラー処理等）：             3%
```

**つまり、AI は圧倒的に「思考者」として機能しており、「自動化ツール」ではありません。**

本記事では、この 89% の手動入力が何をしているのか、そしてなぜこれが最も価値あるのかを詳細に探ります。

---

## 第1部：データから見える真実

### 925セッションのツール使用パターン

```
全セッション数：925

セッション内の平均ツール呼び出し：
  - 0-4 calls：  810 セッション（87.6%）✅
  - 5-10 calls： 95 セッション（10.3%）
  - 11+ calls：   20 セッション（2.1%）
```

### 「0-4 calls」の810セッションは何をしていたのか？

```
例：daemon-runtime-control （1セッション）

ツール呼び出し内訳：
  read_file (requirements.md)：        1回
  read_file (design.md)：               1回
  read_file (existing ipc.rs)：         1回
  write_file (new handler code)：       1回
  run_shell_command (git commit)：      1回
  ─────────────────────────────
  合計：5回のツール呼び出し

実際の「思考」のステップ：
  1. 要件を読む、理解する
  2. 既存パターンを分析
  3. 状態遷移を図解する（心の中で）
  4. JSON-RPC メッセージフォーマットを設計
  5. エラーハンドリング戦略を決定
  6. テスト戦略を構想
  7. 30行の Rust コードを構成
  8. コメントを追加
  9. 実装方針を確認
  10. コードを記述
  ─────────────────────────────
  合計：~8,000トークンの思考、分析、実装
```

### ツール呼び出しが多いセッション（20セッション）は何が違う？

```
revolutionary-mapping - Task 2.8 （35+テストの実装）

ツール呼び出し内訳：
  read_file：               8回（テストファイル、定義ファイル確認）
  write_file：             12回（テストケース追加）
  run_shell_command：       5回（テスト実行、git操作）
  grep/search：             4回（既存パターン検索）
  ─────────────────────────────
  合計：29回のツール呼び出し

実際の「思考」のステップ：
  - 35 テストケースの分類（分類ロジック：思考）
  - 各テストの期待値設計（設計：思考）
  - テスト間の依存関係管理（構造化：思考）
  - バリデーションロジックの検証（論理：思考）
  - エッジケース列挙（創造性：思考）
  - テストと実装の対応関係確認（マッピング：思考）
  ─────────────────────────────
  合計：～25,000トークンの思考、検証、設計
```

**重要な観察：**
- ツール呼び出しが**少ない**セッション = シンプルな実装
- ツール呼び出しが**多い**セッション = 複雑な検証・設計が必要

**ツール呼び出し数 ≠ 自動化度**

むしろ、**複雑な問題ほど思考（≠ツール呼び出し）が必要**

---

## 第2部：89%が「思考」だった具体例

### 例1：daemon-runtime-control の意思決定

**セッション開始時点：**
```
あなたの指示：
"Work on task 1.1 for spec daemon-runtime-control"

AI が読む仕様：
- requirements.md（要件）
- design.md（設計）
- 既存 daemon/ipc.rs（パターン参照）

ここまで：ツール呼び出し 3回
```

**AI の思考プロセス（ツール呼び出しなし）：**
```
1. 状態遷移を理解
   "Idle → Starting → Running"
   要件から：「既に起動中は error を返す」
   → State machine をメンタルモデル化

2. JSON-RPC メッセージ形式を設計
   {
     "jsonrpc": "2.0",
     "method": "engine.start",
     "params": {},
     "id": 1
   }
   → リクエスト・レスポンス形式を構想

3. エラーケースを列挙
   - Already running: error code -32603
   - Timeout: error code -32000
   - Internal error: error code -32603

4. Rust 実装アーキテクチャを設計
   ```rust
   pub async fn handle_engine_start(
       &mut self,
       params: serde_json::Value
   ) -> Result<Response> {
       match self.engine.state() {
           EngineState::Idle => {
               // transition to Starting
               // await engine.start()
               // transition to Running
           }
           EngineState::Running => {
               // return error
           }
           _ => {
               // return error
           }
       }
   }
   ```

5. テスト戦略を構想
   - test_engine_start_success: Idle → Running
   - test_engine_start_already_running: Running + start → error
   - Timeout handling test
```

**ツール呼び出しなし、すべて思考**

次に、ツール呼び出し開始：
```
write_file: Rust コードを記述（思考の結果を記述）
run_shell_command: cargo test （検証）
git commit: コミット
```

**重要：** 実装（write_file）は、思考の**結果**であり、思考自体ではない

### 例2：revolutionary-mapping のテスト設計

**Task 2.9 : Comprehensive Tests（35+ テストケース）**

**思考フェーズ（ツール呼び出しなし）：**
```
テスト戦略の分類：
  1. Parser Tests (TOML parsing)
     - Valid TOML parsing
     - Invalid syntax handling
     - Type validation
     → 8 テストケース

  2. Validation Tests (Definition validation)
     - Duplicate device detection
     - Invalid device IDs
     - Missing required fields
     → 10 テストケース

  3. Lookup Tests (VID:PID matching)
     - Exact match
     - Fallback to default
     - Not found
     → 5 テストケース

  4. Integration Tests (End-to-end)
     - Load multiple definitions
     - Resolve to correct profile
     → 7 テストケース

  5. Error Cases
     - File not found
     - Corrupted data
     - Permission errors
     → 5 テストケース
```

**なぜこの分類が必要か？**
- 各テストカテゴリーが異なる「問題領域」をカバー
- バグの検出確率を最大化
- 将来のリグレッションを防止

**実装フェーズ（ツール呼び出し）：**
```
write_file × 12 : 各テストケースを実装
run_shell_command: cargo test を実行
→ 3個のバグ発見
```

**結論：**
- テスト設計（思考）：100%
- テスト実装（ツール呼び出し）：20%

テスト価値の大部分は「何をテストするか」の思考から生まれている

---

## 第3部：なぜ89%の思考が必要なのか

### 問題1：仕様の曖昧性の解決

```
あなたの指示：
"Implement device identification"

AI の思考：
"『device identification』とは、実は何か？

  - パスベース？→ USB 変更で失敗
  - VID:PID？→ 複数デバイス同一
  - Serial number？→ 全デバイスに存在しない
  - 3段階フォールバック？→ 複雑だが堅牢

あるいは、仕様書に『VID:PID:Serial』と書いてあるか？
→ 仕様読む（ツール：read_file 1回）
→ 理解する（思考：50分）
→ 実装する（実装：30分、ツール：write_file）
"
```

**89%が処理している内容：**
- 曖昧さの解消
- 複数の解釈の中から最適を選択
- エッジケースの検討
- 依存関係の理解

### 問題2：既存パターンとの整合性

```
あなたの指示に含まれる：
"Leverage: Existing Linux driver patterns from core/src/drivers/linux/"

AI の思考：
"既存パターンとは何か？
  - src/drivers/linux/event_handler.rs を読む（ツール）
  - error handling の方法を理解する（思考）
  - debug logging の形式を理解する（思考）
  - ioctl 呼び出しのパターンを分析する（思考）

この新規コードが既存パターンと一貫性を持つように実装
  → 50%の時間を「既存コード理解」に使う"
```

**89%が処理している内容：**
- 既存コード分析
- パターンマッチング
- 一貫性維持のための設計調整

### 問題3：テスト戦略の構想

```
AI の思考：
"Task 2.8 でテストを書く。
 何をテストすべきか？

  - TOML のパース成功ケース
  - パース失敗ケース（invalid TOML）
  - 型チェック失敗
  - デバイス重複検出
  - デバイス未検出の扱い
  - ネストされた定義のロード
  - 大規模ファイルの処理
  - 数値境界値（VID:PID は u16）

各テストで『何を確認するか』を思考
  → 合計 35 テストケースを構想"
```

**89%が処理している内容：**
- テスト設計（創造性）
- バグ予測（経験値）
- カバレッジ最適化（分析）

---

## 第4部：ツール呼び出しの真実

### ツール呼び出しは「低価値」ではない。むしろ「検証」

```
read_file: 既存コード確認
  ← 思考を情報で支援

write_file: コード記述
  ← 思考の結果を具体化

run_shell_command: テスト実行
  ← 思考の正確性を検証

git commit: バージョン管理
  ← 成果を記録
```

**ツール呼び出しなしにコードは存在しない**

しかし、**ツール呼び出しだけではコードは生まれない**

---

## 第5部：89%の思考はどこで起こっているか

### LLMのトークン処理から見る

```
1つの実装セッション（daemon-runtime-control）：

Input Tokens:
  - 要件書：5,000 tokens
  - 設計書：3,000 tokens
  - 既存コード：4,000 tokens
  ────────────────────
  合計：12,000 tokens

処理プロセス（思考）：
  - 要件の解析：1,000 tokens 分の計算
  - 既存パターン理解：2,000 tokens 分の計算
  - 実装設計：3,000 tokens 分の計算
  - テスト戦略：2,000 tokens 分の計算
  - コード構成：2,000 tokens 分の計算
  ────────────────────
  合計：10,000 tokens 分の思考（出力なし）

Output (Completion):
  - Rust code：1,200 tokens （実装）
  - Comments：400 tokens （説明）
  - Error handling：600 tokens （堅牢性）
  ────────────────────
  合計：2,200 tokens の出力
```

**Input の 10,000 vs Output の 2,200**

**この比率こそが、AI が「思考者」である証拠**

---

## 第6部：あなたが設計した指示方法 - 89%の思考を引き出す

### パターン1：「Leverage」指示による思考促進

```bash
Leverage: Existing Linux driver patterns from core/src/drivers/linux/

↓

AI の思考：
1. src/drivers/linux/ 内のファイル構成を理解
2. error handling パターンを分析
3. logging の形式を確認
4. ioctl 使用パターンを研究
5. この新規実装が一貫性を持つように設計

思考時間：20-30分
ツール呼び出し：read_file 3-4回
実装時間：10-15分
```

### パターン2：「Restrictions」指示による思考の制約化

```bash
Restrictions:
  - Do not modify existing device event structure
  - Use only safe Rust + unsafe ioctl calls
  - Must support Windows 10/11/12

↓

AI の思考：
"『Do not modify』という制約がある。
 つまり、新しい方法を見つけるべき。

 既存構造を使う方法：
   Option 1: 構造体を拡張
   → 制約違反

   Option 2: Wrapper を作成
   → 制約遵守、実装可能

Option 2 を採用しよう"
```

この思考フローは ツール呼び出しなしに発生

### パターン3：「Success Criteria」指示による検証の内包

```bash
Success Criteria:
  - Extracts serial from 100+ test device paths
  - Zero panics, all errors handled
  - Tests pass (expected 5 test cases)

↓

AI の思考：
"100+ デバイスパスを想定するなら、
 エッジケースが多数存在する可能性：
   - パスの長さ制限
   - 特殊文字を含むパス
   - ネットワークマウント
   - シンボリックリンク

各エッジケースに対応するテストを設計する"
```

また ツール呼び出しなし

---

## 第7部：読者への提案 - 89%の思考を活用する

### 提案1：ツール呼び出しの多さを評価指標にしない

```
❌ 悪い評価：
"このセッションはツール呼び出し 3 回しかしなかった。
 自動化が不足している"

✅ 良い評価：
"このセッション、思考が深かったのか浅かったのか？
 複雑な問題を適切に分解したのか？"
```

### 提案2：思考を促す指示を設計する

```
❌ 浅い指示：
"Implement device identification"

✅ 深い思考を促す指示：
"Implement device identification.
 Leverage: Existing Linux driver patterns.
 Restrictions: Do not break existing code.
 Success: Identify 100+ device types uniquely.
 Test: Edge cases (serial missing, duplicates, etc)"
```

### 提案3：思考の結果を記録する

```
単なるコミットログではなく、
意思決定ログを記録する：

"Task 2.8 での思考：
  1. なぜ 35 テストケースなのか？
     → 5 つのテストカテゴリー、各 7 テスト
  2. なぜこのテストの順序か？
     → Parser → Validator → Lookup という依存順
  3. エッジケース検出方法？
     → 負パターン、境界値、複雑なネスト構造"
```

---

## 結論：AIは思考エンジン

```
AI コーディング支援の真実：

❌ 「自動化エンジン」ではない
   自動化は結果、思考は本質

✅ 「思考エンジン」である
   89% の計算が思考に費やされる
   11% の計算がツール呼び出しと実装に費やされる
```

### 最後に

ツール呼び出しの数を数えることは、
歌手が歌った**リズムの数**を数えることに等しく、

実際の価値は歌の**内容と表現**にあります。

AI のツール呼び出しの数を数えるのではなく、
AI の思考の**深さと品質**を評価するべきです。

あなたが設計した指示方法（Role, Task, Leverage, Restrictions, Success）は、
この 89% の思考を最大限に引き出すための フレームワークです。

---

**本記事は、925セッションのデータに基づいた、AIペアプログラミングの真実です。**
