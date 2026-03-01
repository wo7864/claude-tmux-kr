# claude-tmux-kr

> **이 저장소는 [nielsgroen/claude-tmux](https://github.com/nielsgroen/claude-tmux)의 Fork입니다.**
>
> 원본 프로젝트를 기반으로 **한국어 번역 및 일부 커스터마이즈**를 목적으로 관리됩니다.
> 원본 저장소의 라이선스(AGPL-3.0)를 그대로 따릅니다.

tmux 내에서 여러 Claude Code 세션을 관리하기 위한 터미널 UI(TUI) 도구입니다. claude-tmux는 모든 Claude Code 인스턴스를 한눈에 보여주며, 빠른 세션 전환, 상태 모니터링, 세션 생명주기 관리(git worktree 및 Pull Request 지원 포함)를 제공합니다.

## 설치

### Cargo install

아래 명령어를 실행하세요:

```bash
cargo install claude-tmux
```

`~/.tmux.conf`에 다음 줄을 추가하세요:

```bash
bind-key v display-popup -E -w 80 -h 30 "~/.cargo/bin/claude-tmux"
```

### 소스에서 빌드

```bash
git clone https://github.com/wo7864/claude-tmux-kr.git
cd claude-tmux-kr
cargo build --release
```

`~/.tmux.conf`에 다음을 추가하여 키 바인딩을 설정하세요:

```bash
bind-key v display-popup -E -w 80 -h 30 "/path/to/claude-tmux"
```

### 사용 방법

tmux 설정을 다시 불러옵니다.
아무 tmux 세션에서 `Ctrl-b, v`를 누르면 claude-tmux가 열립니다.

Pull Request 기능을 사용하려면 `gh`(GitHub CLI)가 설치되어 있어야 합니다.

### tmux 옵션

옵션 설명:
- `-E` — claude-tmux 종료 시 팝업도 함께 닫힘
- `-w 80 -h 30` — 팝업 크기 (원하는 대로 조정 가능)

## 기능

- **세션 개요** — 모든 tmux 세션을 Claude Code 상태 표시와 함께 한눈에 확인
- **상태 감지** — 각 Claude Code 인스턴스가 유휴, 작업 중, 입력 대기 중인지 표시
- **빠른 전환** — 최소한의 키 입력으로 원하는 세션으로 이동
- **실시간 미리보기** — 선택한 세션의 Claude Code 패널 마지막 줄을 ANSI 색상과 함께 표시
- **세션 관리** — TUI를 벗어나지 않고 세션 생성, 종료, 이름 변경 가능
- **상세 정보 확장** — 윈도우 수, 패널 명령어, 가동 시간, 연결 상태 등 메타데이터 확인
- **퍼지 필터링** — 이름이나 경로로 세션을 빠르게 검색

## 스크린샷

스크린샷은 [GitHub](https://github.com/nielsgroen/claude-tmux)에서 확인할 수 있습니다.

<img src="docs/images/screenshot.png" alt="claude-tmux 스크린샷" width="400">

<img src="docs/images/screenshot2.png" alt="claude-tmux 스크린샷 2" width="400">

**상태 표시:**
- `●` — 작업 중: Claude가 처리 중
- `○` — 유휴: 입력 대기 상태
- `◐` — 입력 대기: 권한 확인 프롬프트 (`[y/n]`)
- `?` — 알 수 없음: Claude Code 세션이 아니거나 상태를 판별할 수 없음

## 키 바인딩

### 탐색

| 키 | 동작 |
|-----|--------|
| `j` / `↓` | 아래로 이동 |
| `k` / `↑` | 위로 이동 |
| `l` / `→` | 세션 상세 정보 펼치기 |
| `h` / `←` | 세션 상세 정보 접기 |
| `Enter` | 선택한 세션으로 전환 |

### 동작

| 키 | 동작 |
|-----|--------|
| `n` | 새 세션 생성 |
| `K` | 선택한 세션 종료 (확인 후) |
| `r` | 선택한 세션 이름 변경 |
| `/` | 이름/경로로 세션 필터링 |
| `Ctrl+c` | 필터 초기화 |
| `R` | 세션 목록 새로고침 |

### 기타

| 키 | 동작 |
|-----|--------|
| `?` | 도움말 표시 |
| `q` / `Esc` | 종료 |

## 상태 감지

claude-tmux는 패널 내용을 분석하여 Claude Code의 상태를 감지합니다:

| 패턴 | 상태 |
|---------|--------|
| 입력 프롬프트 (`❯`) + 상단 테두리 + "ctrl+c to interrupt" | 작업 중 |
| 입력 프롬프트 (`❯`) + 상단 테두리, interrupt 메시지 없음 | 유휴 |
| `[y/n]` 또는 `[Y/n]` 포함 | 입력 대기 |
| 그 외 | 알 수 없음 |

## 세션 모델

claude-tmux는 `claude` 명령어를 실행 중인 패널을 찾아 Claude Code가 포함된 세션을 식별합니다. 표시되는 작업 디렉토리와 미리보기는 Claude Code 패널이 있을 경우 해당 패널에서, 없을 경우 첫 번째 패널에서 가져옵니다.

세션은 연결된(attached) 세션을 먼저, 그 다음 이름순으로 정렬됩니다.

## 의존성

- [ratatui](https://ratatui.rs/) — 터미널 UI 프레임워크
- [crossterm](https://github.com/crossterm-rs/crossterm) — 터미널 조작
- [ansi-to-tui](https://github.com/uttarayan21/ansi-to-tui) — ANSI 이스케이프 시퀀스 렌더링
- [anyhow](https://github.com/dtolnay/anyhow) — 에러 처리
- [dirs](https://github.com/dirs-dev/dirs-rs) — 홈 디렉토리 경로 확인
- [unicode-width](https://github.com/unicode-rs/unicode-width) — 텍스트 정렬

## 프로젝트 구조

```
claude-tmux/
├── Cargo.toml
├── src/
│   ├── main.rs        # 진입점, 터미널 설정
│   ├── app.rs         # 애플리케이션 상태 머신
│   ├── ui.rs          # Ratatui 렌더링
│   ├── tmux.rs        # tmux 명령어 래퍼
│   ├── session.rs     # 세션/패널 데이터 구조
│   ├── detection.rs   # Claude Code 상태 감지
│   └── input.rs       # 키보드 이벤트 처리
└── README.md
```

## 원본과의 차이점

이 Fork는 원본 [nielsgroen/claude-tmux](https://github.com/nielsgroen/claude-tmux)에 다음과 같은 변경을 적용했습니다.

### tmux 바인드 키 변경

원본의 `Ctrl-b, Ctrl-c`에서 `Ctrl-b, v`로 변경했습니다. `~/.tmux.conf`의 `bind-key` 줄에서 원하는 키로 자유롭게 수정할 수 있습니다.

### UI 전체 한국어 번역

모든 사용자 대면 텍스트를 한국어로 번역했습니다:
- 상태 표시: `idle` → `대기`, `working` → `작업중`, `input` → `입력대기`, `unknown` → `알수없음`
- 액션 메뉴: `Switch to session` → `세션으로 전환`, `Kill session` → `세션 종료` 등
- 확인 다이얼로그, 도움말, 푸터 힌트, 에러/성공 메시지 등 전체
- 세션 상세 정보: `windows` → `윈도우`, `branch` → `브랜치`, `staged` → `스테이지` 등

### 레이아웃 변경 — 좌우 분할

원본의 상하 배치(세션 목록 위, 미리보기 아래)를 **좌우 분할**(세션 목록 30%, 미리보기 70%)로 변경했습니다.
미리보기 패널은 왼쪽 테두리와 제목(" 미리보기 ")이 표시됩니다.

### 세션 목록 2줄 표시

각 세션 항목을 2줄로 표시하도록 변경했습니다:
- **1줄**: 상태 아이콘 + 세션 이름 + 상태 라벨
- **2줄**: 작업 디렉토리 경로 + git 정보 (브랜치, 변경사항 등)

### 동적 미리보기 높이

미리보기 패널의 줄 수가 고정값(15줄)이 아니라 실제 렌더링 영역 높이에 맞춰 동적으로 조정됩니다.

### 새 세션 생성 개선

> **⚠️ 주의:** 기본 경로가 `~/projects/`로 하드코딩되어 있습니다. 본인의 프로젝트 디렉토리 구조가 다르다면 `src/app/mod.rs`의 `start_new_session()` 메서드에서 경로를 수정하세요.

- 기본 경로가 현재 디렉토리 대신 `~/projects/`로 설정
- 초기 포커스가 이름 필드가 아닌 경로 필드로 변경
- 세션 이름을 비워두면 경로의 마지막 폴더명으로 자동 생성 (예: `~/projects/my-app/` → `my-app`)

### Claude Code 실행 옵션

> **⚠️ 경고:** 새 세션에서 Claude Code를 시작할 때 `--dangerously-skip-permissions --teammate-mode tmux` 플래그가 자동으로 추가됩니다. 이 플래그는 **모든 권한 확인을 건너뛰므로**, Claude가 파일 수정, 명령어 실행 등을 확인 없이 수행합니다. 이 동작을 원하지 않는 경우 `src/tmux.rs`의 `new_session()` 메서드에서 해당 플래그를 제거하거나 수정하세요.
>
> ```rust
> // src/tmux.rs — 아래 줄에서 플래그를 조정하세요
> .args(["send-keys", "-t", name, "claude --dangerously-skip-permissions --teammate-mode tmux", "Enter"])
> ```

## 원본 저장소

- 원본: [nielsgroen/claude-tmux](https://github.com/nielsgroen/claude-tmux)
- 라이선스: [AGPL-3.0](./LICENSE)
