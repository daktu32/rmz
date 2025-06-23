#!/usr/bin/env node

import { input, select, confirm } from '@inquirer/prompts';
import chalk from 'chalk';
import ora from 'ora';
import * as fs from 'fs-extra';
import * as path from 'path';
import { PromptSelector } from './lib/promptSelector.js';
import { TemplateProcessor } from './lib/templateProcessor.js';
import { FileManager } from './lib/fileManager.js';
import { Validator } from './lib/validator.js';
import { ProjectConfig, SetupOptions, TechStackConfig, PromptType, TeamConfig } from './lib/types.js';

class SetupAssistant {
  private options: SetupOptions;
  private fileManager: FileManager;
  private templateProcessor: TemplateProcessor;
  private sourceDir: string;
  private targetDir: string = '';

  constructor() {
    this.options = this.parseCliOptions();
    // If running from scripts directory, use parent directory as source
    const currentDir = process.cwd();
    this.sourceDir = path.basename(currentDir) === 'scripts' 
      ? path.dirname(currentDir) 
      : currentDir;
    this.fileManager = new FileManager(this.sourceDir);
    this.templateProcessor = new TemplateProcessor(this.sourceDir);
  }

  private parseCliOptions(): SetupOptions {
    const args = process.argv.slice(2);
    return {
      dryRun: args.includes('--dry-run'),
      skipPromptSelection: args.includes('--skip-prompt'),
      prompt: args.find((arg) => arg.startsWith('--prompt='))?.split('=')[1] as
        | PromptType
        | undefined,
      verbose: args.includes('--verbose') || args.includes('-v'),
    };
  }

  async run(): Promise<void> {
    try {
      console.log(chalk.blue('🚀 Claude Code Development Starter Kit - 新規プロジェクト作成'));
      console.log(chalk.gray('新しいプロジェクトディレクトリを作成します\n'));

      if (this.options.dryRun) {
        console.log(chalk.yellow('🔍 DRY RUN モード - ファイルは変更されません\n'));
      }

      // プロジェクト情報を収集
      const projectInfo = await this.collectProjectInfo();

      // プロンプトを選択または確認
      const { prompt, team } =
        this.options.skipPromptSelection && this.options.prompt
          ? {
              prompt: this.options.prompt,
              team: {
                size: 1,
                type: 'individual',
                industry: 'technology',
                complianceLevel: 'medium',
              } as TeamConfig,
            }
          : await PromptSelector.selectPrompt();

      // 技術スタック情報を収集
      const techStack = await this.collectTechStackInfo();

      // プロジェクト設定を作成
      const config: ProjectConfig = {
        ...projectInfo,
        prompt,
        team,
        techStack,
        customizations: {},
      };

      // 設定を検証
      await this.validateConfiguration(config);

      // サマリーを表示して確認
      if (!this.options.dryRun) {
        await this.showSummaryAndConfirm(config);
      }

      // 新しいプロジェクトディレクトリを作成
      await this.createNewProject(config);

      // 完了メッセージを表示
      this.showCompletionMessage();
    } catch (error) {
      console.error(chalk.red('❌ プロジェクト作成に失敗しました:'), error);
      process.exit(1);
    }
  }

  private async collectProjectInfo(): Promise<
    Omit<ProjectConfig, 'prompt' | 'team' | 'techStack' | 'customizations'>
  > {
    console.log(chalk.blue('\n📝 プロジェクト情報\n'));

    try {
      const projectName = await input({
        message: 'プロジェクト名を入力してください:',
        validate: Validator.validateProjectName,
        transformer: (input: string) => Validator.sanitizeProjectName(input),
      });

      const description = await input({
        message: 'プロジェクトの説明を入力してください:',
        validate: Validator.validateDescription,
        transformer: (input: string) => Validator.sanitizeDescription(input),
      });

      const repositoryUrl = await input({
        message: 'GitHubリポジトリのURLを入力してください:',
        validate: Validator.validateRepositoryUrl,
        default: `https://github.com/your-username/${Validator.generateSlugFromName(projectName)}`,
      });

      const targetPath = await input({
        message: 'プロジェクトを作成するパスを入力してください（絶対パス、相対パス両方可）:',
        default: `../${projectName}`,
        validate: (input: string) => {
          if (!input.trim()) {
            return 'パスを入力してください';
          }
          // パスの形式を簡単にチェック
          const trimmedInput = input.trim();
          if (trimmedInput.includes('..') && !path.isAbsolute(trimmedInput)) {
            // 相対パスの場合の警告
            return true; // 有効だが注意が必要
          }
          return true;
        },
        transformer: (input: string) => {
          const trimmedInput = input.trim();
          if (path.isAbsolute(trimmedInput)) {
            return `${trimmedInput} (絶対パス)`;
          } else {
            return `${trimmedInput} (相対パス)`;
          }
        },
      });

      return {
        projectName,
        description,
        repositoryUrl,
        targetPath,
      };
    } catch (error) {
      console.error(chalk.red('入力エラー:'), error);
      throw error;
    }
  }

  private async collectTechStackInfo(): Promise<TechStackConfig> {
    console.log(chalk.blue('\n🛠️  技術スタック\n'));

    try {
      const projectType = await select({
        message: 'プロジェクトタイプを選択してください:',
        choices: [
          { name: 'Web アプリケーション', value: 'web-app' },
          { name: 'CLI ツール', value: 'cli-tool' },
          { name: 'API サーバー', value: 'api-server' },
          { name: 'その他', value: 'other' },
        ],
      }) as 'web-app' | 'cli-tool' | 'api-server' | 'other';

      let frontend = undefined;
      let cliLanguage = undefined;
      let backend = undefined;
      let database = undefined;

      if (projectType === 'web-app') {
        frontend = await select({
          message: 'フロントエンドフレームワークを選択してください:',
          choices: [
            { name: 'Next.js (React)', value: 'Next.js' },
            { name: 'React', value: 'React' },
            { name: 'Vue.js', value: 'Vue.js' },
            { name: 'Angular', value: 'Angular' },
            { name: 'Svelte', value: 'Svelte' },
            { name: 'その他', value: 'Other' },
          ],
        });
      }

      if (projectType === 'cli-tool') {
        cliLanguage = await select({
          message: 'CLI ツールの言語を選択してください:',
          choices: [
            { name: 'Rust', value: 'Rust' },
            { name: 'Go', value: 'Go' },
            { name: 'Node.js', value: 'Node.js' },
            { name: 'Python', value: 'Python' },
            { name: 'その他', value: 'Other' },
          ],
        });
      }

      if (projectType === 'web-app' || projectType === 'api-server') {
        backend = await select({
          message: 'バックエンドフレームワークを選択してください:',
          choices: [
            { name: 'Node.js + Express', value: 'Node.js + Express' },
            { name: 'Node.js + Fastify', value: 'Node.js + Fastify' },
            { name: 'AWS Lambda', value: 'AWS Lambda' },
            { name: 'Python + FastAPI', value: 'Python + FastAPI' },
            { name: 'Python + Django', value: 'Python + Django' },
            { name: 'Rust + Axum', value: 'Rust + Axum' },
            { name: 'その他', value: 'Other' },
          ],
        });

        database = await select({
          message: 'データベースを選択してください:',
          choices: [
            { name: 'PostgreSQL', value: 'PostgreSQL' },
            { name: 'MySQL', value: 'MySQL' },
            { name: 'MongoDB', value: 'MongoDB' },
            { name: 'DynamoDB', value: 'DynamoDB' },
            { name: 'SQLite', value: 'SQLite' },
            { name: 'その他', value: 'Other' },
          ],
        });
      }

      const infrastructure = await select({
        message: 'インフラストラクチャプラットフォームを選択してください:',
        choices: [
          { name: 'AWS', value: 'AWS' },
          { name: 'Google Cloud Platform', value: 'GCP' },
          { name: 'Microsoft Azure', value: 'Azure' },
          { name: 'Vercel', value: 'Vercel' },
          { name: 'Netlify', value: 'Netlify' },
          { name: 'その他', value: 'Other' },
        ],
      });

      const deployment = await select({
        message: 'デプロイ方法を選択してください:',
        choices: [
          { name: 'GitHub Actions', value: 'GitHub Actions' },
          { name: 'GitLab CI', value: 'GitLab CI' },
          { name: 'Jenkins', value: 'Jenkins' },
          { name: 'Docker', value: 'Docker' },
          { name: 'その他', value: 'Other' },
        ],
      });

      const monitoring = await select({
        message: '監視ソリューションを選択してください:',
        choices: [
          { name: 'Sentry', value: 'Sentry' },
          { name: 'DataDog', value: 'DataDog' },
          { name: 'New Relic', value: 'New Relic' },
          { name: 'CloudWatch', value: 'CloudWatch' },
          { name: 'その他', value: 'Other' },
        ],
      });

      return {
        projectType,
        frontend,
        cliLanguage,
        backend,
        database,
        infrastructure,
        deployment,
        monitoring,
      };
    } catch (error) {
      console.error(chalk.red('技術スタック選択エラー:'), error);
      throw error;
    }
  }

  private async validateConfiguration(config: ProjectConfig): Promise<void> {
    const spinner = ora('設定を検証中...').start();

    try {
      // 基本的な検証
      if (!config.projectName || !config.description || !config.repositoryUrl) {
        throw new Error('必須項目が不足しています');
      }

      // ターゲットパスの検証
      const targetPath = path.isAbsolute(config.targetPath || '')
        ? config.targetPath!
        : path.resolve(this.sourceDir, config.targetPath || `../${config.projectName}`);
      if (await fs.pathExists(targetPath)) {
        try {
          const overwrite = await confirm({
            message: `ディレクトリ "${targetPath}" は既に存在します。上書きしますか？`,
            default: false,
          });

          if (!overwrite) {
            throw new Error('ユーザーによってキャンセルされました');
          }

          await fs.remove(targetPath);
        } catch (error) {
          if (error instanceof Error && error.message === 'ユーザーによってキャンセルされました') {
            throw error;
          }
          console.error(chalk.red('ディレクトリ上書き確認中にエラーが発生しました:'), error);
          throw new Error('ディレクトリ上書き確認プロセスが中断されました');
        }
      }

      spinner.succeed('設定は有効です');
    } catch (error) {
      spinner.fail('設定の検証に失敗しました');
      throw error;
    }
  }

  private async showSummaryAndConfirm(config: ProjectConfig): Promise<void> {
    console.log(chalk.blue('\n📋 設定サマリー\n'));

    console.log(chalk.white('プロジェクト:'));
    console.log(chalk.gray(`  名前: ${config.projectName}`));
    console.log(chalk.gray(`  説明: ${config.description}`));
    console.log(chalk.gray(`  リポジトリ: ${config.repositoryUrl}`));
    const displayTargetPath = path.isAbsolute(config.targetPath || '')
      ? config.targetPath!
      : path.resolve(this.sourceDir, config.targetPath || `../${config.projectName}`);
    console.log(chalk.gray(`  作成先: ${displayTargetPath}`));

    console.log(chalk.white('\n開発アプローチ:'));
    console.log(chalk.gray(`  プロンプト: ${config.prompt}`));
    console.log(chalk.gray(`  チームサイズ: ${config.team.size}`));
    console.log(chalk.gray(`  業界: ${config.team.industry}`));

    console.log(chalk.white('\n技術スタック:'));
    console.log(chalk.gray(`  フロントエンド: ${config.techStack.frontend}`));
    console.log(chalk.gray(`  バックエンド: ${config.techStack.backend}`));
    console.log(chalk.gray(`  データベース: ${config.techStack.database}`));
    console.log(chalk.gray(`  インフラ: ${config.techStack.infrastructure}`));

    try {
      const confirmResult = await confirm({
        message: 'この設定でプロジェクトを作成しますか？',
        default: true,
      });

      if (!confirmResult) {
        throw new Error('ユーザーによってキャンセルされました');
      }
    } catch (error) {
      if (error instanceof Error && error.message === 'ユーザーによってキャンセルされました') {
        throw error;
      }
      console.error(chalk.red('設定確認中にエラーが発生しました:'), error);
      throw new Error('設定確認プロセスが中断されました');
    }
  }

  private async createNewProject(config: ProjectConfig): Promise<void> {
    const spinner = ora('新しいプロジェクトを作成中...').start();
    // 相対パスの場合はsourceDirからの相対パスとして解決
    const targetPath = path.isAbsolute(config.targetPath || '')
      ? config.targetPath!
      : path.resolve(this.sourceDir, config.targetPath || `../${config.projectName}`);
    this.targetDir = targetPath;

    try {
      // ターゲットディレクトリを作成
      await fs.ensureDir(targetPath);

      // コピーするファイルとディレクトリのリスト
      const copyItems = [
        'README.md',
        'package.json',
        'package-lock.json',
        '.gitignore',
        'CLAUDE.md',
        'CONTRIBUTING.md',
        'CUSTOMIZATION_GUIDE.md',
        'DEVELOPMENT_ROADMAP.md',
        'FEATURE_SUMMARY.md',
        'PROGRESS.md',
        'PROJECT_STRUCTURE.md',
        'docs',
        'prompts',
        'scripts',
        'infrastructure',
        '.github',
        'decisions',
      ];

      // ファイルとディレクトリをコピー
      for (const item of copyItems) {
        const sourcePath = path.join(this.sourceDir, item);
        const targetItemPath = path.join(targetPath, item);

        if (await fs.pathExists(sourcePath)) {
          await fs.copy(sourcePath, targetItemPath);
        }
      }

      // プロジェクト設定ファイルを作成
      await this.createProjectConfig(targetPath, config);

      // テンプレートファイルを処理
      await this.processTemplates(targetPath, config);

      // Rust CLIテンプレートを処理
      if (config.techStack.projectType === 'cli-tool' && config.techStack.cliLanguage === 'Rust') {
        await this.processRustCliTemplate(targetPath, config);
      }

      // 選択されたプロンプトファイルをコピー
      await this.copyPromptFile(targetPath, config.prompt);

      // .cursorrules を生成
      await this.generateCursorRules(targetPath, config);

      // package.json を更新
      await this.updatePackageJson(targetPath, config);

      // 不要なファイルを削除
      await this.cleanupFiles(targetPath);

      spinner.succeed('新しいプロジェクトの作成が完了しました');
    } catch (error) {
      spinner.fail('プロジェクトの作成に失敗しました');
      throw error;
    }
  }

  private async createProjectConfig(targetPath: string, config: ProjectConfig): Promise<void> {
    const configDir = path.join(targetPath, '.claude');
    const configPath = path.join(configDir, 'project-config.json');

    await fs.ensureDir(configDir);
    await fs.writeFile(configPath, JSON.stringify(config, null, 2));
  }

  private async processTemplates(targetPath: string, config: ProjectConfig): Promise<void> {
    const templateProcessor = new TemplateProcessor(targetPath);
    await templateProcessor.processAllTemplates(config);
  }

  private async copyPromptFile(targetPath: string, promptType: string): Promise<void> {
    const sourceFile = path.join(this.sourceDir, 'prompts', `${promptType}.md`);
    const targetFile = path.join(targetPath, 'PROMPT.md');

    if (await fs.pathExists(sourceFile)) {
      await fs.copy(sourceFile, targetFile);
    }
  }

  private async processRustCliTemplate(targetPath: string, config: ProjectConfig): Promise<void> {
    const rustTemplateDir = path.join(this.sourceDir, 'templates', 'rust-cli');
    
    if (!(await fs.pathExists(rustTemplateDir))) {
      console.warn('Rust CLI template directory not found, skipping...');
      return;
    }

    // Rust CLIテンプレートファイルをコピーして処理
    await this.copyRustCliTemplate(rustTemplateDir, targetPath, config);
  }

  private async copyRustCliTemplate(templateDir: string, targetPath: string, config: ProjectConfig): Promise<void> {
    const templateFiles = await this.getAllTemplateFiles(templateDir);
    
    for (const templateFile of templateFiles) {
      const relativePath = path.relative(templateDir, templateFile);
      const targetFile = path.join(targetPath, relativePath.replace('.template', ''));
      
      // ディレクトリを作成
      await fs.ensureDir(path.dirname(targetFile));
      
      // テンプレートファイルを読み込んで処理
      const content = await fs.readFile(templateFile, 'utf-8');
      const processedContent = this.processTemplateContent(content, config);
      
      await fs.writeFile(targetFile, processedContent);
    }
  }

  private async getAllTemplateFiles(dir: string): Promise<string[]> {
    const files: string[] = [];
    const items = await fs.readdir(dir);
    
    for (const item of items) {
      const fullPath = path.join(dir, item);
      const stat = await fs.stat(fullPath);
      
      if (stat.isDirectory()) {
        const subFiles = await this.getAllTemplateFiles(fullPath);
        files.push(...subFiles);
      } else if (item.endsWith('.template')) {
        files.push(fullPath);
      }
    }
    
    return files;
  }

  private processTemplateContent(content: string, config: ProjectConfig): string {
    const sanitizedProjectName = config.projectName.toLowerCase().replace(/[^a-z0-9_]/g, '_');
    const pascalProjectName = config.projectName.replace(/[^a-zA-Z0-9]/g, '').replace(/^./, (c) => c.toUpperCase());
    
    return content
      .replace(/\{\{projectName\}\}/g, sanitizedProjectName)
      .replace(/\{\{projectName\|pascal\}\}/g, pascalProjectName)
      .replace(/\{\{description\}\}/g, config.description)
      .replace(/\{\{repositoryUrl\}\}/g, config.repositoryUrl);
  }

  private async generateCursorRules(targetPath: string, config: ProjectConfig): Promise<void> {
    const cursorRulesContent = `# Cursor Rules - 日本語コミュニケーション設定

## 会話ガイドライン
- 常に日本語で会話する

## 開発哲学

### テスト駆動開発（TDD）
- 原則としてテスト駆動開発（TDD）で進める
- 期待される入出力に基づき、まずテストを作成する
- 実装コードは書かず、テストのみを用意する
- テストを実行し、失敗を確認する
- テストが正しいことを確認できた段階でコミットする
- その後、テストをパスさせる実装を進める
- 実装中はテストを変更せず、コードを修正し続ける
- すべてのテストが通過するまで繰り返す

### Git Worktree 管理
- 新機能開発やバグ修正は原則としてworktreeでブランチを切ってから開始する
- メインブランチ（main）での直接開発は避ける
- worktree作成手順：
  1. \`git worktree add ../feature/機能名 機能名\`
  2. 開発作業を実施
  3. 完了後、\`git worktree remove ../feature/機能名\` で削除
- 複数の機能を並行開発する場合は、それぞれ別のworktreeを使用する

## 言語設定
- 常に日本語でコミュニケーションを行ってください
- コードコメントも日本語で記述してください
- エラーメッセージやログの説明も日本語で行ってください

## コーディングスタイル
- 変数名や関数名は英語で記述（プログラミングの慣例に従う）
- コメント、ドキュメント、READMEは日本語で記述
- コミットメッセージは日本語で記述

## コミュニケーション
- 技術的な説明は分かりやすい日本語で行ってください
- 専門用語を使用する場合は、必要に応じて説明を加えてください
- 質問や確認は日本語で行ってください

## プロジェクト固有の設定
- このプロジェクトは ${config.projectName} です
- 開発環境のセットアップや設定に関する質問は日本語で対応してください
- ドキュメントの作成や更新も日本語で行ってください

## ファイル命名規則
- 設定ファイルやドキュメントファイルは日本語名も可
- ソースコードファイルは英語名で統一
- ディレクトリ名は英語で統一

## エラーハンドリング
- エラーメッセージの説明は日本語で行ってください
- デバッグ情報も日本語で提供してください
- トラブルシューティングの手順も日本語で説明してください
`;

    await fs.writeFile(path.join(targetPath, '.cursorrules'), cursorRulesContent);
  }

  private async updatePackageJson(targetPath: string, config: ProjectConfig): Promise<void> {
    const packageJsonPath = path.join(targetPath, 'package.json');
    const packageJson = await fs.readJson(packageJsonPath);

    // プロジェクト名を更新
    packageJson.name = config.projectName.toLowerCase().replace(/\s+/g, '-');
    packageJson.description = `${config.projectName} - ${config.description}`;

    // リポジトリ情報をクリア
    delete packageJson.repository;
    delete packageJson.bugs;
    delete packageJson.homepage;

    await fs.writeJson(packageJsonPath, packageJson, { spaces: 2 });
  }

  private async cleanupFiles(targetPath: string): Promise<void> {
    // .git ディレクトリを削除（新しいリポジトリとして初期化するため）
    const gitPath = path.join(targetPath, '.git');
    if (await fs.pathExists(gitPath)) {
      await fs.remove(gitPath);
    }

    // node_modules を削除（新しくインストールするため）
    const nodeModulesPath = path.join(targetPath, 'node_modules');
    if (await fs.pathExists(nodeModulesPath)) {
      await fs.remove(nodeModulesPath);
    }

    // バックアップディレクトリを削除
    const backupPath = path.join(targetPath, '.backups');
    if (await fs.pathExists(backupPath)) {
      await fs.remove(backupPath);
    }
  }

  private showCompletionMessage(): void {
    console.log(chalk.green.bold('\n✅ 新しいプロジェクトの作成が完了しました！'));
    
    console.log(chalk.cyan.bold('\n📋 次のステップ:'));
    console.log(chalk.white(`1. プロジェクトディレクトリに移動:`));
    console.log(chalk.gray(`   cd ${this.targetDir || '新しいプロジェクトディレクトリ'}`));
    console.log(chalk.white(`2. 依存関係をインストール:`));
    console.log(chalk.gray(`   npm install`));
    console.log(chalk.white(`3. Git リポジトリを初期化:`));
    console.log(chalk.gray(`   git init`));
    console.log(chalk.gray(`   git add .`));
    console.log(chalk.gray(`   git commit -m "Initial commit"`));
    console.log(chalk.white(`4. 開発を開始:`));
    console.log(chalk.gray(`   npm run setup`));
    console.log(chalk.cyan.bold('\n🎉 新しいプロジェクトの準備が完了しました！'));
  }
}

// メイン実行
if (require.main === module) {
  const assistant = new SetupAssistant();
  assistant.run().catch(console.error);
}

export default SetupAssistant;
