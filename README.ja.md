# KeyRx2 クイックスタート (日本語・Linux デスクトップ)

KeyRx2 はキーボードのキー割り当てを変更するデーモンです。ブラウザの画面
(Web UI) で「プロファイル」を作って有効化するだけで、CapsLock と Ctrl の入れ替えなどができます。
英語の詳しい説明は [README.md](README.md) と
[Linux セットアップガイド](docs/user-guide/linux-setup.md) を見てください。

## 1. インストール

```bash
make build-release        # Web UI を埋め込んでビルド
./scripts/install.sh      # 実行ファイル・systemd サービス・udev ルールを入れる (sudo を聞かれます)
```

- インストール後に **一度ログアウトして、ログインし直してください**
  (`input` / `uinput` グループが有効になります)。
- うまく動かないときは `keyrx_daemon doctor` を実行します。足りない設定とその直し方が表示されます。

## 2. サービスを起動する

```bash
systemctl --user enable --now keyrx     # 今すぐ起動し、ログインのたびに自動起動
systemctl --user status keyrx           # 動いているか確認
```

## 3. Web UI を開く

ブラウザで <http://127.0.0.1:9867> を開きます (このパソコンからだけ見られます)。

```bash
xdg-open http://127.0.0.1:9867
```

## 4. CapsLock と Ctrl を入れ替える

Web UI でプロファイルを作り (テンプレートは「blank」でかまいません)、
次の 2 つの割り当てを追加して **有効化 (Activate)** します。

| 押すキー | 出力するキー |
|---|---|
| CapsLock | LCtrl |
| LCtrl | CapsLock |

有効化すると再起動なしでその場で反映されます。うまくいかないとき、
画面にエラーの行番号が出て、**直前の設定のまま動き続けます**。

コマンドで同じことをするなら:

```bash
keyrx_daemon profiles create my --template blank
keyrx_daemon config set-key CapsLock VK_LCtrl --profile my
keyrx_daemon config set-key LCtrl VK_CapsLock --profile my
keyrx_daemon profiles activate my
```

キー名は `CapsLock` のように **プレフィックスなし**で書きます (`VK_CapsLock` と書いても同じキーとして扱われます)。
出力側は `VK_LCtrl` のように `VK_` を付けます。

> 注意: 初期テンプレートは `device_start("*")` で **すべてのキーボード**に効きます。
> 特定のキーボードだけにしたいときは、`keyrx_daemon list-devices` で名前を調べて
> `device_start("*Logitech*")` のように絞り込んでください。

## 5. IME (日本語入力) のキーについて

JIS キーボードの日本語入力用キーは、次の名前で扱えます:

| キー | 名前 |
|---|---|
| 半角/全角 | `Zenkaku` |
| 無変換 | `Muhenkan` |
| 変換 | `Henkan` |
| カタカナ/ひらがな | `KatakanaHiragana` |

- KeyRx2 は「どのキーを出すか」を変えるだけです。IME のオン/オフを実際に切り替えるのは
  fcitx5 / ibus / Mozc 側のキー設定です。たとえば「無変換でオフ、変換でオン」にしたいなら、
  IME 側でそのキーを割り当ててください。
- これらのキーを割り当て変更する前に、IME 側の設定と重ならないか確認してください。
- 日本語入力の状態に応じて動きを変える条件 (`IME`) の書き方は
  [DSL マニュアル](docs/user-guide/dsl-manual.md) にあります。

## 6. キーボードが効かなくなったら (緊急停止)

設定を間違えて入力できなくなっても、次のどちらかで **すべてのキーボードを元に戻して KeyRx2 を止められます**
(設定が壊れていても、物理キーを直接見ているので効きます):

- **両手:** 左 Ctrl + 右 Ctrl + Escape を同時に押し続ける。
- **片手:** **Escape だけを 3 秒間押し続ける** (途中で他のキーを押すと取り消されます。
  Escape を連打しただけでは止まりません)。

別の端末や SSH からなら `systemctl --user stop keyrx` でも止められます。
このキーの組み合わせと秒数は変更できます
([緊急停止の設定](docs/user-guide/linux-setup.md#emergency-escape-keyboard-unusable-from-a-bad-config))。
片手での操作やスティッキーキーなどは [アクセシビリティ](docs/user-guide/linux-setup.md#accessibility) を見てください。
