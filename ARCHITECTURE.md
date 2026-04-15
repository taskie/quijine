# Architecture

`quijine` は QuickJS を Rust から使うためのバインディング群で、役割ごとに 4 つのクレートを持つ Cargo ワークスペースで構成されている。本ドキュメントは各層の責務、所有権/寿命モデル、主要な設計判断を俯瞰する。

## Workspace overview

```
quijine_core        ─┐
                     ├─ quijine (high-level)  ─┐
libquickjs-sys  ─────┘                         ├─ serde_quijine
                                               ├─ quijine_jj (binary "jj")
```

| crate           | kind    | 責務                                                                       |
| --------------- | ------- | -------------------------------------------------------------------------- |
| `quijine_core`  | library | `libquickjs-sys` を Rust 型で薄く包んだ低レベル FFI 層 (手動 RC・`unsafe`) |
| `quijine`       | library | 自動 refcount、型付き `Value`、クラス登録、`Context`/`Runtime` のスコープ管理 |
| `serde_quijine` | library | serde の `Serializer` / `Deserializer` と `quijine::Value` の相互変換     |
| `quijine_jj`    | binary  | `jq` 風の JSON プロセッサ CLI (`jj`)。QuickJS を JS 式評価エンジンとして使う |

ライブラリ 3 つ (`quijine_core` / `quijine` / `serde_quijine`) は MSRV 1.65 を守るが、`quijine_jj` は制約なしで最新依存を利用する (`weekly.yml` で別ジョブとして分離)。

---

## Layer 1: `quijine_core` — typed FFI

### 基本型

すべての主要型は `Copy` な薄い構造体で、寿命パラメータ `'q` と `NonNull<_>` を持つ:

```rust
pub struct Runtime<'q>(NonNull<ffi::JSRuntime>, Covariant<'q>);
pub struct Context<'q>(NonNull<ffi::JSContext>, Covariant<'q>);
pub struct Value<'q>(ffi::JSValue, Covariant<'q>);
pub struct Atom<'q>(ffi::JSAtom, Covariant<'q>);
```

- `Covariant<'a> = PhantomData<&'a ()>` (`marker.rs`) で寿命のみ表現。
- `NonNull` を内包しているため `Runtime` / `Context` は自動的に `!Send + !Sync`。
- 所有権は持たない: `Value` のクローンは単なるコピーで、refcount 操作は行わない。`dup_value` / `free_value` を呼ぶのは**呼び出し側の責務**。
- ほぼすべての関数は 1:1 で `ffi::JS_*` を呼ぶ。セーフティは「寿命トークン `'q` が有効である限りポインタも有効」という前提で成立。

### マクロ層

`macros.rs` は QuickJS の C コールバックを Rust の関数/クロージャに変換する極薄ラッパを提供する:

- `js_c_function!` — `JSCFunction` 互換 `unsafe extern "C" fn wrap(...)` を生成し、内部で `convert_function_arguments` を呼ぶ。
- `js_class_finalizer!` / `js_class_gc_mark!` / `js_class_call!` — クラス C API 用。
- `js_module_init_func!` — モジュール初期化関数用。

いずれも `wrap<'q>` のように寿命を生成側に渡し、本体の型注釈 (`fn(Context<'q>, ...) -> Value<'q>`) でそれを利用する。

### その他のモジュール

- `class.rs` — `ClassId` と `ClassDef`。`JS_NewClassID` がスレッドセーフでないため `lazy_static` による `Mutex<()>` で保護。
- `alloc.rs` — Rust グローバルアロケータを使う `JSMallocFunctions` (非 Windows で使用)。
- `flags.rs` — `bitflags!` による `EvalFlags` / `GpnFlags` / `PropFlags` / `ReadObjFlags` / `WriteObjFlags`。
- `string.rs` — QuickJS から取得した C 文字列の RAII ラッパ (`JS_FreeCString`)。
- `function.rs` — `CFunctionListEntry` と `convert_function_arguments` / `convert_function_result` の変換ヘルパ。
- `internal/util.rs` — packed 構造体の安全なバイトコピー用ヘルパ (`ref_sized_to_slice` など)。

---

## Layer 2: `quijine` — high-level API

この層の中心アイデアは **`Value` の `Clone`/`Drop` で自動 refcount を行う** こと。ユーザーは `unsafe` なしで QuickJS 値を扱える。

### `Value<'q>` (src/value.rs)

```rust
pub struct Value<'q> {
    value: qc::Value<'q>,
    context: qc::Context<'q>,
}

impl Clone for Value<'_> { /* JS_DupValue */ }
impl Drop  for Value<'_> { /* JS_FreeValue */ }
```

- 各 `Value` は **自身が属する Context を参照として保持** する。`dup`/`free` は context 経由。
- `debug_leak` フィーチャ有効時はグローバル `AtomicU16` に基づく `_debug_count` フィールドを追加してログを出す。

### 型付きサブクラス (src/types.rs)

`Value` の特定タグに限定した `#[repr(transparent)]` なニュータイプを多数定義する。2 段階の親クラスで「参照型」と「値型」を分けている:

```
Value ──┬── HasPtr ──┬── BigDecimal / BigInt / BigFloat / Symbol
        │            ├── String / Module / FunctionBytecode / Object
        │            └── (ClassObject<C>)
        └── HasVal ──┬── Int / Bool / Null / Undefined / Uninitialized
                     ├── CatchOffset / Exception / Float64
```

- `impl_try_from_value!` マクロで `Value → T` の `TryFrom` と `Tag` チェックを同時生成。
- `impl_into_qj!` で `&str`, `StdString`, `i32`, `bool`, `f64`, `()` などから `Value` への変換を提供。
- `Variant<'q>` は同じ分類を `enum` で表現したランタイム分岐用。

### 変換トレイト (src/convert.rs)

4 つのコア trait と tuple arity 1..=4 の impl:

| trait            | 方向          | 備考                                                    |
| ---------------- | ------------- | ------------------------------------------------------- |
| `IntoQj<'q>`     | Rust → JS     | `T: Into<Value>`, `Option<T>`, `Vec<T>`, `HashMap`, `BTreeMap` |
| `FromQj<'q>`     | JS → Rust     | `TryFrom<Value>`, `Option`, `Vec`, `HashMap`, `BTreeMap` |
| `IntoQjMulti<'q>`| 複数 → JS 引数 | tuple 1..=4, `Vec<T>`, `&[Value]`, `()`                |
| `FromQjMulti<'q>`| JS 引数 → 複数 | tuple 1..=4, `Vec<Value>`, `()`                        |
| `IntoQjAtom<'q>` | Rust → Atom   | プロパティキー用                                        |

### `Runtime<'r>` / `RuntimeScope` (src/runtime.rs)

所有権のルート。`RuntimeScope` は `Drop` で `Box<RuntimeOpaque>` を回収してから `JS_FreeRuntime` を呼ぶ。

```rust
pub struct RuntimeOpaque {
    registered_classes: HashMap<TypeId, qc::ClassId>,
    class_defs: HashMap<qc::ClassId, qc::ClassDef>,
    class_names: HashSet<CString>,
}
```

- 生の `JS_GetRuntimeOpaque()` を `*mut RuntimeOpaque` にキャストしてアクセス。
- クラス登録は **Rust の `TypeId` をキーにした `ClassId` のキャッシュ**。同一型を複数回登録しても整合する。

### `Context<'q>` / `ContextScope` (src/context.rs)

同様に `ContextOpaque` (`registered_classes: HashSet<TypeId>`) を `Box::into_raw` で opaque に格納。`Drop` 順序が重要 (先に `Context` → `Runtime`)。

`Context` は以下を提供する:
- 値作成 (`new_int32`, `new_float64`, `new_string`, `new_array`, `new_object`, ...)
- 評価 (`eval`, `eval_into`, `eval_function`, `compile`, `parse_json`, `json_stringify`, `write_object`, `read_object`)
- 例外 (`throw_*`, `take_exception`)
- 関数 (`new_function`, `new_function_from`, `new_callback`)
- モジュール (`new_c_module`)
- クラス登録 (`register_class`)

`ContextAddIntrinsicExt` trait (src/context_ext.rs) で `add_intrinsic_*` 系を別トレイトに分離している。

### クロージャコールバック (src/context.rs `new_callback`)

QuickJS の C コールバックは素の関数ポインタしか受け取れない。これを回避するために以下の方式を採用している:

1. `Box<Callback<'q, 'q, R>>` を作り、さらにそれを `Box` に入れる。
2. 外側の `Box` を `new_array_buffer_from_boxed` で ArrayBuffer に変換し、QuickJS の値としてエンジンに保持させる。
3. `JS_NewCFunctionData` に固定の `extern "C" fn call` とその ArrayBuffer を渡す。
4. 呼び出し時は ArrayBuffer から `Box<Callback<R>>` を取り出して呼ぶ。

ArrayBuffer のライフサイクルが QuickJS 側の refcount に紐づくため、クロージャは値が GC されたタイミングで自動的に解放される。

### クラス登録 (src/class.rs)

`Class` トレイトを実装した型を QuickJS のクラスとして公開する。プロセスは:

1. `Runtime` ごとに 1 回: `ClassId` を生成し `JS_NewClass` を呼ぶ (finalizer を含む `ClassDef` を作成)。
2. `Context` ごとに 1 回: プロトタイプオブジェクトを作成し、`Class::define_properties`/`setup_proto` を呼ぶ。プロトタイプを `JS_SetClassProto` で登録。
3. インスタンス作成時: `Value::set_opaque` に `Box::into_raw` した Rust オブジェクトを埋め、finalizer で `Box::from_raw` して解放。

`ClassProperties` トレイトに `define_method` / `define_method_mut` / `define_get` / `define_get_mut` / `define_set_mut` / `define_get_set_mut` / `define_get_mut_set_mut` が存在し、`&C` と `&mut C` の受け取りを区別する。

### エラー型 (src/error.rs)

```rust
pub struct Error { pub kind: ErrorKind, pub value: ErrorValue }

pub enum ErrorValue {
    None,
    String(String),
    JsError(JsErrorData),          // name, message, file_name, line_number, stack
    External(Arc<dyn StdError + Send + Sync>),
}
```

`ExternalError` trait (ブランケット impl) で任意のエラー型を `Error::external()` 化できる。`from_js_error` は JS 例外オブジェクトから標準プロパティを抽出する。

---

## Layer 3: `serde_quijine`

`serde::Serializer` / `Deserializer` を `quijine::Value` との変換で実装する。

- `to_qj(ctx, T)` — `T: Serialize` を `Value<'q>` に。
- `from_qj(v)` — `Value<'q>` を `T: Deserialize` に。
- `Error` は `quijine::Error` のラッパで双方向 `From` を持つ。

実装上の注意:

- `serialize_bytes` は `unimplemented!()`。
- `serialize_u64` は `i32` に収まらない値を `f64` に落とす (コメント上「dirty hacking」)。`serde_json::Value` の整数モデルに合わせた妥協で、2^53 を超えると精度落ち。
- `serialize_i64` は `new_int64` (内部で `Float64` または `BigInt` に委譲される QuickJS の動作に依存)。

---

## Layer 4: `quijine_jj`

`jq` 風 CLI。`clap` で引数を取り、QuickJS を式評価エンジンとして使う。

処理フロー:

1. スクリプトを一度だけ `FunctionBytecode` にコンパイル (`EvalFlags::FLAG_COMPILE_ONLY`)。
2. 各入力 (stdin/ファイル/raw/slurp) を読み取り、`serde_json::Value` 経由で `to_qj` → QuickJS 値に変換。
3. `_` (current), `_I` (index), `_F` (filename), `_P` (print function) をグローバルにセットして bytecode を `eval_function` で実行。
4. 結果を `serde_quijine::from_qj` で `serde_json::Value` に戻し、`ColoredFormatter` / `PrettyFormatter` / `CompactFormatter` で出力。

特筆すべきテクニック:
- 同じ bytecode を全入力に対して再利用するので parse コストは初回のみ。
- `_P` (print) はユーザースクリプト内から明示的に複数出力するために Rust 側で実装した callback として注入する。
- `-i` フラグで iterator 対応 (`for v of result` 的に `result.iterator()` を回す)。

---

## 横断的な設計方針

### 所有権モデル

- **所有**: `RuntimeScope` / `ContextScope` が RAII で所持する (Drop で `JS_FreeContext` / `JS_FreeRuntime`)。
- **共有**: 高レベル `Value` の `Clone` / `Drop` が `JS_DupValue` / `JS_FreeValue` に直結して自動 refcount。
- **寿命**: `'q` (または `'r`) 寿命パラメータが「この Value/Context は元の Runtime が生きている間のみ有効」を静的に強制する。

### 安全性の前提

- QuickJS の opaque slot に `Box<RuntimeOpaque>` / `Box<ContextOpaque>` を `Box::into_raw` で格納し、Drop 時に `Box::from_raw` で回収する。前提が崩れる (例: 他の何かが opaque を書き換える) と UB。
- `Value::opaque_ptr::<C>()` は生ポインタを返すラッパで、`opaque` と `opaque_mut` が同じポインタから `&C` / `&mut C` を作る。**同じ Rust 型 `C` で finalize される限り** 安全だが、アライアス不変条件はユーザー側の責任。
- `set_opaque` と finalizer は型パラメータ `C` で連動しているため、型を跨いだ誤用は現状コンパイル時に防げない。

### スレッドモデル

QuickJS の `JSRuntime` はスレッドをまたがない設計。本クレートでは `NonNull<JSRuntime>` を内蔵することで自動的に `!Send + !Sync` となる。`JS_NewClassID` はプロセス全体でグローバルな連番を触るため `quijine_core::class` で `Mutex` により保護されている。

### フィーチャフラグ

| feature            | 説明 |
| ------------------ | ---- |
| `debug_leak`       | `Value` にデバッグカウンタを付与し、dup/free/from/drop をトレースログ出力する。 |
| `c_function_list`  | `CFunctionListBuilder` と `CStringArena` を有効化。QuickJS のネイティブ C 関数リスト定義 API 向け。 |

---

## テスト構造

ドキュメントに乏しい代わりに、各クレートの `tests/` ディレクトリが事実上のユースケース集になっている:

- `quijine_core/tests/` — `examples.rs` (クラス登録低レベル例), `class.rs`, `array_buffer.rs`, `rw_object.rs`
- `tests/` (quijine) — `class.rs`, `convert.rs`, `examples.rs`, `exception.rs`, `module.rs`, `property.rs`, `runtime.rs`, `rw_object.rs`, `value.rs`
- `serde_quijine/tests/` — `examples.rs`, `json.rs`
- `quijine_jj/tests/` — `command.rs` (`assert_cmd` ベースの CLI E2E)

README からは `tests/examples.rs` が「使い方」として参照されている。

---

## CI

- `ci.yml` — push/PR で test + lint + llvm-cov + Codecov upload。
- `weekly.yml` — 週次実行。3 ジョブに分割:
  - `check-oldest` (libs) — Rust 1.65 + `cargo update -Z minimal-versions` でライブラリ 3 crate のみテスト。
  - `check-latest` (libs) — Rust beta + `cargo update` で最新に、ライブラリ 3 crate をテスト。
  - `check-jj` — Rust stable/beta で `quijine_jj` のみインストール & テスト (MSRV 非拘束)。

`release.yml` は未作成。
