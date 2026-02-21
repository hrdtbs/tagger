# 製品仕様書：OmniTagger (仮称)

## 1. 概要
ローカルの画像ファイルやブラウザ上の画像を、右クリックメニューからAIプロンプト（タグ）に変換し、クリップボードへ転送するデスクトップアプリケーション。

## 2. 機能要件

### 2.1 ローカルファイル連携 (Windows / Linux)
 * **コンテキストメニュー**:
   * **Windows**: エクスプローラーで画像ファイル（.png, .jpg, .webp等）を右クリックし、「Get Tags」を選択することで発火。
   * **Linux**: デスクトップ環境（Nautilus, Dolphin等）で「Get Tags」アクションを選択。
 * **バックグラウンド処理**: アプリが起動していない場合でも自動的に起動し、タグ生成後にクリップボードへコピーして終了（または常駐）。
   * **Headless (CLI)**: コマンドライン引数付きで起動した場合は、処理完了後に自動終了。
     * `omni-tagger <file_path>`: 指定された画像ファイルを処理。
     * `omni-tagger --process-url <url>`: 指定されたURLの画像をダウンロードして処理（Native Messaging等で使用）。
     * `omni-tagger --delete-after <file_path>`: 指定された画像ファイルを処理後に削除（一時ファイル処理用）。
   * **GUI**: 通常起動（引数なし）の場合は常駐し、後続のリクエストを処理。

### 2.2 ブラウザ連携 (Chrome/Edge/Brave/Firefox予定)
 * **ブラウザ拡張機能**: 専用の拡張機能を導入し、Webページ上の画像を右クリックして「Get Tags」を選択。
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
| 推論エンジン | ONNX Runtime (ort) | CPU/GPUを活用したクロスプラットフォーム推論 |
| 画像処理 | image-rs | 画像の読み込みと前処理 |
| UIライブラリ | React + Tailwind CSS | 設定画面の構築 |
| 連携技術 | Native Messaging | ブラウザ拡張機能との通信 |
| OS統合 | Windows Registry / Linux .desktop | コンテキストメニューの登録 |

## 4. システムアーキテクチャ・データフロー
1. **Trigger**:
    *   **Local**: Context Menu (Registry/.desktop) -> 実行引数としてファイルパスを受け取る。CLI経由でのURL処理もサポート。
    *   **Browser**: Chrome Extension Context Menu -> Native Messaging経由でJSONメッセージを受信。
2. **Image Loading**: ファイルパスまたはURL/Base64から画像データをメモリに展開。
3. **Preprocessing**: 画像を 448 x 448 ピクセルにリサイズし、BGR正規化を実行 (WD14 SwinV2 標準)。
4. **Inference**: ONNXモデルに入力し、各タグのスコア（0.0 ~ 1.0）を算出。
5. **Post-processing**: 除外タグ（Sensitiveな内容など）をフィルタリングし、文字列に整形。
6. **Action**: クリップボードへ書き込み、通知を表示。

## 5. ユーザーインターフェース (UI) 要件

### 5.1 設定画面 (Settings Window)
 * **Model Selection**: 使用するONNXモデルの切り替え。
 * **Context Menu Integration**:
   * "Add to Windows/Linux Context Menu" ボタン（レジストリ/.desktop登録）。
   * "Install Browser Extension" 手順表示。
 * **Tag Formatting**:
   * アンダースコアの有無
   * 除外タグ設定

## 6. 非機能要件
 * **パフォーマンス**: トリガーからクリップボード完了まで 1秒以内 を目標とする。
 * **配布サイズ**: アプリ本体を 100MB以下 に抑制（モデルファイルを除く）。
 * **オフライン動作**: インターネット接続なしで全ての機能が動作すること。

## 7. 技術仕様 (Technical Details)

### 7.1 Native Messaging Protocol
ブラウザ拡張機能 (`browser-extension`) とネイティブホスト (`native_host.exe` / `native_host`) 間の通信プロトコル（JSON over Stdin/Stdout）。

### 7.2 Registry & Configuration Paths

**Windows Registry:**
*   Context Menu: `HKCU\Software\Classes\*\shell\OmniTagger`
*   Native Host (Chrome): `HKCU\Software\Google\Chrome\NativeMessagingHosts\com.omnitagger.host`
*   Native Host (Edge): `HKCU\Software\Microsoft\Edge\NativeMessagingHosts\com.omnitagger.host`
*   Native Host (Brave): `HKCU\Software\BraveSoftware\Brave-Browser\NativeMessagingHosts\com.omnitagger.host`

**Linux Configuration:**
*   Context Menu (.desktop): `~/.local/share/applications/omni-tagger-context.desktop`
*   Native Host Manifest (Chrome): `~/.config/google-chrome/NativeMessagingHosts/com.omnitagger.host.json`
*   Native Host Manifest (Chromium): `~/.config/chromium/NativeMessagingHosts/com.omnitagger.host.json`
*   Native Host Manifest (Edge): `~/.config/microsoft-edge/NativeMessagingHosts/com.omnitagger.host.json`
*   Native Host Manifest (Brave): `~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.omnitagger.host.json`

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
