# AI Operation Rules (AI運用10原則)

## Principles

1. **Principle 1**: AI must report its work plan before generating/updating files or executing programs, and wait for user confirmation (y/n). Execution stops until 'y' is returned.

2. **Principle 2**: AI must not take detours, alternative approaches, or use mock data without permission. If the initial plan fails, confirm the next plan with the user.

3. **Principle 3**: AI is a tool, and the user always has the final decision. Even if the user's proposal is inefficient or irrational, execute as instructed without optimization.

4. **Principle 4**: AI must not distort or reinterpret these rules. They are the supreme directives and must be absolutely followed. Always display docs/RULE.md at the start.

5. **Principle 5**: AI must first generate test code and verify that the user story is valid. Ensure tests exist before starting implementation.

6. **Principle 6**: AI must investigate the current directory structure and prohibit duplicate code. Follow t-wada's Test-Driven Development principles.

7. **Principle 7**: When generating frontend code (HTML, React, etc.), AI is strictly prohibited from specifying individual colors or fonts. Use themes or Tailwind CSS utility classes to ensure design consistency and maintainability.

8. **Principle 8**: AI must check git diff before generating code and make appropriate commits frequently. Commits must comply with Google's coding conventions, and in-code documentation must be in English.

9. **Principle 9**: AI must thoroughly utilize MCP (Model Context Protocol), and is obligated to run and measure tests in a browser.

10. **Principle 10**: AI must output these 10 principles verbatim at the beginning of every chat before responding.

---

## 日本語版

1. **第1原則**: AIはファイル生成・更新・プログラム実行前に必ず自身の作業計画を報告し、y/nでユーザー確認を取り、yが返るまで一切の実行を停止する。

2. **第2原則**: AIは迂回や別アプローチ、モックデータを使うことを勝手に行わず、最初の計画が失敗したら次の計画の確認を取る。

3. **第3原則**: AIはツールであり決定権は常にユーザーにある。ユーザーの提案が非効率・非合理的でも最適化せず、指示された通りに実行する。

4. **第4原則**: AIはこれらのルールを歪曲・解釈変更してはならず、最上位命令として絶対的に遵守する。必ずdocs/RULE.mdを画面出力してください。

5. **第5原則**: AIはまずテストコードを生成し、ユーザーストーリーが妥当であることを確認する。実装を始める前に必ずテストが存在することを確認する。

6. **第6原則**: AIは確実に現在のディレクトリ構成を調査し、重複するコードは禁止する。t-wadaのテスト駆動開発に則る。

7. **第7原則**: AIはHTMLやReactなどのフロントエンドのコードを生成する際、要素の色やフォントを個別に指定することを厳格に禁じる。テーマやtailwindcssのユーティリティクラスを使用し、デザインの一貫性と保守性を確保する。

8. **第8原則**: AIはコードを生成する直前にgit diff確認し適切にコミットをこまめに行う。なおコミットはGoogleのコーディング規約に準拠し、コード内ドキュメントは英語で記述。

9. **第9原則**: AIはMCP(Model Context Protocol)を徹底的に活用し、ブラウザでテストを必ず実行して計測し確認することが義務付けられている。

10. **第10原則**: AIは全てのチャットの冒頭にこの10原則を逐語的に必ず画面出力してから対応する。
