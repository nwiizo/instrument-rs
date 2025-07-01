# instrument-rs TODO

## ステータス凡例
- 🟦 **待機中** (Waiting) - 他のタスクの完了待ち
- 🟨 **取り込み中** (In Progress) - エージェントが作業中
- 🟩 **レビュー待ち** (Review) - 実装完了、レビュー待機
- ✅ **完了** (Done) - マージ済み
- 🟥 **ブロック** (Blocked) - 問題発生、対応必要

## 作業担当
- **Agent A**: 基礎実装・CLI担当
- **Agent B**: コア機能担当
- **Agent C**: 高度な機能担当
- **未割当**: 担当エージェント未定

## CLIツール実装タスク（READMEとの差分解消）

### 最優先度 (Critical - READMEに記載済みの機能)

- [ ] 🟨 **CLIエントリーポイントの実装 (`src/main.rs`)** [Agent A - Step 4実装中]
  - clap v4を使用したコマンドライン引数解析
  - `--trace-from-endpoints`オプションの実装
  - `--framework`オプション（axum|actix|rocket|tonic|auto）
  - `--format`オプション（human|json|dot|mermaid）
  - その他のオプション（threshold, min-lines, max-depth等）

- [ ] 🟦 **エンドポイントベース解析機能の実装** [待機: CLI完了後]
  - HTTPエンドポイントからの実行パス追跡
  - gRPCサービスメソッドの検出
  - エンドポイントからのコールグラフ構築
  - クリティカルパスの特定と優先順位付け

- [ ] 🟦 **フレームワーク検出の拡張** [待機: CLI完了後]
  - Actix-web対応（`src/frameworks/actix.rs`）
  - Rocket対応（`src/frameworks/rocket.rs`）
  - Tonic対応（`src/frameworks/tonic.rs`）
  - 自動フレームワーク検出機能

- [ ] 🟦 **CLI向け出力フォーマッターの実装** [待機: CLI完了後]
  - Human-readableなツリー表示（カラー対応）
  - DOT形式出力の実装
  - プログレスバーとステータス表示
  - エラーパスのハイライト表示

- [ ] 🟦 **パターンファイルのサポート** [待機: CLI完了後]
  - YAMLパターンファイルの読み込み
  - デフォルトパターン（`patterns/default.yml`）の作成
  - カスタムパターンのマージ機能

### 高優先度 (High Priority - 既存設計)

## 実践的な機能設計の実装タスク

### 高優先度 (High Priority)

- [ ] 🟦 **既存計装の検出・分析システムの実装** [未割当 - Phase 2]
  - ExistingInstrumentation構造体
  - InstrumentationType列挙型
  - QualityScoreによる品質評価
  - InstrumentationIssueによる問題検出

- [ ] **計装検出ロジックの構築**
  - #[instrument]マクロの検出
  - 手動span作成の検出
  - log!マクロ使用の分析
  - 品質チェックと問題検出

- [ ] **差分計装提案システムの作成**
  - InstrumentationDiff構造体
  - MigrationStep列挙型
  - ImpactAnalysisによる影響分析
  - 移行ステップの自動生成

### 中優先度 (Medium Priority)

- [ ] **計装カバレッジ分析の実装**
  - カバレッジメトリクスの計算
  - 重要度別・モジュール別分析
  - 実行パスカバレッジ
  - ギャップ分析レポート

- [ ] **パフォーマンス影響推定器の構築**
  - 計装オーバーヘッドの推定
  - ホットパス影響分析
  - パフォーマンス警告の生成
  - 代替案の提案

- [ ] **チーム標準化サポートの作成**
  - スタイルガイド設定の適用
  - 命名規則のチェック
  - 必須フィールドの検証
  - セキュリティ違反の検出

### 低優先度 (Low Priority)

- [ ] **段階的導入計画システムの設計**
  - フェーズ別実装計画の生成
  - リスク評価
  - ロールバック計画
  - 成功基準の定義

- [ ] **動的計装設定システムの実装**
  - ランタイム最適化ルール
  - 条件ベースのアクション
  - サンプリングレート調整
  - 動的ログレベル変更

- [ ] **継続的改善メトリクスダッシュボード**
  - トレンド分析
  - 問題解決の追跡
  - 次のアクション提案
  - 改善効果の可視化

## 実運用を考慮した高度な機能

### コスト最適化 (Cost Optimization)

- [ ] **ログボリューム予測とコスト計算器**
  - TelemetryMetricsによるボリューム計測
  - PricingModelの実装（DataDog, CloudWatch等）
  - 月額コスト予測とブレークダウン
  - サンプリング提案による節約額計算

- [ ] **コスト最適化レコメンダー**
  - 高頻度エンドポイントのサンプリング提案
  - 構造化フィールドへの変換提案
  - 条件付きログ記録の実装
  - コスト削減インパクトの可視化

### インシデント対応 (Incident Response)

- [ ] **エラー追跡可能性の強化**
  - エラーコンテキストの自動追加
  - 相関IDの要件定義
  - 分散トレース要件の実装
  - スタックトレースの自動記録

- [ ] **インシデント対応レディネス分析**
  - クリティカルエラーパスの検出
  - 不足している診断情報の特定
  - リトライ情報の記録
  - ゲートウェイレスポンスの追跡

### 規制コンプライアンス (Compliance)

- [ ] **データプライバシー自動チェッカー**
  - GDPR対応（PII検出とハッシュ化）
  - PCI-DSS対応（カード情報のマスキング）
  - HIPAA対応（PHIパターン検出）
  - コンプライアンス違反の自動検出

- [ ] **コンプライアンスレポート生成**
  - 違反箇所の特定と修正提案
  - ログ保持期間の確認
  - 機密データのサニタイズ提案
  - 規制別の対応状況レポート

### マルチテナント対応 (Multi-tenant)

- [ ] **テナント別計装設定システム**
  - TenantConfigによる個別設定
  - ログレベルのテナント別制御
  - サンプリングレートのカスタマイズ
  - テナント固有フィールドの追加

- [ ] **テナントプロファイル別推奨設定**
  - エンタープライズ向け設定
  - スタートアップ向け設定
  - 規制業界向け設定
  - カスタムプロファイルの作成

### 開発者エクスペリエンス (Developer Experience)

- [ ] **学習曲線を考慮した段階的提案**
  - スキルレベル別の学習パス生成
  - 週次の段階的導入計画
  - カスタマイズされたサンプルコード
  - チーム固有のパターン認識

- [ ] **インタラクティブな導入ガイド**
  - 最も使用される関数からの開始提案
  - エラー率の高い箇所の優先対応
  - 実際のコードパターンに基づく例
  - 進捗トラッキングとフィードバック

### CI/CD統合 (CI/CD Integration)

- [ ] **品質ゲートの実装**
  - 最小カバレッジチェック
  - センシティブフィールドの検出
  - パフォーマンス影響の評価
  - 自動修正提案の生成

- [ ] **PR自動レビュー機能**
  - 計装カバレッジの差分表示
  - 新規・解決済み問題のサマリー
  - インライン修正提案
  - チーム標準への準拠チェック

## CLIツール実装の詳細設計

### コマンドライン引数構造体
```rust
#[derive(Parser)]
#[command(name = "instrument-rs")]
#[command(about = "Rust observability detection tool")]
struct Cli {
    /// Paths to analyze
    #[arg(default_value = ".")]
    paths: Vec<PathBuf>,
    
    /// Detection threshold (0.0-1.0)
    #[arg(short, long, default_value = "0.8")]
    threshold: f64,
    
    /// Minimum function lines
    #[arg(short = 'm', long, default_value = "3")]
    min_lines: usize,
    
    /// Trace execution paths from entry points
    #[arg(long)]
    trace_from_endpoints: bool,
    
    /// Framework to use
    #[arg(long, value_enum)]
    framework: Option<Framework>,
    
    /// Maximum call depth
    #[arg(long, default_value = "10")]
    max_depth: usize,
    
    /// Include test endpoints
    #[arg(long)]
    include_tests: bool,
    
    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "human")]
    format: OutputFormat,
    
    /// Filter paths by regex
    #[arg(long)]
    filter_path: Option<String>,
    
    /// Custom patterns file
    #[arg(long)]
    patterns: Option<PathBuf>,
}
```

### モジュール再構成計画
```
src/
├── main.rs              # NEW: CLIエントリーポイント
├── cli/                 # NEW: CLI関連モジュール
│   ├── mod.rs
│   ├── args.rs         # コマンドライン引数定義
│   └── runner.rs       # CLI実行ロジック
├── analyzer/            # RENAME: ast/ → analyzer/
│   ├── mod.rs
│   ├── ast.rs          # 既存のAST解析
│   ├── endpoints.rs    # NEW: エンドポイント検出
│   └── trace.rs        # NEW: 実行パス追跡
├── frameworks/          # RENAME: framework/ → frameworks/
│   ├── mod.rs
│   ├── auto.rs         # NEW: 自動検出
│   ├── axum.rs         # 既存
│   ├── actix.rs        # NEW
│   ├── rocket.rs       # NEW
│   └── tonic.rs        # NEW
├── detector/            # NEW: 計装ポイント検出
│   ├── mod.rs
│   ├── critical.rs     # クリティカルパス検出
│   └── existing.rs     # 既存計装の分析
└── output/              # 既存の拡張
    ├── human.rs        # NEW: CLI向け出力
    └── dot.rs          # NEW: DOT形式
```

## Git Worktreeを使った並行開発プロセス

### Worktree構成と開発フロー

```bash
# メインリポジトリの構造
instrument-rs/                    # メインブランチ (main)
├── instrument-rs-cli/           # feature/cli-implementation
├── instrument-rs-detection/     # feature/instrumentation-detection
├── instrument-rs-coverage/      # feature/coverage-analysis
├── instrument-rs-performance/   # feature/performance-impact
├── instrument-rs-cost/          # feature/cost-optimization
├── instrument-rs-compliance/    # feature/compliance-checking
├── instrument-rs-tenant/        # feature/multi-tenant
├── instrument-rs-dx/            # feature/developer-experience
└── instrument-rs-cicd/          # feature/cicd-integration
```

### 各Worktreeでの開発手順

1. **Worktree作成とセットアップ**
```bash
# CLIの基本実装用
git worktree add ../instrument-rs-cli feature/cli-implementation

# 既存計装検出機能用
git worktree add ../instrument-rs-detection feature/instrumentation-detection

# 各機能ごとに同様に作成
```

2. **各Worktreeでの品質チェック**
```bash
# 各worktreeで実行すべきコマンド
cd ../instrument-rs-{feature}/
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo doc --no-deps
```

3. **Worktree管理スクリプト (`scripts/worktree-check.sh`)**
```bash
#!/bin/bash
# 全worktreeで品質チェックを実行

for dir in ../instrument-rs-*/; do
    echo "Checking $dir"
    cd "$dir"
    cargo fmt --check || exit 1
    cargo clippy -- -D warnings || exit 1
    cargo test || exit 1
done
```

### 開発優先順位とWorktree割り当て

1. **Phase 1: 基礎実装** (最初に着手)
   - `instrument-rs-cli`: CLIの基本実装とmain.rs
   - 他のClaude CodeがStep 4を完了後すぐに開始

2. **Phase 2: コア機能** (CLI完了後)
   - `instrument-rs-detection`: 既存計装の検出
   - `instrument-rs-coverage`: カバレッジ分析
   - 並行して開発可能

3. **Phase 3: 高度な機能** (コア機能の基盤完成後)
   - `instrument-rs-cost`: コスト最適化
   - `instrument-rs-compliance`: コンプライアンス
   - `instrument-rs-performance`: パフォーマンス分析

4. **Phase 4: 統合機能** (他機能との連携必要)
   - `instrument-rs-tenant`: マルチテナント
   - `instrument-rs-dx`: 開発者体験
   - `instrument-rs-cicd`: CI/CD統合

### マージ戦略

1. **機能完成時のチェックリスト**
   - [ ] ユニットテストのカバレッジ80%以上
   - [ ] 統合テストの実装
   - [ ] ドキュメント更新
   - [ ] `cargo fmt`と`cargo clippy`のパス
   - [ ] CHANGELOGへの追記

2. **段階的マージ**
   ```bash
   # 機能ブランチからmainへのPR作成
   cd ../instrument-rs-{feature}/
   git push -u origin feature/{feature-name}
   gh pr create --base main --title "feat: {feature description}"
   ```

## タスク進捗管理表

| Worktree | 機能 | 担当 | ステータス | 開始日 | 更新日 | ブロッカー |
|----------|------|------|------------|--------|--------|------------|
| instrument-rs-cli | CLI実装 | Agent A | 🟨 取り込み中 | 2025-07-01 | - | なし |
| instrument-rs-detection | 既存計装検出 | 未割当 | 🟦 待機中 | - | - | CLI完了待ち |
| instrument-rs-coverage | カバレッジ分析 | 未割当 | 🟦 待機中 | - | - | CLI完了待ち |
| instrument-rs-performance | パフォーマンス | 未割当 | 🟦 待機中 | - | - | Phase 2完了待ち |
| instrument-rs-cost | コスト最適化 | 未割当 | 🟦 待機中 | - | - | Phase 2完了待ち |
| instrument-rs-compliance | コンプライアンス | 未割当 | 🟦 待機中 | - | - | Phase 2完了待ち |
| instrument-rs-tenant | マルチテナント | 未割当 | 🟦 待機中 | - | - | Phase 3完了待ち |
| instrument-rs-dx | 開発者体験 | 未割当 | 🟦 待機中 | - | - | Phase 3完了待ち |
| instrument-rs-cicd | CI/CD統合 | 未割当 | 🟦 待機中 | - | - | Phase 3完了待ち |

### ステータス更新方法

```bash
# タスク開始時
# TODO.mdの該当行を 🟦 → 🟨 に変更し、担当者と開始日を記入

# レビュー完了時
# TODO.mdの該当行を 🟨 → 🟩 に変更し、更新日を記入

# マージ完了時
# TODO.mdの該当行を 🟩 → ✅ に変更

# ブロック発生時
# TODO.mdの該当行を現在のステータス → 🟥 に変更し、ブロッカーを記入
```

## 実装の詳細

各タスクの詳細な設計と実装例は、プロジェクトのドキュメントに記載されている実践的な機能設計を参照してください。

### Cargo.tomlの更新事項
- [x] ✅ package.descriptionをobservability関連に更新 [完了]
- [x] ✅ keywordsを["observability", "tracing", "instrumentation", "monitoring", "rust"]に変更 [完了]
- [x] ✅ [[bin]]セクションは既に追加済み（main.rsのパスが設定済み） [完了]