# 製品仕様書：OmniTagger (仮称)

## 1. 概要
ローカルの画像ファイルやブラウザ上の画像を、右クリックメニューからAIプロンプト（タグ）に変換し、クリップボードへ転送するデスクトップアプリケーション。

## 2. 機能要件

### 2.1 ローカルファイル連携 (Windows / Linux / macOS)
 * **コンテキストメニュー**:
   * **Windows**: エクスプローラーで画像ファイル（.png, .jpg, .webp等）を右クリックし、「Get Tags」を選択することで発火。
   * **Linux**: デスクトップ環境（Nautilus, Dolphin等）で「Get Tags」アクションを選択。
   * **macOS**: Automator Quick Action (`.wflow`) によって実装。
 * **複数ファイル選択の制限**: 現在の実装では、複数ファイルを同時に選択してコンテキストメニューから実行した場合、OSから複数回の独立したプロセス起動（引数付き）として処理されます。それぞれが独立してクリップボードを上書きするため、最後に処理された画像の結果のみがクリップボードに残る仕様（または競合による上書き問題）となっています。
 * **バックグラウンド処理**: アプリが起動していない場合でも自動的に起動し、タグ生成後にクリップボードへコピーして終了（または常駐）。
   * **Headless (CLI)**: コマンドライン引数付きで起動した場合は、処理完了後に自動終了。
     * `omni-tagger <file_path>`: 指定された画像ファイルを処理。
     * `omni-tagger --process-url <url>`: 指定されたURLの画像をダウンロードして処理（Native Messaging等で使用）。
     * `omni-tagger --delete-after <file_path>`: 指定された画像ファイルを処理後に削除（一時ファイル処理用）。
   * **GUI**: 通常起動（引数なし）の場合は常駐し、後続のリクエストを処理。

### 2.2 ブラウザ連携 (Chrome/Edge/Brave/Firefox)
 * **ブラウザ拡張機能**: 専用の拡張機能を導入し、Webページ上の画像を右クリックして「Get Tags」を選択。
   * **Safari**: macOSのSafariは拡張機能のアーキテクチャが大きく異なる（Xcodeプロジェクトへの組み込みが必要）ため、現在はサポート対象外です。
 * **Native Messaging**: ブラウザからデスクトップアプリへ直接画像データ（またはURL）を送信し、ローカルで推論を実行。

### 2.3 AI解析（Tagger）機能
 * **ローカル推論**: プライバシーと速度のため、外部APIを使わずローカルのONNX Runtimeで実行。
 * **対応モデル**: WD14 Tagger (SwinV2 / ConvNext / ConvNextV2) を標準搭載。
 * **しきい値調整**: 抽出するタグの確信度（Probability）の下限を設定可能（例: P > 0.35）。

### 2.4 出力機能
 * **クリップボード保存**: 抽出したタグをカンマ区切りのテキストとして即座にコピー。
 * **通知**: タグ生成完了時にデスクトップ通知を表示。

## 3. 技術スタック

| 区分 | 選定技術 | 備考 |
|---|---|---|
| フレームワーク | Tauri (v2) | Rust製の軽量コア + Webフロントエンド |
| 言語 | Rust / TypeScript | 高速な画像処理と安全なメモリ管理 |
| 推論エンジン | ONNX Runtime (ort) | CPU/GPUを活用したクロスプラットフォーム推論（※100MBのサイズ制限を満たすため、標準ではCPU実行のみ。GPU Execution Providerは動的ダウンロード等での対応が必要） |
| 画像処理 | image-rs | 画像の読み込みと前処理 |
| UIライブラリ | React + Tailwind CSS | 設定画面の構築 |
| 連携技術 | Native Messaging | ブラウザ拡張機能との通信 |
| OS統合 | Windows Registry / Linux .desktop | コンテキストメニューの登録 |

## 4. システムアーキテクチャ・データフロー
1. **Trigger**:
    *   **Local**: Context Menu (Registry/.desktop) -> 実行引数としてファイルパスを受け取る。CLI経由でのURL処理もサポート。
    *   **Browser**: Browser Extension Context Menu -> Native Messaging経由でJSONメッセージを受信。
2. **Image Loading**: ファイルパスまたはURL/Base64から画像データをメモリに展開。
3. **Preprocessing**: 画像を 448 x 448 ピクセルにリサイズし、BGR変換を実行 (0-255範囲) (WD14 SwinV2 標準)。
4. **Inference**: ONNXモデルに入力し、各タグのスコア（0.0 ~ 1.0）を算出。
5. **Post-processing**: 除外タグ（Sensitiveな内容など）をフィルタリングし、文字列に整形。
6. **Action**: クリップボードへ書き込み、通知を表示。

## 5. ユーザーインターフェース (UI) 要件

### 5.1 設定画面 (Settings Window)
 * **Model Selection**: 使用するONNXモデルの切り替え。
 * **Context Menu Integration**:
   * "Add to Windows/Linux Context Menu" ボタン（レジストリ/.desktop登録）。
   * "Install Browser Extension" 手順表示。
 * **Target Browser**: Native Messagingの登録先ブラウザを選択。
 * **Tag Formatting**:
   * アンダースコアの有無
   * 除外タグ設定
 * **Advanced Model Settings**:
   * Input Size, Color Format, Normalize 等の推論前処理の設定。

## 6. 非機能要件
 * **パフォーマンス**: トリガーからクリップボード完了まで 1秒以内 を目標とする。
 * **配布サイズ**: アプリ本体を 100MB以下 に抑制（モデルファイルを除く）。
 * **オフライン動作**: インターネット接続なしで全ての機能が動作すること。

## 7. 技術仕様 (Technical Details)

### 7.1 Native Messaging Protocol
ブラウザ拡張機能 (`browser-extension`) とネイティブホスト (`native_host.exe` / `native_host`) 間の通信プロトコル（JSON over Stdin/Stdout）。
**注意**: 現在の実装では、Linux環境およびmacOS環境においてもネイティブホストのバイナリ名は `native_host.exe` となっています（ビルドプロセスの一貫性のため）。各OSのマニフェストファイルはこの名前を参照します。
また、FirefoxのNative Messaging Hostマニフェストには `allowed_extensions` フィールドが必須であり、特定の拡張機能IDを指定する必要があります（Chrome等の `allowed_origins` とは異なります）。開発中の拡張機能をFirefoxで利用する場合、`manifest.json`に`browser_specific_settings.gecko.id`が指定されています。

### 7.2 Registry & Configuration Paths

**Windows Registry:**
*   Context Menu: `HKCU\Software\Classes\*\shell\OmniTagger`
*   Native Host (Chrome): `HKCU\Software\Google\Chrome\NativeMessagingHosts\com.omnitagger.host`
*   Native Host (Edge): `HKCU\Software\Microsoft\Edge\NativeMessagingHosts\com.omnitagger.host`
*   Native Host (Brave): `HKCU\Software\BraveSoftware\Brave-Browser\NativeMessagingHosts\com.omnitagger.host`
*   Native Host (Firefox): `HKCU\Software\Mozilla\NativeMessagingHosts\com.omnitagger.host`

**Linux Configuration:**
*   Context Menu (.desktop): `~/.local/share/applications/omni-tagger-context.desktop`
*   Native Host Manifest (Chrome): `~/.config/google-chrome/NativeMessagingHosts/com.omnitagger.host.json`
*   Native Host Manifest (Chromium): `~/.config/chromium/NativeMessagingHosts/com.omnitagger.host.json`
*   Native Host Manifest (Edge): `~/.config/microsoft-edge/NativeMessagingHosts/com.omnitagger.host.json`
*   Native Host Manifest (Brave): `~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.omnitagger.host.json`
*   Native Host Manifest (Firefox): `~/.mozilla/native-messaging-hosts/com.omnitagger.host.json`

**macOS Configuration:**
*   Context Menu: `~/Library/Services/OmniTagger.workflow`
*   Native Host Manifest (Chrome): `~/Library/Application Support/Google/Chrome/NativeMessagingHosts/com.omnitagger.host.json`
*   Native Host Manifest (Chromium): `~/Library/Application Support/Chromium/NativeMessagingHosts/com.omnitagger.host.json`
*   Native Host Manifest (Edge): `~/Library/Application Support/Microsoft Edge/NativeMessagingHosts/com.omnitagger.host.json`
*   Native Host Manifest (Brave): `~/Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.omnitagger.host.json`
*   Native Host Manifest (Firefox): `~/Library/Application Support/Mozilla/NativeMessagingHosts/com.omnitagger.host.json`

**注意**: Linux環境におけるSnapやFlatpakでインストールされたサンドボックス化されたブラウザ（UbuntuのデフォルトFirefoxなど）では、上記の標準的な設定パス（`~/.mozilla/...`など）にあるNative Messaging Hostマニフェストを読み込むことができず、連携が根本的に破綻します。これに対応するためには、SnapやFlatpak固有のディレクトリ（例: `~/snap/firefox/current/.mozilla/native-messaging-hosts/` や Flatpakのパーミッション設定）へのマニフェスト配置を考慮する必要があります。さらに、マニフェストが読み込めたとしても、サンドボックスの隔離仕様により任意のホストバイナリ（`native_host.exe`）の直接実行はブロックされます。Flatpak環境ではラッパースクリプト（`flatpak-spawn --host`）を介した実行、Snap環境では特定のplugインターフェースを通じた実行などの根本的な回避策が追加で必要となります。

**注意**: `url` フィールドで画像URLを送信する場合、バックエンド側で再ダウンロードを行うため、Cookie認証が必要な画像や `blob:` URL は処理できません。その場合は `data` フィールド（Base64）を使用してください。

**Request (Extension -> Host):**
```json
{
  "url": "https://example.com/image.jpg",
  "data": null
}
```
*または*
```json
{
  "url": null,
  "data": "data:image/png;base64,..."
}
```

**Response (Host -> Extension):**
```json
{
  "status": "ok",
  "message": "Processing started"
}
```
*または*
```json
{
  "status": "error",
  "message": "Error description..."
}
```

### 7.3 Concurrency & Lifecycle
**同時実行による競合 (Race Conditions):**
1. **クリップボード競合**: コンテキストメニューから複数ファイルを同時に実行すると、TauriのSingle Instance Pluginによってメインプロセスに引数が転送されますが、それぞれが非同期に処理され、完了順にクリップボードを上書きします。結果としてユーザーは1つの結果しか得られません。
2. **モデルダウンロード競合**: 初回起動時に複数ファイルが同時にリクエストされた場合（またはGUIによる事前ロード中にCLIリクエストが来た場合）、`check_and_download_models` が複数スレッド/タスクから同時に実行され、同じファイルパス (`model.onnx`) に対して同時に書き込みを行おうとしてファイルが破損する、あるいはエラーでパニックする可能性があります。

**アンインストールとクリーンアップ:**
本アプリはコンテキストメニューのレジストリ/Desktop Entryや、Native MessagingのManifestファイルをOS上に配置します。現状、これらを一括で削除する正式なアンインストーラーやクリーンアップ機能が存在しないため、アプリ本体を削除する前に、設定画面から手動で「Unregister」を実行する必要があります。

### 7.4 Security Considerations
1. **SSRF (Server-Side Request Forgery)**: `--process-url` および Native Messaging経由でのURL処理において、受け取ったURLの検証（ローカルIPのブロック、スキームの制限等）が行われていません。悪意のある拡張機能やコマンド呼び出しによって、ローカルネットワーク内のリソースに対する意図しないアクセス(SSRF)を引き起こすリスクがあります。
2. **OOM (Out Of Memory) / Payload Limits**: Native MessagingでBase64データを受け取る際、現状メッセージサイズや画像サイズの厳格な上限チェックが存在しないため、巨大な画像データを送りつけられることでメモリ枯渇を引き起こす可能性があります。

### 7.5 Linux Headless Execution
**重要**: OmniTaggerはTauri v2 (GTK) に依存しているため、CLIモード（`--process-url` や ファイルパス引数）での実行であっても、ディスプレイサーバーへの接続が必要です。
ディスプレイのない環境（サーバーやCI環境など）で実行する場合は、`xvfb` (X virtual framebuffer) を使用してください。

例:
```bash
xvfb-run -a ./omni-tagger <image_path>
```

**警告**: `xvfb-run` を使用して実行した場合、Tauri/GTKがアクセスするクリップボードはXvfbによって作成された「隔離された仮想クリップボード」となります。そのため、処理完了後にホストOSのクリップボード（WaylandやX11）に生成されたタグが反映されません。ヘッドレス環境で出力を取得するには、クリップボード同期ツールを併用するか、標準出力 (`stdout`) を利用する仕組み（現在未実装）が必要になります。

## 8. Architectural Limitations & Known Issues
現在のアーキテクチャおよび実装には、以下の既知の制限と課題が存在します。

（現在、重大なアーキテクチャ上の制限は解決されています。過去に存在した SSRF 脆弱性、ペイロードサイズ制限の欠如、バッチ処理時のクリップボード競合、モデルダウンロード時の競合によるファイル破損などは、バリデーションの強化とキューイング機構の導入により修正済みです。）
