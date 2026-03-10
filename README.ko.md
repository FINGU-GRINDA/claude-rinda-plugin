# RINDA AI — B2B 수출 영업 자동화

> 해외 바이어 검색, 연락처 확보, 이메일 캠페인, 답변 관리, 성과 분석까지 — B2B 수출 영업을 자동화하는 Claude Code **플러그인 마켓플레이스**입니다.

---

## 빠른 시작

```bash
# 1. 마켓플레이스 설치
/plugin marketplace add FINGU-GRINDA/claude-rinda-plugin

# 2. RINDA 계정 연결
/rinda-ai:rinda 내 계정을 연결해줘

# 3. 바로 사용하기
/rinda-ai:rinda 미국에서 화장품 수입업체를 찾아줘
```

CLI 바이너리는 첫 세션에서 자동으로 설치됩니다.

---

## 주요 기능

자연어(한국어/영어)로 대화하듯 명령하면 됩니다. AI가 `rinda-cli` 도구를 사용하여 처리합니다.

| 워크플로우 | 예시 |
|-----------|------|
| **바이어 검색** | "미국에서 연매출 500만 달러 이상의 화장품 수입업체 50개를 찾아줘" |
| **연락처 확보** | "방금 찾은 바이어들의 연락처를 확보해줘" |
| **이메일 캠페인** | "'US Cosmetics Q1'이라는 6단계 영업 시퀀스를 만들어줘" |
| **답변 관리** | "오늘 온 답변을 확인하고 무엇을 해야 하는지 알려줘" |
| **성과 리포트** | "지난 30일간의 캠페인 성과를 보여줘" |

각 단계가 끝나면 AI가 다음 단계를 제안하고, 이전 결과(바이어 ID, 시퀀스 ID 등)를 자동으로 넘겨받아 진행합니다.

---

## 상세 기능

### 바이어 검색

산업, 국가, 바이어 유형, 최소 매출을 지정하면 AI가 검색 후 가중 평가(매출, 직원 수, 수입 이력, 제품 적합도)를 통해 순위를 매겨 제시합니다.

### 연락처 확보

이메일, 전화번호, LinkedIn 프로필, 취급 제품 정보를 자동 수집하고 우선순위를 분류합니다:

- **높음**: 구매 담당자 이메일 확보 + 제품 매칭
- **보통**: 이메일 확보, 직책 불명확
- **낮음**: 대표 이메일만 확보 (info@, contact@)

### 이메일 캠페인

AI가 여러 단계의 영업 이메일을 자동 생성합니다. 내장 가이드라인:

- 제목: 60자 이내, 회사명 포함
- 본문: Hook(니즈 언급) → Value(제안) → CTA(행동 유도)
- 팔로업: 매번 새로운 가치 제공, 점진적으로 짧게

### 답변 관리

답변을 의도와 긴급도별로 분류합니다:

| 의도 | 조치 |
|------|------|
| 미팅 요청 | 즉시 응답 |
| 긍정적 관심 | 당일 응답 |
| 질문 | 답변 + 3일 후 팔로업 |
| 지금은 아님 | 90일 후 재접근 |
| 거절 | 태그 업데이트, 시퀀스 제외 |
| 수신 거부 | 즉시 처리 |

### 캠페인 리포트

퍼널 분석, 이메일 성과, 핫 리드 목록, AI 인사이트를 제공합니다. B2B 수출 벤치마크와 자동 비교 (오픈율 35% 이상 양호, 답변율 10% 이상 양호).

---

## 아키텍처

이 저장소는 **플러그인 마켓플레이스** 구조로, `plugins/` 아래에 여러 플러그인을 호스팅할 수 있습니다.

```
.claude-plugin/
  marketplace.json          # 플러그인 레지스트리
  plugin.json               # 마켓플레이스 메타데이터
plugins/
  rinda-ai/                 # B2B 수출 영업 플러그인
    .claude-plugin/plugin.json
    hooks/hooks.json        # 세션 시작 시 CLI + MCP 서버 자동 설치
    skills/
      rinda/
        SKILL.md            # 메인 스킬 (CLI 명령어 + 워크플로우)
        references/
          buyer-qualification.md
          email-writing.md
          export-sales.md
bin/
  install.sh                # CLI + MCP 서버 설치 스크립트 (크로스 플랫폼)
crates/
  cli/                      # rinda-cli (Rust, 인증 + API)
  sdk/                      # OpenAPI 스펙에서 자동 생성
  mcp-server/               # rinda-mcp (Rust, stdio MCP 서버)
```

---

## MCP 서버 단독 사용 (플러그인 외부)

`rinda-mcp` 서버는 Claude Code 플러그인 없이도 MCP를 지원하는 모든 클라이언트(Claude Desktop, Cursor 등)에서 사용할 수 있습니다.

**사전 조건:** `rinda-cli auth login`을 한 번 실행하여 `~/.rinda/credentials.json`을 생성하세요. MCP 서버가 자동으로 이 파일에서 인증 정보를 읽습니다.

**설치:**

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/FINGU-GRINDA/claude-rinda-plugin/main/bin/install.sh)
```

**MCP 클라이언트 설정:**

Claude Desktop (macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "rinda": {
      "command": "/Users/사용자이름/.rinda/bin/rinda-mcp",
      "args": []
    }
  }
}
```

`/Users/사용자이름` 부분을 실제 홈 디렉토리 경로로 교체하세요. Linux에서는 `/home/사용자이름/.rinda/bin/rinda-mcp`를 사용합니다.

---

## FAQ

**프로그래밍 지식이 필요한가요?**
아닙니다. 자연어로 대화하듯 명령하면 됩니다.

**어떤 산업/국가를 지원하나요?**
B2B 수출이 가능한 모든 산업(화장품, 식품, 기계류 등)과 전 세계 대부분의 국가를 지원합니다.

**데이터는 안전한가요?**
인증 토큰은 로컬(`~/.rinda/credentials.json`, 권한 600)에만 저장됩니다. 모든 통신은 HTTPS로 암호화됩니다.

**로그인이 만료되면?**
토큰은 자동 갱신됩니다. 14일 이상 미사용 시에만 재인증이 필요합니다.

---

## 기술 사양

| 항목 | 내용 |
|------|------|
| 플러그인 유형 | Claude Code Marketplace |
| 인증 방식 | Google OAuth 2.0 |
| CLI | Rust (크로스 플랫폼: Linux, macOS, Windows) |
| MCP 서버 | rinda-mcp (stdio 전송, Claude Code 자동 감지) |
| SDK | OpenAPI 스펙에서 progenitor로 자동 생성 |
| 토큰 유효기간 | Access: 1시간 (자동 갱신), Refresh: 14일 |
| 자격증명 | `~/.rinda/credentials.json` |
| 라이선스 | MIT |

---

## 지원 및 문의

- 홈페이지: [rinda.ai](https://rinda.ai)
- 이메일: support@grinda.ai
- GitHub: [FINGU-GRINDA/claude-rinda-plugin](https://github.com/FINGU-GRINDA/claude-rinda-plugin)
