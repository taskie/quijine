# TODO / 改善候補

`ARCHITECTURE.md` の設計レビューから洗い出した改善項目を優先度別にまとめる。優先度は「現行コードの安全性・正しさに直接影響する度合い」で判定している。

## 🔴 High — 安全性・正しさに関わるもの

### 1. `Value::opaque_ptr` による潜在的な `&mut` エイリアス

**現状** (`src/value.rs:428` 付近): `opaque` と `opaque_mut` は同じ生ポインタから `&C` / `&mut C` を作る。同じ JS オブジェクトに対して 2 つの `ClassObject` を持ち、一方で `opaque()`、もう一方で `opaque_mut()` を呼ぶと `&C` と `&mut C` が同時存在して UB。

**案**: `RefCell<C>` で包む (実行時 borrow check) か、`UnsafeCell<C>` + `unsafe fn opaque_mut` に格下げして呼び出し側に証明を求める。前者は動作安定、後者は zero-cost。

### 2. `set_opaque` と finalizer の型 mismatch を型で防げない

**現状** (`src/class.rs:78` `finalize`): finalizer は型パラメータ `C` に基づいて `Box::from_raw(p as *mut C)` を呼ぶ。もし誤って別の型 `D` で `set_opaque` したインスタンスに対してこの finalizer が呼ばれると UB。現在は「`Class` trait の使用者が整合性を守る」前提。

**案**: `ClassId` に対応する Rust の `TypeId` を `RuntimeOpaque` に持たせ、`set_opaque` 時に `assert_eq!(TypeId::of::<C>(), ...)` する。あるいは `ClassObject<C>` の生成ルートを `new_object_with_opaque` に一本化し、`set_opaque` を pub にしない。

### 3. `Tag::CatchOffset` に対応する `Variant` のデフォルト初期化

**現状** (`src/value.rs:182`): `Variant::CatchOffset(Default::default())` — `CatchOffset` の実値を捨てて `Opaque<4>` のデフォルトに差し替えている。元の値を失うため `to_variant` → 戻しラウンドトリップが壊れる。

**案**: `Variant::CatchOffset` に実値を持たせるか、`CatchOffset` 自体を `Copy` にして保持する。

### 4. `serde_quijine::serialize_bytes` が `unimplemented!()`

**現状** (`serde_quijine/src/ser.rs:94`): `&[u8]` を serialize すると panic する。`bincode` 等ではなく `serde_json` が主なユーザーなので露見しにくいが、任意の `Serialize` 実装を受ける API としては危険。

**案**: QuickJS の `ArrayBuffer` / `Uint8Array` に変換するか、少なくとも `Error` を返す。

### 5. `serde_quijine::serialize_u64` の精度落ち

**現状** (`serde_quijine/src/ser.rs:67`): `u64` が `i32` に収まらなければ `f64` へキャスト。2^53 を超える値で silent な精度落ちが起きる。

**案**: QuickJS の `BigInt` に変換するパス (`new_big_int_u64` 相当) を用意する。最低限、精度落ちが起きるケースで `Error` を返すオプションを足す。

### 6. `convert.rs` のタプルマクロ typo

**現状** (`src/convert.rs:210`): `impl_into_qj_multi_for_tuple! { for 4 => (0 => T0, 1 => T1, 2 => T2, 3 => T4) }` — 最後の識別子は `T3` であるべき。コンパイルは通るが読者を混乱させる。

**案**: `T4` → `T3` に修正。

---

## 🟡 Medium — API 品質・保守性

### 7. `ClassProperties` の define_* メソッドが 8 種類

**現状** (`src/class.rs:11-59`): `define_method` / `define_method_mut` / `define_get` / `define_get_mut` / `define_set_mut` / `define_get_set_mut` / `define_get_mut_set_mut` と、`&C` / `&mut C` の組合せ爆発で 8 変種を手書きしている。

**案**: `Fn(&C, ...) | Fn(&mut C, ...)` を受ける共通 trait `Callable` を定義し、マクロかビルダーパターンに集約する。呼び出し側の DX (補完) を損ねない範囲で。

### 8. `FromQjMulti` / `IntoQjMulti` のタプル arity が 4 止まり

**現状** (`src/convert.rs:149-152, 207-210`): callback の引数が 5 個以上必要な場合は直接 `Vec<Value>` を使うしかない。

**案**: `impl_from_qj_multi_for_tuple!` を arity 16 (serde / `hlist` 式の一般慣例) まで展開する。コードが増えるがマクロ化済みなのでコストは小さい。

### 9. `ValueIterator::next` の過剰 clone

**現状** (`src/value.rs:617-625`): `next_impl` が `self.next.clone().apply(self.iterator.clone(), &[])` と 2 回 `Value::dup` する。イテレータ毎ステップで 2 回の refcount 操作が発生する。

**案**: `apply` を `&Value` で取るように変更するか、`next_impl` でクロージャを事前に `get` しておく。

### 10. `RuntimeOpaque::class_defs` の get-after-insert

**現状** (`src/class.rs:279-282`): `rt.register_class_def(clz, class_def); let class_def = rt.class_def(clz).unwrap(); rt.new_class(clz, class_def)` — HashMap に入れてすぐ参照を取り直す不自然な流れ。`new_class` が `&ClassDef` を要求するためだが、所有権管理の都合で経由する。

**案**: `register_class_def` が `&ClassDef` を返すように変える、あるいは `ClassDef` を `Arc` で共有して clone する。

### 11. `Error::from_js_error` が文字列に落とす

**現状** (`src/error.rs:117`): `JsErrorData` は `name` / `message` / `file_name` / `line_number` / `stack` のみ。カスタムエラー型のプロパティが失われる。

**案**: 元の `Value<'q>` を `Arc<JsException>` として保持するオプションを追加する。寿命が `'q` に閉じているため `'static` 変換にはシリアライズが必要だが、`write_object` / `read_object` を使う案も考えられる。

### 12. `new_function` vs `new_function_from` の二重化

**現状** (`src/context.rs:338-358`): 固定シグネチャ版 (`Fn(Context, Value, &[Value]) -> Result<Value>`) と汎用版 (`Fn(Context, T, A) -> Result<R>`) が並ぶ。ユーザーが迷う。

**案**: `new_function` を deprecated にして `new_function_from` 一本化を検討。あるいは前者を後者のシンタックスシュガーに統合。

### 13. `Tag::*` チェックが文字列経由になっている箇所

**現状** (`src/value.rs:167-186` `to_variant`): `self.clone().try_into().unwrap()` の連鎖でタグ → ニュータイプ変換している。`try_into` は内部でもう一度 `Tag` を見る → 二重チェック。

**案**: `to_variant` 内で `unsafe { as_any::<T>().clone() }` を直接使う (タグが確定しているので安全)。

### 14. `README.md` が 1 行しかない

**現状**: `See examples` のみ。crates.io 公開を想定するならもう少し説明が要る。

**案**: `ARCHITECTURE.md` の overview と、`tests/examples.rs` からの最小スニペットを転記したクイックスタートを追加。

---

## 🟢 Low — 依存・ツール・CI

### 15. `atty` の置き換え

**現状** (`quijine_jj/src/main.rs:81, 127`): `atty = "0.2.14"` は deprecated で RUSTSEC advisory あり。Rust 1.70+ の `std::io::IsTerminal` が代替になる。`quijine_jj` は MSRV 非拘束なので即置換可能。

**案**: `atty::is(atty::Stream::Stdout)` → `std::io::stdout().is_terminal()`。

### 16. `lazy_static` の置き換え候補

**現状** (`quijine_core/src/class.rs:44`): ライブラリのため MSRV 1.65 は維持するが、将来 MSRV を 1.70 以降に上げれば `std::sync::OnceLock` に置き換えられる。

**案**: MSRV 引き上げ計画と一緒に検討。単体で MSRV を上げる価値は小さい。

### 17. MSRV の引き上げ検討

**現状**: ライブラリは 1.65 に固定。`bitflags 2.11`、`serde 1.0.228`、`log 0.4.29` 等は通るが、`env_logger 0.11` (要 1.71) や `rand 0.9` (要 1.63、ただし API リネーム) など新しい依存を避けている。

**案**: 1.74 (let-else / GATs + clap 4.5 互換) を目標に段階的に引き上げる。`quijine_jj` は既にその前提で動かせる。

### 18. `release.yml` が存在しない

**現状**: `.github/workflows/release.yml` 未作成。`quijine_jj` のバイナリリリース体制がない。

**案**: `v*.*.*` タグで発火する `release.yml` を作成。`quijine_jj` の Linux musl (`taiki-e/install-action@cross`) / macOS / Windows バイナリを `gh release upload` でアップロードする。

### 19. `libquickjs-sys 0.10` が古い

**現状**: `libquickjs-sys` 0.10 がベンダーしている QuickJS は比較的古い (2021 頃のリリース)。Annex B 挙動やモジュール ESM 改善などを取り込めない。

**案**: QuickJS-NG や `rquickjs-sys` への切り替えを検討するか、`libquickjs-sys` 上流に最新の QuickJS をバンドルする PR を出す。影響範囲が広いため検証必須。

### 20. `cargo doc` の CI 化

**現状**: rustdoc 警告を CI で検出していない。公開 API にドキュメントがほぼない (`tests/` に頼っている) 状況と相まって、`#[doc = ""]` 欠落や壊れたリンクが気付きにくい。

**案**: `ci.yml` に `cargo doc --workspace --no-deps` ジョブを追加し、`RUSTDOCFLAGS: -D warnings` で差分のドキュメント欠落を検出する。

### 21. `fmt --check` が `lint` ジョブに同居

**現状** (`ci.yml`): `lint` ジョブが clippy + fmt を両方見る。どちらが失敗したかログから判定することになる。

**案**: 分離するか、`cargo fmt --all -- --check` を先頭に置いて fail-fast 化。

---

## 🔵 Feature / Nice-to-have

### 22. Async / Promise の統合

**現状**: `Runtime::execute_pending_job` と `set_host_promise_rejection_tracker` のパススルーのみ。Rust の `Future` との相互運用はない。

**案**: `Context::eval_into::<PromiseFuture>` 相当を作り、`JS_ExecutePendingJob` を内部で回す `Runtime::run_pending_jobs` を async ランタイムから呼び出せるようにする。設計は大仕事。

### 23. Module loader の高レベル API

**現状** (`src/module.rs`): `Module<'q>` / `ModuleDef<'q>` 型は存在するが、`JS_SetModuleLoaderFunc` を Rust 関数で設定する高レベル API がない。

**案**: `Runtime::set_module_loader(|name| -> Result<String>)` のようなクロージャベース API。C コールバック登録のロジックは `new_callback` と同じ方針で実装可能。

### 24. `Value::set_opaque` の安全化

**現状** (`src/value.rs:449` 周辺): `pub(crate)` だが、`ctx.new_object_with_opaque::<C>(v)` から到達可能。型と opaque の対応を runtime に記録していないため (#2 と同根)。

**案**: 記録と検証を組み合わせる。

### 25. `debug_leak` の計数が `u16`

**現状** (`src/value.rs:29`): `AtomicU16` で `DEBUG_GLOBAL_COUNT` を管理。65536 個以上の値を生成するとラップする。

**案**: `AtomicU64` に変更。性能差は実質ゼロ。

### 26. `CStringArena` の代替

**現状** (`src/arena.rs`): `c_function_list` feature 時に `CStringArena` を手書き。

**案**: `bumpalo` か `typed-arena` で置き換え、自前実装の保守負担を下げる。feature 下なので影響限定的。

### 27. `tests/examples.rs` をドキュメントに昇格

**現状**: 使用例が `tests/examples.rs` にあるだけ。

**案**: `lib.rs` の crate-level doc に `#![doc = include_str!("../README.md")]` と、要約版の例を doctest として置く。`cargo doc` 化 (#20) と合わせて意味が出る。

---

## 参考 — 即効性の目安

| 難易度 | 即効性 | 優先度 | ID |
|--------|--------|--------|----|
| 小     | 小     | 🔴      | #6 (typo 修正) |
| 小     | 中     | 🟢      | #15 (`atty` → `IsTerminal`) |
| 小     | 中     | 🟡      | #9, #13 (クロージャ/変換最適化) |
| 中     | 大     | 🔴      | #1, #2 (型安全化) |
| 中     | 中     | 🟡      | #7, #8 (API 整理) |
| 大     | 大     | 🔵      | #19, #22, #23 (QuickJS 更新 / async / module) |
