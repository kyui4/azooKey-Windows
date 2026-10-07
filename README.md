# azooKey for Windows

[AzooKeyKanaKanjiConverter](https://github.com/azooKey/AzooKeyKanaKanjiConverter)を利用したWindows版IMEです。

> [!WARNING]
> 現在開発中であるため、安定性や機能に関しては保証できません。使用する際は自己責任でお願いします。

# インストール方法
[Release](https://github.com/fkunn1326/azooKey-Windows/releases)から`azookey-setup.exe`をダウンロードし、インストーラーを実行してください。

# 機能

- [x] ライブ変換
- [x] Zenzaiを使用したニューラルかな漢字変換

- [ ] 学習機能
- [x] 辞書登録機能（設定画面のユーザー辞書）
- [x] 無変換キーで英数、変換キーで日本語への切り替え
- [ ] テーマ変更機能
- [ ] 辞書のインポート/エクスポート機能
- [ ] いい感じ変換
- [ ] 個人最適化システム
- [ ] 予測変換

# 設定

## ユーザー辞書

設定画面の「ユーザー辞書」で読みと単語を入力して登録できます。読みはひらがな・全角カタカナ（64文字以内）、単語は128文字以内です。カタカナの読みはひらがなに統一して保存します。同じ読みで複数の単語を登録できます。同じ読みと単語の重複登録はできません。

登録した単語は名詞として変換エンジンの動的辞書に反映します。登録・削除後の次の変換から反映されます。エンジンに接続できない場合も保存し、画面に再起動が必要と表示します。保存先は `%APPDATA%/Azookey/settings.json` の `user_dictionary` です。現在のエンジンは動的辞書を線形検索するため、上限は1000件です。

## キー操作

修飾キーなしの無変換キーで英数、変換キーで日本語に切り替えます。同じモードのキーは入力途中でも何もしません。日本語の入力途中に無変換を押すと、現在の候補と残りの読みを確定してから英数に切り替えます。Ctrl・Alt・Shiftとの組み合わせはアプリ側に渡します。半角／全角キーも引き続き利用できます。

## Zenzai

### 変換プロファイル
設定で変換プロファイルを指定すると、プロファイルに応じた変換候補が表示されます。

### バックエンド
以下の3種類のバックエンドをサポートしています。

- **CPU**: 動作が非常に遅いため、非推奨です。
- **CUDA**: NvidiaのGPU専用。[CUDA Toolkit 12系](https://developer.nvidia.com/cuda-downloads)をインストールする必要があります。
- **Vulkan**: GPUのドライバーに標準で含まれているため、追加のインストールは不要です。

### GPUへの割り当てと計測

CUDA／Vulkan選択時は、設定の `gpu_layers` に応じてモデルの層をGPUへ載せます。既定値99は全層への割り当てを要求します（モデルの実際の層数を超えても問題ありません）。GPUメモリ不足や内蔵GPUで遅くなる場合は、設定画面で8／16／24／32層または0層に減らせます。CPU選択時は常に0層です。バックエンド・層数の変更にはエンジンの再起動が必要です。

ビルド時に固定版の変換エンジンへ `scripts/prepare_converter.ps1` でGPU割り当てを適用します。依存バージョンが違う場合は処理を止めます。起動時に選択バックエンドのDLLを読み込み、利用できなければCPUへフォールバックした旨をログに表示します。ログの層数は要求値であり、実際にGPUへ載った層数はllamaのログも確認してください。

変換時間を測る場合は、PowerShellで `$env:AZOOKEY_TIMING = '1'` を設定してビルドした `launcher.exe` を起動します。変換ごとの時間が `conversion_ms=...` と出力されます（入力内容は記録しません）。同じモデル・辞書・文章でCPUとGPUを比較し、初回の読み込みを除いて20回以上の中央値と遅い側の値を比較してください。この計測は候補生成と結果の受け渡しを含み、画面表示やキー入力全体の時間は含みません。実機での高速化率は未測定です。

# コミュニティ

## 開発を支援する
- [GitHub Sponsors (Miwa)](https://github.com/sponsors/ensan-hcl): 変換エンジンの開発者
- [Patreon (fkunn1326)](https://www.patreon.com/c/fkunn1326): Windowsに移植した人

## 開発に参加する

### 開発環境のセットアップ

- [Rust](https://www.rust-lang.org/tools/install)
- [Swift for Windows](https://www.swift.org/install/windows/) (Swift 6.0以上)
- [protoc](https://protobuf.dev/installation/) 
- [node.js](https://nodejs.org/en/download/)
- [inno setup](https://jrsoftware.org/isinfo.php)

### ビルド

#### リポジトリのクローン
```
git clone https://github.com/fkunn1326/azookey-Windows --recursive
```
`--recursive`オプションを付けて、サブモジュールも一緒にクローンしてください。

#### cargo-makeのインストール
```
cargo install --force cargo-make
```

#### ビルド
```
cargo make build [--debug/--release]
```
`--debug`オプションを付けるとデバッグビルド、`--release`オプションを付けるとリリースビルドになります。必ずどちらかを指定してください。

`build`フォルダーが作成され、ビルドされた実行ファイルが格納されます。

`launcher.exe`を管理者権限で実行すると、azookeyの変換エンジンが起動します。

また、IMEを登録する際は以下のように`regsvr32.exe`を使用して登録する必要があります。
```c
regsvr32.exe "path/to/build/azookey_windows.dll" /s
regsvr32.exe "path/to/build/x86/azookey_windows.dll" /s
```
逆に登録を解除する場合は`/u`オプションを付けて実行してください。

#### 開発時のヒント
- 開発は仮想マシンまたは専用のPCで行うことを推奨します。IMEがクラッシュするとWindowsがフリーズする可能性があります。
- IMEを解除する際、IMEを使用中のアプリケーション（メモ帳など）を終了しないと、解除できないことがあります。

# 関連

- [azooKey/azooKey](https://github.com/azooKey/azooKey): iOS / iPadOS向けの日本語キーボードアプリ
- [7ka-Hiira/fcitx5-hazkey](https://github.com/7ka-Hiira/fcitx5-hazkey): fcitx5向けのLinux版azooKey
- [azooKey/AzookeyKanakanjiConverter](https://github.com/azooKey/AzooKeyKanaKanjiConverter): azooKeyの変換エンジン

# 参考
本プロジェクトの開発にあたり、以下のリソースを参考にしました。ありがとうございます！
- [OMAMA-Taioan/khiin-rs](https://github.com/OMAMA-Taioan/khiin-rs/tree/master/windows)
- [google/mozc](https://github.com/google/mozc/tree/master/src/win32/tip)
- [microsoft/Windows-classic-samples](https://github.com/microsoft/Windows-classic-samples/tree/main/Samples/Win7Samples/winui/input/tsf/textservice)
- [dec32/ajemi](https://github.com/dec32/ajemi)
- https://zenn.dev/mkpoli/scraps/6dc57fcd0335cf
