# Pingora / nginx 静的ファイル配信の比較

## 準備（Ubuntu / Debian）

```bash
sudo apt-get update
sudo apt-get install -y git curl build-essential pkg-config libssl-dev cmake clang libclang-dev nginx apache2-utils

git clone <リポジトリのURL> pingora-static
cd pingora-static
```

確認：rustup導入済み。未導入の場合は https://rustup.rs/ を参照。

```bash
cargo build --release --locked

mkdir -p www logs
dd if=/dev/urandom of=www/1k.bin bs=1024 count=1 status=none
dd if=/dev/urandom of=www/64k.bin bs=1024 count=64 status=none
dd if=/dev/urandom of=www/1m.bin bs=1048576 count=1 status=none

wc -c www/*.bin
nginx -v
ab -V
```

確認：ビルド成功。ファイルサイズはそれぞれ1,024 / 65,536 / 1,048,576 bytes。

## 起動

各ターミナルの作業ディレクトリ：cloneした`pingora-static/`。

ターミナル1：Pingora

```bash
./target/release/pingora-static-bench
```

8080番が使用中の場合：

```bash
LISTEN_ADDR=0.0.0.0:18080 ./target/release/pingora-static-bench
```

確認：ポート変更時は以降のcurl・abのURLも変更。

ターミナル2：nginx

```bash
nginx -p "$PWD/" -c nginx/nginx.conf -e stderr
```

ターミナル3：応答確認

```bash
curl -I http://127.0.0.1:8080/64k.bin
curl -I http://127.0.0.1:8081/64k.bin
```

確認：両方とも`200 OK`、`Content-Length: 65536`。

## 計測

ウォームアップ：

```bash
ab -k -n 1000 -c 32 http://127.0.0.1:8080/64k.bin
ab -k -n 1000 -c 32 http://127.0.0.1:8081/64k.bin
```

本計測：同時に実行せず、1つずつ実行。

```bash
# Pingora
ab -k -n 100000 -c 32 http://127.0.0.1:8080/64k.bin

# nginx
ab -k -n 100000 -c 32 http://127.0.0.1:8081/64k.bin
```

確認：

- `-k`：KeepAlive有効
- `-n 100000`：合計10万リクエスト
- `-c 32`：同時接続32
- `Failed requests`：0
- `Non-2xx responses`：表示なし
- `Requests per second`：RPSを比較
- `Transfer rate`：ヘッダー込みの転送量。表示値を1024で割るとMiB/s
- 同じコマンドを数回実行し、値のばらつきを確認

## 条件変更

- ファイルサイズ：両方のURLを`1k.bin`または`1m.bin`に変更
- 同時接続数：両方の`-c`を`1`または`128`に変更
- 別マシンから実行：`127.0.0.1`をサーバーIPに変更。8080 / 8081番への接続を確認
- sendfile無効：`nginx/nginx.conf`の`sendfile on;`を`sendfile off;`に変更し、nginxを再起動

## 停止

Pingora・nginxを起動したターミナルでそれぞれ`Ctrl+C`。

## 比較条件の確認

- Pingora 0.2.0 + static-files-module 0.2.0（最新Pingoraの比較ではない）
- 両方ともworker数1、圧縮OFF、アクセスログOFF、配信元`www/`
- nginxは初期設定で`sendfile on`
- 同じファイルを繰り返す、OSキャッシュが温まった状態の計測

