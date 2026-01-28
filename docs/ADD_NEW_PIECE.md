# 새 기물 추가 가이드

이 문서는 Chesstack 엔진에 새로운 기물을 추가하는 방법을 설명합니다.

## 개요

새 기물을 추가하려면 다음 3개 파일을 수정해야 합니다:

1. `rust/engine/src/lib.rs` - 기물 정의 및 점수
2. `rust/wasm/src/lib.rs` - WASM 바인딩 (선택)
3. `rust/index.html` - UI 표시 (선택)

---

## 1단계: PieceKind 열거형에 기물 추가

`engine/src/lib.rs`에서 `PieceKind` enum을 찾아 새 기물을 추가합니다:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PieceKind {
    Pawn,
    King,
    Queen,
    Rook,
    Knight,
    Bishop,
    // ... 기존 기물들 ...
    
    // 새 기물 추가
    MyNewPiece,  // ← 여기에 추가
    
    Custom(String),
}
```

---

## 2단계: 기물 점수 정의

같은 파일에서 `PieceKind::score()` 메서드를 찾아 점수를 추가합니다:

```rust
impl PieceKind {
    pub fn score(&self) -> i32 {
        match self {
            PieceKind::Pawn => 1,
            PieceKind::King => 4,
            // ... 기존 기물들 ...
            
            // 새 기물 점수 추가
            PieceKind::MyNewPiece => 5,  // ← 점수 설정
            
            PieceKind::Custom(_) => 3,
        }
    }
}
```

### 점수별 이동 스택 (stack.md 참조)

| 점수 | 이동 스택 |
|------|-----------|
| 1~2점 | 5 |
| 3~5점 | 3 |
| 6~7점 | 2 |
| 8점+ | 1 |

---

## 3단계: 행마법 스크립트 작성 (Chessembly)

`PieceKind::chessembly_script()` 메서드에서 기물의 이동 패턴을 정의합니다:

```rust
pub fn chessembly_script(&self, is_white: bool) -> &'static str {
    match self {
        // ... 기존 기물들 ...
        
        PieceKind::MyNewPiece => {
            // 예: 상하좌우 2칸 점프
            "take-move(2, 0); take-move(-2, 0); take-move(0, 2); take-move(0, -2);"
        }
        
        _ => ""
    }
}
```

### Chessembly 문법 요약

| 명령 | 설명 |
|------|------|
| `take-move(dx, dy)` | 이동 또는 잡기 가능 |
| `move(dx, dy)` | 이동만 가능 (잡기 불가) |
| `take(dx, dy)` | 잡기만 가능 (빈 칸 이동 불가) |
| `repeat(n)` | 앞의 n개 토큰을 실패할 때까지 반복 |
| `;` | 체인 구분자 (앵커 초기화) |
| `{ }` | 스코프 블록 (앵커 격리) |

### 행마법 예시

```
# 룩 (직선 슬라이드)
take-move(1, 0) repeat(1); take-move(-1, 0) repeat(1);
take-move(0, 1) repeat(1); take-move(0, -1) repeat(1);

# 나이트 (L자 점프)
take-move(1, 2); take-move(2, 1); take-move(2, -1); take-move(1, -2);
take-move(-1, 2); take-move(-2, 1); take-move(-2, -1); take-move(-1, -2);

# 비숍 (대각선 슬라이드)
take-move(1, 1) repeat(1); take-move(1, -1) repeat(1);
take-move(-1, 1) repeat(1); take-move(-1, -1) repeat(1);

# 와지르 (상하좌우 1칸)
take-move(1, 0); take-move(-1, 0); take-move(0, 1); take-move(0, -1);

# 페르츠 (대각선 1칸)
take-move(1, 1); take-move(1, -1); take-move(-1, 1); take-move(-1, -1);

# 다바바 (상하좌우 2칸 점프)
take-move(2, 0); take-move(-2, 0); take-move(0, 2); take-move(0, -2);

# 알필 (대각선 2칸 점프)
take-move(2, 2); take-move(2, -2); take-move(-2, 2); take-move(-2, -2);

# 카멜 (3-1 점프, 나이트의 확장)
take-move(1, 3); take-move(3, 1); take-move(3, -1); take-move(1, -3);
take-move(-1, 3); take-move(-3, 1); take-move(-3, -1); take-move(-1, -3);
```

---

## 4단계: WASM 바인딩 업데이트 (선택)

`wasm/src/lib.rs`에서 문자열 변환 함수들을 업데이트합니다:

```rust
fn kind_to_string(&self, kind: &PieceKind) -> String {
    match kind {
        // ... 기존 ...
        PieceKind::MyNewPiece => "mynewpiece".to_string(),
        // ...
    }
}

fn parse_piece_kind(&self, s: &str) -> PieceKind {
    match s.to_lowercase().as_str() {
        // ... 기존 ...
        "mynewpiece" => PieceKind::MyNewPiece,
        // ...
    }
}
```

---

## 5단계: UI 아이콘 추가 (선택)

`index.html`의 `pieceSymbols` 객체에 아이콘을 추가합니다:

```javascript
const pieceSymbols = {
    // ... 기존 ...
    'mynewpiece': { white: '🆕', black: '🆕' },
};
```

### 권장 아이콘 소스

- 유니코드 체스 기호: ♔ ♕ ♖ ♗ ♘ ♙ (U+2654~U+265F)
- 이모지: 🦁 🐎 🏰 등
- 커스텀 SVG

---

## 6단계: 빌드 및 테스트

```bash
# 테스트 실행
cd rust
cargo test

# WASM 빌드
cd wasm
wasm-pack build --target web --out-dir ../pkg

# 로컬 서버 실행
cd ..
python3 -m http.server 8080
```

---

## 예제: 카멜 기물 추가

카멜은 나이트의 확장판으로, (3,1) 또는 (1,3) 점프를 합니다.

### 1. PieceKind 추가
```rust
PieceKind::Camel,
```

### 2. 점수 설정
```rust
PieceKind::Camel => 3,
```

### 3. 행마법 작성
```rust
PieceKind::Camel => {
    "take-move(1, 3); take-move(3, 1); take-move(3, -1); take-move(1, -3);
     take-move(-1, 3); take-move(-3, 1); take-move(-3, -1); take-move(-1, -3);"
}
```

---

## 고급: 조건부 이동

특수한 조건이 필요한 경우 Chessembly의 조건식을 사용합니다:

```
# 폰의 초기 2칸 전진 (아직 움직이지 않은 경우만)
if-state("moved", false) move(0, 2);

# 적이 있을 때만 대각선 이동
observe(1, 1) enemy take(1, 1);

# 빈 칸일 때만 이동
observe(0, 1) peek move(0, 1);
```

자세한 내용은 [TUTORIAL.md](chessembly/TUTORIAL.md)를 참조하세요.

---

## 체크리스트

- [ ] `PieceKind` enum에 추가
- [ ] `score()` 메서드에 점수 추가
- [ ] `chessembly_script()` 메서드에 행마법 추가
- [ ] (선택) WASM 변환 함수 업데이트
- [ ] (선택) UI 아이콘 추가
- [ ] `cargo test` 통과 확인
- [ ] WASM 빌드 확인
