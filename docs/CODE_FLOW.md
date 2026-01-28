# 게임 진행 코드 흐름

이 문서는 Chesstack 게임의 코드 흐름과 주요 컴포넌트 간의 상호작용을 설명합니다.

---

## 아키텍처 개요

```
┌─────────────────────────────────────────────────────────────┐
│                        Web UI (index.html)                  │
│                     JavaScript + HTML/CSS                   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    WASM 바인딩 (wasm/src/lib.rs)            │
│                 Game 구조체 + wasm-bindgen                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    게임 엔진 (engine/src/lib.rs)            │
│              GameState, Piece, Action, LegalMove            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              행마법 인터프리터 (chessembly/src/lib.rs)      │
│           Interpreter, Token, Activation, BoardState        │
└─────────────────────────────────────────────────────────────┘
```

---

## 크레이트 의존성

```
chesstack-wasm
    └── engine
            └── chessembly
```

---

## 게임 초기화 흐름

### 1. WASM 로드 (JavaScript)

```javascript
// index.html
import init, { Game } from './pkg/chesstack_wasm.js';

await init();              // WASM 모듈 초기화
game = new Game();         // 새 게임 생성
game.setup_initial();      // 초기 포지션 설정
render();                  // UI 렌더링
```

### 2. Game 생성 (WASM)

```rust
// wasm/src/lib.rs
#[wasm_bindgen(constructor)]
pub fn new() -> Game {
    Game {
        state: GameState::new_default(),  // ← engine 호출
    }
}
```

### 3. GameState 초기화 (Engine)

```rust
// engine/src/lib.rs
pub fn new(starting_player: PlayerId) -> Self {
    let mut state = Self {
        board: HashMap::new(),
        pockets: HashMap::new(),
        pieces: HashMap::new(),
        turn: starting_player,
        // ...
    };
    
    // 킹 배치 (e1, e8)
    state.setup_initial_kings();
    state
}

pub fn setup_initial_position(&mut self) {
    // 포켓에 기물 추가 (퀸, 룩x2, 비숍x2, 나이트x2, 폰x8)
    let white_pocket = vec![
        PieceSpec::new(PieceKind::Queen),
        PieceSpec::new(PieceKind::Rook),
        // ...
    ];
    self.setup_pocket(0, white_pocket);
    // 흑도 동일
}
```

---

## 기물 선택 및 이동 가능 칸 계산

### 1. 사용자 클릭 (JavaScript)

```javascript
function onSquareClick(x, y) {
    // 자신의 기물 클릭 시
    if (piece && piece.owner === state.current_player) {
        selectedSquare = { x, y };
        legalMoves = game.get_legal_moves(x, y);  // ← WASM 호출
    }
}
```

### 2. WASM 전달

```rust
// wasm/src/lib.rs
#[wasm_bindgen]
pub fn get_legal_moves(&self, x: i32, y: i32) -> JsValue {
    let square = Square::new(x, y);
    let moves = self.state.get_legal_moves_at(square);  // ← engine 호출
    
    // JsMove 배열로 변환하여 반환
    serde_wasm_bindgen::to_value(&js_moves).unwrap()
}
```

### 3. Engine에서 Chessembly 실행

```rust
// engine/src/lib.rs
pub fn get_legal_moves_at(&self, square: Square) -> Vec<LegalMove> {
    let piece_id = self.board.get(&square)?;
    self.get_legal_moves(piece_id)
}

pub fn get_legal_moves(&self, piece_id: &PieceId) -> Vec<LegalMove> {
    let piece = self.pieces.get(piece_id)?;
    
    // 이동 불가 상태 확인 (스턴, 이동 스택)
    if !piece.can_move() {
        return vec![];
    }
    
    // Chessembly 보드 상태 생성
    let mut board = self.to_chessembly_board(piece_id)?;
    
    // 행마법 스크립트 가져오기
    let script = piece.effective_kind().chessembly_script(piece.is_white());
    
    // Chessembly 인터프리터 실행
    let mut interpreter = Interpreter::new();
    interpreter.parse(script);
    let activations = interpreter.execute(&mut board);  // ← chessembly 호출
    
    // Activation → LegalMove 변환
    activations.iter().map(|a| LegalMove {
        from: pos,
        to: Square::new(pos.x + a.dx, pos.y + a.dy),
        move_type: a.move_type,
        is_capture: self.board.contains_key(&target),
        tags: a.tags.clone(),
    }).collect()
}
```

### 4. Chessembly 인터프리터 실행

```rust
// chessembly/src/lib.rs
pub fn execute(&self, board: &mut BoardState) -> Vec<Activation> {
    let mut activations = Vec::new();
    let mut pc = 0;           // 프로그램 카운터
    let mut anchor_x = 0;     // 앵커 X (누적 오프셋)
    let mut anchor_y = 0;     // 앵커 Y
    let mut last_value = true;
    
    while pc < self.tokens.len() {
        let token = &self.tokens[pc];
        pc += 1;
        
        // 체인 종료 조건 확인
        if should_terminate {
            // 세미콜론까지 스킵, 앵커 초기화
        }
        
        match token {
            Token::TakeMove(dx, dy) => {
                let target_x = board.piece_x + anchor_x + dx;
                let target_y = board.piece_y + anchor_y + dy;
                
                if board.in_bounds(target_x, target_y) 
                   && !board.has_friendly(target_x, target_y) {
                    activations.push(Activation {
                        dx: anchor_x + dx,
                        dy: anchor_y + dy,
                        move_type: MoveType::TakeMove,
                        tags: pending_tags.clone(),
                    });
                    anchor_x += dx;
                    anchor_y += dy;
                    last_value = !board.has_enemy(target_x, target_y);
                } else {
                    last_value = false;
                }
            }
            
            Token::Repeat(n) => {
                if last_value && *n > 0 {
                    pc = pc - *n - 1;  // n개 토큰 뒤로 점프
                }
            }
            
            Token::Semicolon => {
                anchor_x = 0;
                anchor_y = 0;
                last_value = true;
            }
            
            // ... 다른 토큰들 ...
        }
    }
    
    activations
}
```

---

## 기물 이동 흐름

### 1. 목표 칸 클릭 (JavaScript)

```javascript
function onSquareClick(x, y) {
    if (selectedSquare && legalMoves.some(m => m.to_x === x && m.to_y === y)) {
        const success = game.move_piece(selectedSquare.x, selectedSquare.y, x, y);
        if (success) {
            render();
            checkGameOver();
        }
    }
}
```

### 2. WASM 처리

```rust
// wasm/src/lib.rs
#[wasm_bindgen]
pub fn move_piece(&mut self, from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> bool {
    let from = Square::new(from_x, from_y);
    let to = Square::new(to_x, to_y);
    
    if self.state.is_valid_move_at(from, to) {
        if let Some(piece) = self.state.get_piece_at(from) {
            let action = Action::Move { 
                piece_id: piece.id.clone(), 
                from, 
                to,
            };
            self.state.apply_action(action);
            return true;
        }
    }
    false
}
```

### 3. Engine 액션 적용

```rust
// engine/src/lib.rs
pub fn apply_action(&mut self, action: Action) {
    match action {
        Action::Move { piece_id, from, to } => {
            self.move_piece(self.turn, &piece_id, from, to);
        }
        // ...
    }
}

pub fn move_piece(&mut self, player: PlayerId, piece_id: &PieceId, from: Square, to: Square) 
    -> Result<Option<PieceId>, String> 
{
    // 유효성 검사
    // ...
    
    // 잡기 처리
    let captured = if let Some(victim_id) = self.board.get(&to) {
        self.capture_piece(&piece_id, victim_id)?
    } else {
        None
    };
    
    // 기물 이동
    self.board.remove(&from);
    self.board.insert(to, piece_id.clone());
    
    if let Some(piece) = self.pieces.get_mut(piece_id) {
        piece.pos = Some(to);
        piece.move_stack -= 1;  // 이동 스택 감소
    }
    
    // 활성 기물 설정 (추가 이동 가능)
    self.active_piece = Some(piece_id.clone());
    
    Ok(captured)
}
```

---

## 턴 종료 흐름

### 1. 턴 종료 버튼 (JavaScript)

```javascript
window.endTurn = function() {
    game.end_turn();
    render();
};
```

### 2. Engine 턴 처리

```rust
// engine/src/lib.rs
pub fn end_turn(&mut self) {
    // 1. 모든 기물 스턴 1 감소
    for piece in self.pieces.values_mut() {
        piece.stun = (piece.stun - 1).max(0);
    }
    
    // 2. 다음 플레이어로 전환
    self.turn = 1 - self.turn;
    
    // 3. 다음 턴 기물들 이동 스택 초기화
    for piece in self.pieces.values_mut() {
        if piece.owner == self.turn && piece.pos.is_some() {
            piece.move_stack = Self::initial_move_stack(piece.score());
        }
    }
    
    // 4. 턴 상태 초기화
    self.active_piece = None;
    self.action_taken = false;
}
```

---

## 승리 조건 확인

```rust
// engine/src/lib.rs
pub fn check_victory(&self) -> GameResult {
    let mut white_has_royal = false;
    let mut black_has_royal = false;
    
    for piece in self.pieces.values() {
        if piece.is_royal {
            if piece.owner == 0 {
                white_has_royal = true;
            } else {
                black_has_royal = true;
            }
        }
    }
    
    if !white_has_royal {
        GameResult::BlackWins
    } else if !black_has_royal {
        GameResult::WhiteWins
    } else {
        GameResult::Ongoing
    }
}
```

---

## 주요 데이터 구조

### GameState (Engine)

```rust
pub struct GameState {
    pub board: HashMap<Square, PieceId>,      // 보드 상태
    pub pockets: HashMap<PlayerId, Vec<PieceSpec>>,  // 포켓
    pub pieces: HashMap<PieceId, Piece>,      // 모든 기물
    pub turn: PlayerId,                        // 현재 턴
    pub global_state: HashMap<String, i32>,   // 전역 상태
    pub active_piece: Option<PieceId>,        // 활성 기물
    pub action_taken: bool,                    // 행동 여부
    next_piece_id: u32,                        // 다음 기물 ID
}
```

### Piece (Engine)

```rust
pub struct Piece {
    pub id: PieceId,
    pub kind: PieceKind,
    pub owner: PlayerId,
    pub pos: Option<Square>,
    pub stun: i32,              // 스턴 스택
    pub move_stack: i32,        // 이동 스택
    pub is_royal: bool,         // 로열 여부
    pub disguise: Option<PieceKind>,  // 변장
}
```

### BoardState (Chessembly)

```rust
pub struct BoardState {
    pub board_width: i32,
    pub board_height: i32,
    pub piece_x: i32,           // 현재 기물 X
    pub piece_y: i32,           // 현재 기물 Y
    pub piece_name: String,
    pub is_white: bool,
    pub pieces: HashMap<(i32, i32), (String, bool)>,  // 위치 → (기물명, 색상)
    pub state: HashMap<String, i32>,
    pub danger_squares: HashSet<(i32, i32)>,
    pub in_check: bool,
}
```

### Activation (Chessembly)

```rust
pub struct Activation {
    pub dx: i32,                // 기물 위치 기준 X 오프셋
    pub dy: i32,                // 기물 위치 기준 Y 오프셋
    pub move_type: MoveType,    // TakeMove, Move, Take 등
    pub tags: Vec<ActionTag>,   // 추가 태그 (Transition 등)
}
```

---

## 시퀀스 다이어그램

### 기물 이동

```
User        JavaScript       WASM/Game       Engine/GameState      Chessembly
  │              │                │                  │                  │
  │──click(x,y)──►              │                  │                  │
  │              │──get_legal_moves(x,y)──────────►│                  │
  │              │                │                  │──execute()──────►│
  │              │                │                  │◄──activations────│
  │              │◄──────────JsMove[]───────────────│                  │
  │◄──render()───│                │                  │                  │
  │              │                │                  │                  │
  │──click(to)───►              │                  │                  │
  │              │──move_piece(from,to)───────────►│                  │
  │              │                │                  │──apply_action()──►
  │              │◄──────────success────────────────│                  │
  │◄──render()───│                │                  │                  │
```

---

## 디버깅 팁

### 1. Console 로그 확인

```javascript
console.log('Selected piece at:', x, y);
console.log('Legal moves:', legalMoves);
```

### 2. Chessembly 스크립트 테스트

```rust
#[test]
fn test_my_piece() {
    let mut interp = Interpreter::new();
    interp.parse("take-move(1, 2); take-move(2, 1);");
    
    let mut board = make_empty_board();
    let activations = interp.execute(&mut board);
    
    println!("Activations: {:?}", activations);
    assert_eq!(activations.len(), 2);
}
```

### 3. Engine 테스트

```rust
#[test]
fn test_piece_movement() {
    let mut state = GameState::new(0);
    state.setup_initial_position();
    
    let moves = state.get_legal_moves_at(Square::new(4, 0));
    println!("King moves: {:?}", moves);
}
```

---

## 파일 구조 요약

```
rust/
├── Cargo.toml              # 워크스페이스 설정
├── index.html              # 웹 UI
├── pkg/                    # WASM 빌드 출력
│   ├── chesstack_wasm.js
│   ├── chesstack_wasm_bg.wasm
│   └── ...
├── chessembly/
│   ├── Cargo.toml
│   └── src/lib.rs          # 행마법 인터프리터
├── engine/
│   ├── Cargo.toml
│   └── src/lib.rs          # 게임 엔진
└── wasm/
    ├── Cargo.toml
    └── src/lib.rs          # WASM 바인딩
```
