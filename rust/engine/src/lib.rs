#![allow(dead_code)]

use std::collections::HashMap;
use std::collections::HashSet;

// Chessembly 인터프리터 사용
use chessembly::{Interpreter, BoardState as ChessemblyBoard};

// MoveType을 공개적으로 재export
pub use chessembly::MoveType;

pub type PlayerId = u8;
pub type PieceId = String;

/// 보드 좌표 (0-indexed: x=0~7, y=0~7)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square {
    pub x: i32,  // 0=a, 7=h
    pub y: i32,  // 0=1, 7=8 (백 기준 아래가 0)
}

impl Square {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    
    /// "e4" 같은 문자열에서 파싱
    pub fn from_notation(s: &str) -> Option<Self> {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() != 2 {
            return None;
        }
        let x = (chars[0] as i32) - ('a' as i32);
        let y = (chars[1] as i32) - ('1' as i32);
        if x >= 0 && x < 8 && y >= 0 && y < 8 {
            Some(Self { x, y })
        } else {
            None
        }
    }
    
    /// 체스 표기법으로 변환
    pub fn to_notation(&self) -> String {
        let file = (b'a' + self.x as u8) as char;
        let rank = (b'1' + self.y as u8) as char;
        format!("{}{}", file, rank)
    }
    
    pub fn is_valid(&self) -> bool {
        self.x >= 0 && self.x < 8 && self.y >= 0 && self.y < 8
    }
}

/// 기물 종류
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PieceKind {
    Pawn,
    King,
    Queen,
    Rook,
    Knight,
    Bishop,
    Amazon,
    Grasshopper,
    Knightrider,
    Archbishop,
    Dabbaba,
    Alfil,
    Ferz,
    Centaur,
    Camel,
    TempestRook,
    Cannon,
    Experiment,
    Custom(String),
}

impl PieceKind {
    /// 기물 점수 반환 (stack.md 기준)
    pub fn score(&self) -> i32 {
        match self {
            PieceKind::Pawn => 1,
            PieceKind::King => 4,
            PieceKind::Queen => 9,
            PieceKind::Rook => 5,
            PieceKind::Knight => 3,
            PieceKind::Bishop => 3,
            PieceKind::Amazon => 13,
            PieceKind::Grasshopper => 4,
            PieceKind::Knightrider => 7,
            PieceKind::Archbishop => 6,
            PieceKind::Dabbaba => 2,
            PieceKind::Alfil => 2,
            PieceKind::Ferz => 1,
            PieceKind::Centaur => 5,
            PieceKind::Camel => 3,
            PieceKind::TempestRook => 7,
            PieceKind::Cannon => 5,
            PieceKind::Experiment => 1, //실험용 기물.
            PieceKind::Custom(_) => 3, // 기본값
        }
    }
    
    /// 프로모션 가능 여부
    pub fn can_promote(&self) -> bool {
        matches!(self, PieceKind::Pawn)
    }
    
    /// 프로모션 가능한 기물 목록
    pub fn promotion_targets(&self) -> Vec<PieceKind> {
        match self {
            PieceKind::Pawn => vec![
                PieceKind::Queen,
                PieceKind::Rook,
                PieceKind::Bishop,
                PieceKind::Knight,
            ],
            _ => vec![],
        }
    }
    
    /// 프로모션 칸인지 (백: y=7, 흑: y=0)
    pub fn is_promotion_square(&self, square: Square, is_white: bool) -> bool {
        if !self.can_promote() {
            return false;
        }
        if is_white {
            square.y == 7
        } else {
            square.y == 0
        }
    }
    
    /// 프로모션 칸까지의 거리 (이동 스택 기준)
    pub fn distance_to_promotion(&self, square: Square, is_white: bool) -> i32 {
        if !self.can_promote() {
            return 0;
        }
        // 폰 기준: 직선 거리
        if is_white {
            7 - square.y
        } else {
            square.y
        }
    }
    
    /// 프로모션 기물의 최대 스턴 스택
    pub fn max_promotion_stun(&self) -> i32 {
        match self {
            PieceKind::Pawn => 8,
            _ => 0,
        }
    }
    
    /// 기물의 Chessembly 행마법 스크립트 반환
    pub fn chessembly_script(&self, is_white: bool) -> &'static str {
        // 백은 +y 방향이 전진, 흑은 -y 방향이 전진
        // 기본 스크립트는 백 기준으로 작성, 흑은 y 부호 반전 필요
        match self {
            PieceKind::Pawn => {
                if is_white {
                    // 백 폰: 앞으로 이동, 대각선 잡기
                    "move(0, 1); take(1, 1); take(-1, 1);"
                } else {
                    // 흑 폰
                    "move(0, -1); take(1, -1); take(-1, -1);"
                }
            }
            PieceKind::King => {
                // 킹: 모든 방향 1칸
                "take-move(1, 0); take-move(-1, 0); take-move(0, 1); take-move(0, -1);
                 take-move(1, 1); take-move(1, -1); take-move(-1, 1); take-move(-1, -1);"
            }
            PieceKind::Queen => {
                // 퀸: 룩 + 비숍
                "take-move(1, 0) repeat(1); take-move(-1, 0) repeat(1);
                 take-move(0, 1) repeat(1); take-move(0, -1) repeat(1);
                 take-move(1, 1) repeat(1); take-move(1, -1) repeat(1);
                 take-move(-1, 1) repeat(1); take-move(-1, -1) repeat(1);"
            }
            PieceKind::Rook => {
                // 룩: 직선 슬라이드
                "take-move(1, 0) repeat(1); take-move(-1, 0) repeat(1);
                 take-move(0, 1) repeat(1); take-move(0, -1) repeat(1);"
            }
            PieceKind::Knight => {
                // 나이트: L자 도약
                "take-move(1, 2); take-move(2, 1); take-move(2, -1); take-move(1, -2);
                 take-move(-1, 2); take-move(-2, 1); take-move(-2, -1); take-move(-1, -2);"
            }
            PieceKind::Bishop => {
                // 비숍: 대각선 슬라이드
                "take-move(1, 1) repeat(1); take-move(1, -1) repeat(1);
                 take-move(-1, 1) repeat(1); take-move(-1, -1) repeat(1);"
            }
            PieceKind::Amazon => {
                // 아마존: 퀸 + 나이트
                "take-move(1, 0) repeat(1); take-move(-1, 0) repeat(1);
                 take-move(0, 1) repeat(1); take-move(0, -1) repeat(1);
                 take-move(1, 1) repeat(1); take-move(1, -1) repeat(1);
                 take-move(-1, 1) repeat(1); take-move(-1, -1) repeat(1);
                 take-move(1, 2); take-move(2, 1); take-move(2, -1); take-move(1, -2);
                 take-move(-1, 2); take-move(-2, 1); take-move(-2, -1); take-move(-1, -2);"
            }
            PieceKind::Grasshopper => {
                // 그라스호퍼: 직선으로 기물 넘어서 바로 뒤에 착지
                // (간단히 구현 - 실제로는 더 복잡한 로직 필요)
                "do peek(1, 0) while take-move(1, 0);
                 do peek(-1, 0) while take-move(-1, 0);
                 do peek(0, 1) while take-move(0, 1);
                 do peek(0, -1) while take-move(0, -1);
                 do peek(1, 1) while take-move(1, 1);
                 do peek(1, -1) while take-move(1, -1);
                 do peek(-1, 1) while take-move(-1, 1);
                 do peek(-1, -1) while take-move(-1, -1);"
            }
            PieceKind::Knightrider => {
                // 나이트라이더: 나이트 방향으로 슬라이드
                "take-move(1, 2) repeat(1); take-move(2, 1) repeat(1);
                 take-move(2, -1) repeat(1); take-move(1, -2) repeat(1);
                 take-move(-1, 2) repeat(1); take-move(-2, 1) repeat(1);
                 take-move(-2, -1) repeat(1); take-move(-1, -2) repeat(1);"
            }
            PieceKind::Archbishop => {
                // 아크비숍(주교+기사): 비숍 + 나이트
                "take-move(1, 1) repeat(1); take-move(1, -1) repeat(1);
                 take-move(-1, 1) repeat(1); take-move(-1, -1) repeat(1);
                 take-move(1, 2); take-move(2, 1); take-move(2, -1); take-move(1, -2);
                 take-move(-1, 2); take-move(-2, 1); take-move(-2, -1); take-move(-1, -2);"
            }
            PieceKind::Dabbaba => {
                // 다바바: 직교 2칸 도약
                "take-move(2, 0); take-move(-2, 0); take-move(0, 2); take-move(0, -2);"
            }
            PieceKind::Alfil => {
                // 알필: 대각 2칸 도약
                "take-move(2, 2); take-move(2, -2); take-move(-2, 2); take-move(-2, -2);"
            }
            PieceKind::Ferz => {
                // 퍼즈: 대각 1칸
                "take-move(1, 1); take-move(1, -1); take-move(-1, 1); take-move(-1, -1);"
            }
            PieceKind::Centaur => {
                // 센타우르(킹+나이트)
                "take-move(1, 0); take-move(-1, 0); take-move(0, 1); take-move(0, -1);
                 take-move(1, 1); take-move(1, -1); take-move(-1, 1); take-move(-1, -1);
                 take-move(1, 2); take-move(2, 1); take-move(2, -1); take-move(1, -2);
                 take-move(-1, 2); take-move(-2, 1); take-move(-2, -1); take-move(-1, -2);"
            }
            PieceKind::Camel => {
                // 카멜: (3,1) 도약
                "take-move(3, 1); take-move(3, -1); take-move(-3, 1); take-move(-3, -1);
                 take-move(1, 3); take-move(1, -3); take-move(-1, 3); take-move(-1, -3);"
            }
            PieceKind::TempestRook => {
                // 템페스트 룩: 대각 1칸 후 십자 슬라이드
                "take-move(1, 1) { take-move(1, 0) repeat(1) } { take-move(0, 1) repeat(1) };
                 take-move(-1, 1) { take-move(-1, 0) repeat(1) } { take-move(0, 1) repeat(1) };
                 take-move(1, -1) { take-move(1, 0) repeat(1) } { take-move(0, -1) repeat(1) };
                 take-move(-1, -1) { take-move(-1, 0) repeat(1) } { take-move(0, -1) repeat(1) };"
            }
            PieceKind::Cannon => {
                "do take(1, 0) enemy(0, 0) not while jump(1, 0) repeat(1);
                 do take(-1, 0) enemy(0, 0) not while jump(-1, 0) repeat(1);
                 do take(0, 1) enemy(0, 0) not while jump(0, 1) repeat(1);
                 do take(0, -1) enemy(0, 0) not while jump(0, -1) repeat(1);
                 do peek(1, 0) while friendly(0, 0) move(1, 0) repeat(1);
                 do peek(-1, 0) while friendly(0, 0) move(-1, 0) repeat(1);
                 do peek(0, 1) while friendly(0, 0) move(0, 1) repeat(1);
                 do peek(0, -1) while friendly(0, 0) move(0, -1) repeat(1);"
            }
            PieceKind::Experiment => { //행마법(x, y)
                "
                 do 
                    take-move(1, -1) 
                 while 
                 peek(0, 0) 
                 edge-bottom(1, -1) jne(0) 
                    take-move(1, 1) repeat(1) 
                 label(0) 
                 edge-right(1, -1) jne(1) 
                    take-move(-1, -1) repeat(1) 
                 label(1);
                 "
            }
            PieceKind::Custom(_) => {
                // 커스텀 기물: 기본적으로 킹처럼
                "take-move(1, 0); take-move(-1, 0); take-move(0, 1); take-move(0, -1);
                 take-move(1, 1); take-move(1, -1); take-move(-1, 1); take-move(-1, -1);"
            }
        }
    }
}

/// 기물
#[derive(Debug, Clone)]
pub struct Piece {
    pub id: PieceId,
    pub kind: PieceKind,
    pub owner: PlayerId,
    pub pos: Option<Square>,    // None == 포켓에 있음
    pub stun: i32,              // 스턴 스택 (양수면 움직일 수 없음)
    pub move_stack: i32,        // 이동 스택 (한 턴에 이동 가능 횟수)
    pub is_royal: bool,         // 로얄 피스 여부
    pub disguise: Option<PieceKind>,  // 위장 (로얄 피스만)
}

impl Piece {
    pub fn new(id: PieceId, kind: PieceKind, owner: PlayerId) -> Self {
        Self {
            id,
            kind,
            owner,
            pos: None,
            stun: 0,
            move_stack: 0,
            is_royal: false,
            disguise: None,
        }
    }
    
    /// 실제 행마에 사용되는 기물 종류 (위장 고려)
    pub fn effective_kind(&self) -> &PieceKind {
        self.disguise.as_ref().unwrap_or(&self.kind)
    }
    
    /// 현재 기물 점수
    pub fn score(&self) -> i32 {
        self.kind.score()
    }
    
    /// 이동 가능 여부
    pub fn can_move(&self) -> bool {
        self.stun == 0 && self.move_stack > 0
    }
    
    /// 플레이어 색상 (백: 0, 흑: 1)
    pub fn is_white(&self) -> bool {
        self.owner == 0
    }
}

/// 플레이어가 수행할 수 있는 행동
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// 착수: 포켓에서 보드로 기물 배치
    Place {
        piece_id: PieceId,
        target: Square,
    },
    /// 이동: 기물 이동 (한 턴에 같은 기물 여러 번 가능)
    Move {
        piece_id: PieceId,
        from: Square,
        to: Square,
    },
    /// 위장: 로얄 피스를 다른 기물로 위장
    Disguise {
        piece_id: PieceId,
        as_kind: PieceKind,
    },
    /// 계승: 기물을 로얄 피스로 만듦
    Crown {
        piece_id: PieceId,
    },
    /// 스턴: 기물에 스턴 스택 부여 (아군 1~3, 적 1)
    Stun {
        piece_id: PieceId,
        amount: i32,
    },
}

/// 포켓에 있는 기물 스펙
#[derive(Debug, Clone)]
pub struct PieceSpec {
    pub kind: PieceKind,
}

impl PieceSpec {
    pub fn new(kind: PieceKind) -> Self {
        Self { kind }
    }
    
    pub fn score(&self) -> i32 {
        self.kind.score()
    }
}

/// 게임 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameResult {
    Ongoing,
    WhiteWins,
    BlackWins,
}

/// 유효한 이동 정보
#[derive(Debug, Clone)]
pub struct LegalMove {
    pub from: Square,
    pub to: Square,
    pub move_type: MoveType,
    pub is_capture: bool,
    pub tags: Vec<chessembly::ActionTag>,
    pub catch_to: Square,
}

/// 게임 상태
#[derive(Debug, Clone)]
pub struct GameState {
    pub board: HashMap<Square, PieceId>,
    pub pockets: HashMap<PlayerId, Vec<PieceSpec>>,
    pub pieces: HashMap<PieceId, Piece>,
    pub turn: PlayerId,
    pub global_state: HashMap<String, i32>,
    pub active_piece: Option<PieceId>,  // 현재 턴에 이동 중인 기물
    pub action_taken: bool,              // 이번 턴에 행동했는지 (이동 제외)
    pub debug_mode: bool,                // Chessembly 디버그 모드
    next_piece_id: u32,
}

/// 포켓 점수 제한
pub const MAX_POCKET_SCORE: i32 = 39;

impl GameState {
    pub fn new(starting_player: PlayerId) -> Self {
        let mut state = Self {
            board: HashMap::new(),
            pockets: HashMap::new(),
            pieces: HashMap::new(),
            turn: starting_player,
            global_state: HashMap::new(),
            active_piece: None,
            action_taken: false,
            debug_mode: false,
            next_piece_id: 0,
        };
        
        // 초기 킹 배치 (rule.md: e1(백), e8(흑))
        state.setup_initial_kings();
        state
    }
    
    fn setup_initial_kings(&mut self) {
        // 백 킹 (e1)
        let white_king = self.create_piece(PieceKind::King, 0);
        let white_king_id = white_king.id.clone();
        self.pieces.insert(white_king_id.clone(), white_king);
        self.place_king(&white_king_id, Square::new(4, 0)); // e1
        
        // 흑 킹 (e8)
        let black_king = self.create_piece(PieceKind::King, 1);
        let black_king_id = black_king.id.clone();
        self.pieces.insert(black_king_id.clone(), black_king);
        self.place_king(&black_king_id, Square::new(4, 7)); // e8
    }
    
    fn place_king(&mut self, piece_id: &PieceId, square: Square) {
        if let Some(piece) = self.pieces.get_mut(piece_id) {
            piece.pos = Some(square);
            piece.is_royal = true;
            // 킹 초기값: 스턴 0, 이동 3 (rule.md)
            piece.stun = 0;
            piece.move_stack = 3;
            self.board.insert(square, piece_id.clone());
        }
    }
    
    fn create_piece(&mut self, kind: PieceKind, owner: PlayerId) -> Piece {
        let id = format!("piece_{}", self.next_piece_id);
        self.next_piece_id += 1;
        Piece::new(id, kind, owner)
    }
    
    /// 포켓 초기화 (점수 합계 검증)
    pub fn setup_pocket(&mut self, player: PlayerId, specs: Vec<PieceSpec>) -> Result<(), String> {
        let total_score: i32 = specs.iter().map(|s| s.score()).sum();
        if total_score > MAX_POCKET_SCORE {
            return Err(format!(
                "포켓 점수 {}점이 제한 {}점을 초과합니다",
                total_score, MAX_POCKET_SCORE
            ));
        }
        self.pockets.insert(player, specs);
        Ok(())
    }

    /// 점수 제한 없이 포켓 설정 (실험용)
    pub fn setup_pocket_unchecked(&mut self, player: PlayerId, specs: Vec<PieceSpec>) {
        self.pockets.insert(player, specs);
    }
    
    /// 점수에 따른 이동 스택 계산 (stack.md)
    pub fn initial_move_stack(score: i32) -> i32 {
        match score {
            1..=2 => 5,
            3..=5 => 3,
            6..=7 => 2,
            _ if score >= 8 => 1,
            _ => 1,
        }
    }
    
    /// 착수 시 스턴 스택 계산
    fn calculate_placement_stun(&self, piece: &Piece, square: Square) -> i32 {
        let kind = &piece.kind;
        
        if kind.can_promote() {
            // 프로모션 가능 기물: 거리에 따라 스턴 조정
            let distance = kind.distance_to_promotion(square, piece.is_white());
            let max_stun = kind.max_promotion_stun();
            // 가까울수록 높은 스턴 (거리 0 = max, 거리 max = 0)
            let max_distance = 7; // 폰 기준
            max_stun - (max_stun * distance / max_distance)
        } else {
            // 일반 기물: 점수만큼 스턴
            piece.score()
        }
    }
    
    /// 착수 가능 여부 확인
    pub fn can_place(&self, player: PlayerId, kind: &PieceKind, target: Square) -> Result<(), String> {
        // 자신의 턴인지
        if self.turn != player {
            return Err("자신의 턴이 아닙니다".to_string());
        }
        
        // 이미 다른 행동을 했는지
        if self.action_taken {
            return Err("이번 턴에 이미 행동했습니다".to_string());
        }
        
        // 이동 중인 기물이 있는지
        if self.active_piece.is_some() {
            return Err("이동 중인 기물이 있습니다".to_string());
        }
        
        // 해당 칸이 비어있는지
        if self.board.contains_key(&target) {
            return Err("해당 칸에 이미 기물이 있습니다".to_string());
        }
        
        // 프로모션 기물은 프로모션 칸에 착수 불가
        let is_white = player == 0;
        if kind.is_promotion_square(target, is_white) {
            return Err("프로모션 기물은 프로모션 칸에 착수할 수 없습니다".to_string());
        }
        
        // 포켓에 해당 기물이 있는지
        let pocket = self.pockets.get(&player).ok_or("포켓이 없습니다")?;
        if !pocket.iter().any(|s| &s.kind == kind) {
            return Err("포켓에 해당 기물이 없습니다".to_string());
        }
        
        Ok(())
    }
    
    /// 착수 실행
    pub fn place_piece(&mut self, player: PlayerId, kind: PieceKind, target: Square) -> Result<PieceId, String> {
        self.can_place(player, &kind, target)?;
        
        // 포켓에서 기물 제거
        if let Some(pocket) = self.pockets.get_mut(&player) {
            if let Some(idx) = pocket.iter().position(|s| s.kind == kind) {
                pocket.remove(idx);
            }
        }
        
        // 기물 생성 및 배치
        let mut piece = self.create_piece(kind, player);
        let piece_id = piece.id.clone();
        
        // 스택 초기화
        piece.stun = self.calculate_placement_stun(&piece, target);
        piece.move_stack = Self::initial_move_stack(piece.score());
        piece.pos = Some(target);
        
        self.pieces.insert(piece_id.clone(), piece);
        self.board.insert(target, piece_id.clone());
        self.action_taken = true;
        
        Ok(piece_id)
    }
    
    /// 이동 가능 여부 확인
    pub fn can_move_piece(&self, player: PlayerId, piece_id: &PieceId, _from: Square, to: Square, move_type: MoveType) -> Result<(), String> {
        // 자신의 턴인지
        if self.turn != player {
            return Err("자신의 턴이 아닙니다".to_string());
        }
        
        // 다른 행동을 했는지 (이동은 예외)
        if self.action_taken {
            return Err("이번 턴에 이미 다른 행동을 했습니다".to_string());
        }
        
        // 이미 다른 기물이 이동 중인지
        if let Some(ref active) = self.active_piece {
            if active != piece_id {
                return Err("다른 기물이 이동 중입니다".to_string());
            }
        }
        
        // 기물 존재 확인
        let piece = self.pieces.get(piece_id).ok_or("기물을 찾을 수 없습니다")?;
        
        // 자신의 기물인지
        if piece.owner != player {
            return Err("자신의 기물이 아닙니다".to_string());
        }
        
        // 이동 가능한지 (스턴 0, 이동 스택 > 0)
        if !piece.can_move() {
            if piece.stun > 0 {
                return Err(format!("스턴 상태입니다 (스턴: {})", piece.stun));
            } else {
                return Err("이동 스택이 없습니다".to_string());
            }
        }
        
        // MoveType별 검증
        let is_target_empty = !self.board.contains_key(&to);
        let has_enemy = if let Some(target_piece_id) = self.board.get(&to) {
            if let Some(target_piece) = self.pieces.get(target_piece_id) {
                target_piece.owner != player
            } else {
                false
            }
        } else {
            false
        };
        let has_friendly = if let Some(target_piece_id) = self.board.get(&to) {
            if let Some(target_piece) = self.pieces.get(target_piece_id) {
                target_piece.owner == player
            } else {
                false
            }
        } else {
            false
        };
        
        match move_type {
            MoveType::Move => {
                // Move: 빈 칸으로만 이동 가능
                if !is_target_empty {
                    return Err("Move는 빈 칸으로만 이동할 수 있습니다".to_string());
                }
            }
            MoveType::Take => {
                // Take: 적이 있는 칸으로만 이동 가능
                if !has_enemy {
                    return Err("Take는 적이 있는 칸으로만 이동할 수 있습니다".to_string());
                }
            }
            MoveType::Catch => {
                // Catch: 적이 있어야 함 (제자리에서 잡기)
                if !has_enemy {
                    return Err("Catch는 적이 있는 칸만 선택할 수 있습니다".to_string());
                }
            }
            MoveType::Shift => {
                // Shift: 아군 또는 적이 있어야 함
                if is_target_empty {
                    return Err("Shift는 다른 기물이 있는 칸만 선택할 수 있습니다".to_string());
                }
            }
            MoveType::TakeMove => {
                // TakeMove: 빈 칸 또는 적
                if has_friendly {
                    return Err("아군 기물이 있는 칸으로 이동할 수 없습니다".to_string());
                }
            }
            MoveType::Jump => {
                // Jump: 빈 칸으로만 이동 (take-jump 조합용)
                if !is_target_empty {
                    return Err("Jump는 빈 칸으로만 이동할 수 있습니다".to_string());
                }
            }
        }
        
        Ok(())
    }

    /// 액션 태그 처리 (이동 후 적용)
    fn apply_action_tags(&mut self, piece_id: &PieceId, tags: &[chessembly::ActionTag]) {
        for tag in tags {
            match tag.tag_type {
                chessembly::ActionTagType::Transition => {
                    // 기물 변환
                    if let Some(piece_name) = &tag.piece_name {
                        if let Some(piece) = self.pieces.get_mut(piece_id) {
                            // 문자열을 PieceKind로 변환
                            let new_kind = match piece_name.to_lowercase().as_str() {
                                "pawn" => PieceKind::Pawn,
                                "king" => PieceKind::King,
                                "queen" => PieceKind::Queen,
                                "rook" => PieceKind::Rook,
                                "knight" => PieceKind::Knight,
                                "bishop" => PieceKind::Bishop,
                                "amazon" => PieceKind::Amazon,
                                "grasshopper" => PieceKind::Grasshopper,
                                "knightrider" => PieceKind::Knightrider,
                                "archbishop" => PieceKind::Archbishop,
                                "dabbaba" => PieceKind::Dabbaba,
                                "alfil" => PieceKind::Alfil,
                                "ferz" => PieceKind::Ferz,
                                "centaur" => PieceKind::Centaur,
                                "camel" => PieceKind::Camel,
                                "tempestrook" => PieceKind::TempestRook,
                                "cannon" => PieceKind::Cannon,
                                "experiment" => PieceKind::Experiment,
                                _ => continue,
                            };
                            
                            // 기물 종류 변환
                            piece.kind = new_kind.clone();
                            // 이동 스택도 새 기물 점수에 맞게 조정
                            piece.move_stack = Self::initial_move_stack(new_kind.score());
                        }
                    }
                }
                chessembly::ActionTagType::SetState => {
                    // 전역 상태 설정
                    self.global_state.insert(tag.key.clone(), tag.value);
                }
            }
        }
    }

    pub fn move_piece_by_legal_moves(&mut self, mv: LegalMove) -> Result<Option<PieceId>, String> {
        let from = mv.from;
        let to = mv.to;
        let tags = mv.tags.clone(); // 태그 복사
    
        // 출발 위치의 기물 확인
        let piece_id = self.board.get(&from).cloned().ok_or("출발 위치에 기물이 없습니다")?;
        let piece = self.pieces.get(&piece_id).cloned().ok_or("기물을 찾을 수 없습니다")?;
        let player = piece.owner;
    
        // 이동 가능성 검사 (기존 검증 로직 재사용)
        self.can_move_piece(player, &piece_id, from, to, mv.move_type)?;
    
        let mut captured_id: Option<PieceId> = None;
    
        match mv.move_type {
            MoveType::Move => {
                self.board.remove(&from);
                self.board.insert(to, piece_id.clone());
                if let Some(p) = self.pieces.get_mut(&piece_id) {
                    p.pos = Some(to);
                    p.move_stack -= 1;
                }
            }
    
            MoveType::Take | MoveType::TakeMove => {
                if let Some(victim_id) = self.board.get(&to).cloned() {
                    captured_id = Some(victim_id.clone());
                    self.capture(&piece_id, &victim_id)?;
                }
    
                self.board.remove(&from);
                self.board.insert(to, piece_id.clone());
    
                if let Some(p) = self.pieces.get_mut(&piece_id) {
                    p.pos = Some(to);
                    if captured_id.is_none() {
                        p.move_stack -= 1;
                    }
                }
            }
    
            MoveType::Catch => {
                // 제자리에서의 잡기: 대상은 `to` 칸에 있어야 함
                if let Some(victim_id) = self.board.get(&to).cloned() {
                    captured_id = Some(victim_id.clone());
                    self.capture(&piece_id, &victim_id)?;
                    // 공격자는 자리 이동하지 않음 (capture()가 스택 갱신 및 제거 처리)
                } else {
                    return Err("Catch 대상이 없습니다".to_string());
                }
            }
    
            MoveType::Shift => {
                // 자리 교환
                if let Some(target_piece_id) = self.board.get(&to).cloned() {
                    self.board.remove(&from);
                    self.board.remove(&to);
                    self.board.insert(from, target_piece_id.clone());
                    self.board.insert(to, piece_id.clone());
    
                    if let Some(p) = self.pieces.get_mut(&piece_id) {
                        p.pos = Some(to);
                        p.move_stack -= 1;
                    }
                    if let Some(tp) = self.pieces.get_mut(&target_piece_id) {
                        tp.pos = Some(from);
                    }
                } else {
                    return Err("Shift 대상이 없습니다".to_string());
                }
            }
    
            MoveType::Jump => {
                // 빈 칸으로 이동
                self.board.remove(&from);
                self.board.insert(to, piece_id.clone());
                if let Some(p) = self.pieces.get_mut(&piece_id) {
                    p.pos = Some(to);
                    p.move_stack -= 1;
                }
    
                // 만약 `catch_to`에 캡처 대상 좌표가 담겨있다면 그 칸의 기물을 제거
                // (현재 코드에서 빈 값을 (0,0)으로 처리하고 있으므로 정확한 sentinel 처리 필요)
                if mv.catch_to.is_valid() {
                    if let Some(victim_id) = self.board.get(&mv.catch_to).cloned() {
                        // 캡처 규칙 적용
                        captured_id = Some(victim_id.clone());
                        self.capture(&piece_id, &victim_id)?;
                    }
                }
            }
        }
    
        // 활성 이동 기물 설정
        self.active_piece = Some(piece_id.clone());
        
        // 액션 태그 적용 (이동 완료 후)
        self.apply_action_tags(&piece_id, &tags);
    
        Ok(captured_id)
    }
    
    /// 이동 실행 (캡처 포함)
    pub fn move_piece(&mut self, player: PlayerId, piece_id: &PieceId, from: Square, to: Square, move_type: MoveType) -> Result<Option<PieceId>, String> {
        self.can_move_piece(player, piece_id, from, to, move_type)?;
        
        let mut captured_id: Option<PieceId> = None;
        
        match move_type {
            MoveType::Move => {
                // Move: 빈 칸으로 이동만
                self.board.remove(&from);
                self.board.insert(to, piece_id.clone());
                
                if let Some(piece) = self.pieces.get_mut(piece_id) {
                    piece.pos = Some(to);
                    piece.move_stack -= 1;
                }
            }
            MoveType::Take | MoveType::TakeMove => {
                // Take/TakeMove: 잡기 또는 이동
                if let Some(victim_id) = self.board.get(&to).cloned() {
                    captured_id = Some(victim_id.clone());
                    self.capture(piece_id, &victim_id)?;
                }
                
                self.board.remove(&from);
                self.board.insert(to, piece_id.clone());
                
                if let Some(piece) = self.pieces.get_mut(piece_id) {
                    piece.pos = Some(to);
                    if captured_id.is_none() {
                        piece.move_stack -= 1;
                    }
                    // capture에서 이미 move_stack 처리됨
                }
            }
            MoveType::Catch => {
                // Catch: 제자리에서 적 제거
                if let Some(victim_id) = self.board.get(&to).cloned() {
                    captured_id = Some(victim_id.clone());
                    // 피해자 정보 복사
                    let victim = self.pieces.get(&victim_id).ok_or("피해자를 찾을 수 없습니다")?.clone();
                    
                    // 공격자는 제자리에 머물지만 스택 업데이트
                    if let Some(attacker) = self.pieces.get_mut(piece_id) {
                        // Catch: 이동 스택 -1 + 피해자 스택
                        attacker.move_stack = attacker.move_stack - 1 + victim.move_stack;
                        // 스턴 스택: 피해자 스택 추가
                        attacker.stun += victim.stun;
                    }
                    
                    // 피해자 제거
                    self.board.remove(&to);
                    self.pieces.remove(&victim_id);
                } else {
                    return Err("Catch 대상이 없습니다".to_string());
                }
            }
            MoveType::Shift => {
                // Shift: 자리 바꾸기
                if let Some(target_piece_id) = self.board.get(&to).cloned() {
                    // 두 기물의 위치 교환
                    self.board.remove(&from);
                    self.board.remove(&to);
                    self.board.insert(from, target_piece_id.clone());
                    self.board.insert(to, piece_id.clone());
                    
                    // 위치 업데이트
                    if let Some(piece) = self.pieces.get_mut(piece_id) {
                        piece.pos = Some(to);
                        piece.move_stack -= 1;
                    }
                    if let Some(target_piece) = self.pieces.get_mut(&target_piece_id) {
                        target_piece.pos = Some(from);
                    }
                } else {
                    return Err("Shift 대상이 없습니다".to_string());
                }
            }
            MoveType::Jump => {
                // Jump: take-jump 조합 (빈 칸으로 이동)
                self.board.remove(&from);
                self.board.insert(to, piece_id.clone());
                
                if let Some(piece) = self.pieces.get_mut(piece_id) {
                    piece.pos = Some(to);
                    piece.move_stack -= 1;
                }
            }
        }
        
        // 이동 중인 기물 설정
        self.active_piece = Some(piece_id.clone());
        
        Ok(captured_id)
    }
    
    /// 캡처 처리 (stack.md 규칙)
    pub fn capture(&mut self, attacker_id: &PieceId, victim_id: &PieceId) -> Result<(), String> {
        // 피해자 정보 복사
        let victim = self.pieces.get(victim_id).ok_or("피해자를 찾을 수 없습니다")?.clone();
        
        // 공격자 스택 업데이트
        if let Some(attacker) = self.pieces.get_mut(attacker_id) {
            // 이동 스택: -1 (이동 소비) + 피해자 스택
            attacker.move_stack = attacker.move_stack - 1 + victim.move_stack;
            // 스턴 스택: 피해자 스택 추가
            attacker.stun += victim.stun;
        }
        
        // 피해자 제거
        if let Some(pos) = victim.pos {
            self.board.remove(&pos);
        }
        self.pieces.remove(victim_id);
        
        Ok(())
    }
    
    /// 계승 (기물을 로얄 피스로)
    pub fn crown_piece(&mut self, player: PlayerId, piece_id: &PieceId) -> Result<(), String> {
        if self.turn != player {
            return Err("자신의 턴이 아닙니다".to_string());
        }
        if self.action_taken || self.active_piece.is_some() {
            return Err("이번 턴에 이미 행동했습니다".to_string());
        }
        
        let piece = self.pieces.get_mut(piece_id).ok_or("기물을 찾을 수 없습니다")?;
        if piece.owner != player {
            return Err("자신의 기물이 아닙니다".to_string());
        }
        if piece.pos.is_none() {
            return Err("보드 위의 기물만 계승할 수 있습니다".to_string());
        }
        
        piece.is_royal = true;
        self.action_taken = true;
        Ok(())
    }
    
    /// 위장 (로얄 피스를 다른 기물로)
    pub fn disguise_piece(&mut self, player: PlayerId, piece_id: &PieceId, as_kind: PieceKind) -> Result<(), String> {
        if self.turn != player {
            return Err("자신의 턴이 아닙니다".to_string());
        }
        if self.action_taken || self.active_piece.is_some() {
            return Err("이번 턴에 이미 행동했습니다".to_string());
        }
        
        let piece = self.pieces.get_mut(piece_id).ok_or("기물을 찾을 수 없습니다")?;
        if piece.owner != player {
            return Err("자신의 기물이 아닙니다".to_string());
        }
        if !piece.is_royal {
            return Err("로얄 피스만 위장할 수 있습니다".to_string());
        }
        
        // 위장 시 이동 스택은 위장 기물 기준, 스턴은 유지
        let new_score = as_kind.score();
        piece.move_stack = Self::initial_move_stack(new_score);
        piece.disguise = Some(as_kind);
        self.action_taken = true;
        Ok(())
    }
    
    /// 스턴 부여 (적 1, 아군 1~3)
    pub fn apply_stun(&mut self, player: PlayerId, target_id: &PieceId, amount: i32) -> Result<(), String> {
        if self.turn != player {
            return Err("자신의 턴이 아닙니다".to_string());
        }
        if self.action_taken || self.active_piece.is_some() {
            return Err("이번 턴에 이미 행동했습니다".to_string());
        }
        
        let piece = self.pieces.get_mut(target_id).ok_or("기물을 찾을 수 없습니다")?;
        
        if piece.owner == player {
            // 아군: 1~3 스택
            if amount < 1 || amount > 3 {
                return Err("아군에게는 1~3 스턴만 부여할 수 있습니다".to_string());
            }
        } else {
            // 적: 1 스택만
            if amount != 1 {
                return Err("적에게는 1 스턴만 부여할 수 있습니다".to_string());
            }
        }
        
        piece.stun += amount;
        self.action_taken = true;
        Ok(())
    }
    
    /// 턴 종료
    pub fn end_turn(&mut self) {
        // 현재 턴 기물만 스턴 1 감소
        for piece in self.pieces.values_mut() {
            if piece.owner == self.turn {
                piece.stun = (piece.stun - 1).max(0);
            }
        }
        
        // 다음 플레이어
        self.turn = 1 - self.turn;
        
        // 다음 턴 기물들 이동 스택 초기화
        for piece in self.pieces.values_mut() {
            if piece.owner == self.turn && piece.pos.is_some() {
                piece.move_stack = Self::initial_move_stack(piece.score());
            }
        }
        
        // 턴 상태 초기화
        self.active_piece = None;
        self.action_taken = false;
    }
    
    /// 승리 조건 확인
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
    
    /// 특정 위치의 기물 가져오기
    pub fn get_piece_at(&self, square: Square) -> Option<&Piece> {
        self.board.get(&square).and_then(|id| self.pieces.get(id))
    }
    
    /// GameState를 ChessemblyBoard로 변환
    fn to_chessembly_board(&self, piece_id: &PieceId) -> Option<ChessemblyBoard> {
        let piece = self.pieces.get(piece_id)?;
        let pos = piece.pos?;
        
        let mut pieces_map: HashMap<(i32, i32), (String, bool)> = HashMap::new();
        for (sq, pid) in &self.board {
            if let Some(p) = self.pieces.get(pid) {
                pieces_map.insert(
                    (sq.x, sq.y),
                    (format!("{:?}", p.effective_kind()), p.is_white()),
                );
            }
        }
        
        Some(ChessemblyBoard {
            board_width: 8,
            board_height: 8,
            piece_x: pos.x,
            piece_y: pos.y,
            piece_name: format!("{:?}", piece.effective_kind()),
            is_white: piece.is_white(),
            pieces: pieces_map,
            state: self.global_state.clone(),
            danger_squares: HashSet::new(), // TODO: 위협 계산
            in_check: false, // TODO: 체크 계산
        })
    }
    
    /// 특정 기물의 이동 가능한 칸 목록 계산 (chessembly 사용)
    pub fn get_legal_moves(&self, piece_id: &PieceId) -> Vec<LegalMove> {
        let mut legal_moves = Vec::new();
        
        let piece = match self.pieces.get(piece_id) {
            Some(p) => p,
            None => return legal_moves,
        };
        
        // 이동 불가 상태 확인
        if !piece.can_move() {
            return legal_moves;
        }
        
        let pos = match piece.pos {
            Some(p) => p,
            None => return legal_moves,
        };
        
        // chessembly 보드 상태 생성
        let mut board = match self.to_chessembly_board(piece_id) {
            Some(b) => b,
            None => return legal_moves,
        };
        
        // 행마법 스크립트 가져오기
        let script = piece.effective_kind().chessembly_script(piece.is_white());
        
        // chessembly 인터프리터 실행
        let mut interpreter = Interpreter::new();
        interpreter.set_debug(self.debug_mode);
        interpreter.parse(script);
        let activations = interpreter.execute(&mut board);
        
        // 활성화된 칸들을 LegalMove로 변환
        for activation in activations {
            let target = Square::new(pos.x + activation.dx, pos.y + activation.dy);
            let mut takemove_sq = Square::new(0, 0);
            if let Some((x, y)) = activation.catch_to {
                takemove_sq = Square::new(pos.x + x, pos.y + y);
            }
            
            // 보드 범위 확인
            if !target.is_valid() {
                continue;
            }
            
            let is_capture = self.board.contains_key(&target);
            
            legal_moves.push(LegalMove {
                from: pos,
                to: target,
                move_type: activation.move_type,
                is_capture,
                tags: activation.tags,
                catch_to: takemove_sq,
            });
        }
        
        legal_moves
    }
    
    /// 이동이 유효한지 확인 (chessembly 기반)
    pub fn is_valid_move(&self, piece_id: &PieceId, from: Square, to: Square) -> bool {
        let legal_moves = self.get_legal_moves(piece_id);
        legal_moves.iter().any(|m| m.from == from && m.to == to)
    }
    
    /// 이동의 MoveType 찾기
    pub fn get_move_type(&self, piece_id: &PieceId, from: Square, to: Square) -> Option<MoveType> {
        let legal_moves = self.get_legal_moves(piece_id);
        legal_moves.iter()
            .find(|m| m.from == from && m.to == to)
            .map(|m| m.move_type)
    }
    
    /// 프로모션 실행
    pub fn promote(&mut self, piece_id: &PieceId, to_kind: PieceKind) -> Result<(), String> {
        let piece = self.pieces.get(piece_id).ok_or("기물을 찾을 수 없습니다")?;
        
        // 프로모션 가능한 기물인지
        if !piece.kind.can_promote() {
            return Err("프로모션할 수 없는 기물입니다".to_string());
        }
        
        // 유효한 프로모션 대상인지
        if !piece.kind.promotion_targets().contains(&to_kind) {
            return Err("유효하지 않은 프로모션 대상입니다".to_string());
        }
        
        // 프로모션 칸에 있는지
        let pos = piece.pos.ok_or("보드 위에 없는 기물입니다")?;
        if !piece.kind.is_promotion_square(pos, piece.is_white()) {
            return Err("프로모션 칸에 있지 않습니다".to_string());
        }
        
        // 프로모션 실행 (스택 계승)
        if let Some(piece) = self.pieces.get_mut(piece_id) {
            piece.kind = to_kind;
            // 스택은 유지 (promotion.md: 이전 기물의 모든 스택값이 계승)
        }
        
        Ok(())
    }
    
    // === WASM용 추가 메서드들 ===
    
    /// 인자 없이 새 게임 생성
    pub fn new_default() -> Self {
        Self::new(0)
    }
    
    /// 초기 포지션 설정 (킹 + 기본 포켓)
    pub fn setup_initial_position(&mut self) {
        // 킹은 이미 배치됨
        // 기본 포켓 설정 (표준 체스 기물들)
        let white_pocket = vec![
            PieceSpec::new(PieceKind::Queen),
            PieceSpec::new(PieceKind::Rook),
            PieceSpec::new(PieceKind::Rook),
            PieceSpec::new(PieceKind::Bishop),
            PieceSpec::new(PieceKind::Bishop),
            PieceSpec::new(PieceKind::Knight),
            PieceSpec::new(PieceKind::Knight),
            PieceSpec::new(PieceKind::Pawn),
            PieceSpec::new(PieceKind::Pawn),
            PieceSpec::new(PieceKind::Pawn),
            PieceSpec::new(PieceKind::Pawn),
            PieceSpec::new(PieceKind::Pawn),
            PieceSpec::new(PieceKind::Pawn),
            PieceSpec::new(PieceKind::Pawn),
            PieceSpec::new(PieceKind::Pawn),
        ];
        let _ = self.setup_pocket(0, white_pocket);
        
        let black_pocket = vec![
            PieceSpec::new(PieceKind::Queen),
            PieceSpec::new(PieceKind::Rook),
            PieceSpec::new(PieceKind::Rook),
            PieceSpec::new(PieceKind::Bishop),
            PieceSpec::new(PieceKind::Bishop),
            PieceSpec::new(PieceKind::Knight),
            PieceSpec::new(PieceKind::Knight),
            PieceSpec::new(PieceKind::Pawn),
            PieceSpec::new(PieceKind::Pawn),
            PieceSpec::new(PieceKind::Pawn),
            PieceSpec::new(PieceKind::Pawn),
            PieceSpec::new(PieceKind::Pawn),
            PieceSpec::new(PieceKind::Pawn),
            PieceSpec::new(PieceKind::Pawn),
            PieceSpec::new(PieceKind::Pawn),
        ];
        let _ = self.setup_pocket(1, black_pocket);
    }
    
    /// 실험용 포켓 설정 (다양한 기물들)
    pub fn setup_experimental_pocket(&mut self) {
        // 백 실험용 포켓 (점수 무제한)
        let white_pocket = vec![
            PieceSpec::new(PieceKind::Amazon),
            PieceSpec::new(PieceKind::Grasshopper),
            PieceSpec::new(PieceKind::Knightrider),
            PieceSpec::new(PieceKind::Archbishop),
            PieceSpec::new(PieceKind::Dabbaba),
            PieceSpec::new(PieceKind::Alfil),
            PieceSpec::new(PieceKind::Ferz),
            PieceSpec::new(PieceKind::Centaur),
            PieceSpec::new(PieceKind::Camel),
            PieceSpec::new(PieceKind::TempestRook),
            PieceSpec::new(PieceKind::Cannon),
            PieceSpec::new(PieceKind::Experiment),
        ];
        self.setup_pocket_unchecked(0, white_pocket);
        
        // 흑 실험용 포켓 (점수 무제한)
        let black_pocket = vec![
            PieceSpec::new(PieceKind::Amazon),
            PieceSpec::new(PieceKind::Grasshopper),
            PieceSpec::new(PieceKind::Knightrider),
            PieceSpec::new(PieceKind::Archbishop),
            PieceSpec::new(PieceKind::Dabbaba),
            PieceSpec::new(PieceKind::Alfil),
            PieceSpec::new(PieceKind::Ferz),
            PieceSpec::new(PieceKind::Centaur),
            PieceSpec::new(PieceKind::Camel),
            PieceSpec::new(PieceKind::TempestRook),
            PieceSpec::new(PieceKind::Cannon),
            PieceSpec::new(PieceKind::Experiment),
        ];
        self.setup_pocket_unchecked(1, black_pocket);
    }
    
    /// 현재 플레이어 가져오기
    pub fn current_player(&self) -> PlayerId {
        self.turn
    }
    
    /// 모든 기물 가져오기 (보드 위의 기물만)
    pub fn get_all_pieces(&self) -> Vec<PieceInfo> {
        self.pieces.values()
            .filter(|p| p.pos.is_some())
            .map(|p| PieceInfo {
                id: p.id.clone(),
                kind: p.kind.clone(),
                owner: p.owner,
                pos: p.pos.unwrap(),
                stun_stack: p.stun,
                move_stack: p.move_stack,
                is_royal: p.is_royal,
            })
            .collect()
    }
    
    /// 특정 플레이어의 포켓 가져오기
    pub fn get_pocket(&self, player: PlayerId) -> Vec<PieceKind> {
        self.pockets.get(&player)
            .map(|specs| specs.iter().map(|s| s.kind.clone()).collect())
            .unwrap_or_default()
    }
    
    /// 포켓에서 배치 가능한지 확인
    pub fn can_place_from_pocket(&self, kind: &PieceKind, square: Square) -> bool {
        self.can_place(self.turn, kind, square).is_ok()
    }
    
    /// 특정 위치의 기물 이동 가능 칸 계산 (Square로 조회)
    pub fn get_legal_moves_at(&self, square: Square) -> Vec<LegalMove> {
        if let Some(piece_id) = self.board.get(&square) {
            self.get_legal_moves(piece_id)
        } else {
            Vec::new()
        }
    }
    
    /// 이동 유효성 확인 (Square로 조회)
    pub fn is_valid_move_at(&self, from: Square, to: Square) -> bool {
        if let Some(piece_id) = self.board.get(&from) {
            self.is_valid_move(piece_id, from, to)
        } else {
            false
        }
    }
    
    /// 기물에 스턴 부여
    pub fn stun_piece(&mut self, piece_id: &PieceId, amount: i32) -> Result<(), String> {
        let piece = self.pieces.get_mut(piece_id).ok_or("기물을 찾을 수 없습니다")?;
        
        // 아군: 1~3, 적: 1
        let is_ally = piece.owner == self.turn;
        if is_ally {
            if amount < 1 || amount > 3 {
                return Err("아군에게는 1~3 스턴만 부여할 수 있습니다".to_string());
            }
        } else {
            if amount != 1 {
                return Err("적에게는 1 스턴만 부여할 수 있습니다".to_string());
            }
        }
        
        piece.stun += amount;
        self.action_taken = true;
        Ok(())
    }
    
    /// 액션 적용
    pub fn apply_action(&mut self, action: Action) {
        match action {
            Action::Place { piece_id, target } => {
                // 포켓에서 해당 기물 찾아서 배치
                if let Some(piece) = self.pieces.get(&piece_id) {
                    let _ = self.can_place(self.turn, &piece.kind, target);
                    // TODO: 실제 배치 로직
                }
            }
            Action::Move { piece_id, from, to } => {
                // MoveType 찾기
                let legal_moves = self.get_legal_moves_at(from);
                for legal_move in legal_moves {
                    if to == legal_move.to {
                        let _ = self.move_piece_by_legal_moves(legal_move);
                    } 
                }
            }
            Action::Stun { piece_id, amount } => {
                let _ = self.stun_piece(&piece_id, amount);
            }
            Action::Crown { piece_id } => {
                if let Some(piece) = self.pieces.get_mut(&piece_id) {
                    piece.is_royal = true;
                }
            }
            Action::Disguise { piece_id, as_kind } => {
                if let Some(piece) = self.pieces.get_mut(&piece_id) {
                    piece.disguise = Some(as_kind);
                }
            }
        }
    }
}

/// JS용 기물 정보 구조체
#[derive(Debug, Clone)]
pub struct PieceInfo {
    pub id: PieceId,
    pub kind: PieceKind,
    pub owner: PlayerId,
    pub pos: Square,
    pub stun_stack: i32,
    pub move_stack: i32,
    pub is_royal: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_initial_setup() {
        let state = GameState::new(0);
        
        // 킹 위치 확인
        let white_king = state.get_piece_at(Square::new(4, 0));
        assert!(white_king.is_some());
        assert_eq!(white_king.unwrap().kind, PieceKind::King);
        assert!(white_king.unwrap().is_royal);
        assert_eq!(white_king.unwrap().stun, 0);
        assert_eq!(white_king.unwrap().move_stack, 3);
        
        let black_king = state.get_piece_at(Square::new(4, 7));
        assert!(black_king.is_some());
        assert_eq!(black_king.unwrap().kind, PieceKind::King);
    }
    
    #[test]
    fn test_move_stack_calculation() {
        assert_eq!(GameState::initial_move_stack(1), 5);  // 폰
        assert_eq!(GameState::initial_move_stack(2), 5);  // 다바바, 알필
        assert_eq!(GameState::initial_move_stack(3), 3);  // 나이트, 비숍
        assert_eq!(GameState::initial_move_stack(5), 3);  // 룩
        assert_eq!(GameState::initial_move_stack(7), 2);  // 나이트라이더
        assert_eq!(GameState::initial_move_stack(9), 1);  // 퀸
        assert_eq!(GameState::initial_move_stack(13), 1); // 아마존
    }
    
    #[test]
    fn test_piece_scores() {
        assert_eq!(PieceKind::Pawn.score(), 1);
        assert_eq!(PieceKind::King.score(), 4);
        assert_eq!(PieceKind::Queen.score(), 9);
        assert_eq!(PieceKind::Rook.score(), 5);
        assert_eq!(PieceKind::Knight.score(), 3);
        assert_eq!(PieceKind::Bishop.score(), 3);
        assert_eq!(PieceKind::Amazon.score(), 13);
    }
    
    #[test]
    fn test_pocket_score_limit() {
        let mut state = GameState::new(0);
        
        // 39점 이하는 OK
        let valid_pocket = vec![
            PieceSpec { kind: PieceKind::Queen },  // 9
            PieceSpec { kind: PieceKind::Rook },   // 5
            PieceSpec { kind: PieceKind::Rook },   // 5
            PieceSpec { kind: PieceKind::Bishop }, // 3
            PieceSpec { kind: PieceKind::Bishop }, // 3
            PieceSpec { kind: PieceKind::Knight }, // 3
            PieceSpec { kind: PieceKind::Knight }, // 3
            PieceSpec { kind: PieceKind::Pawn },   // 1
            PieceSpec { kind: PieceKind::Pawn },   // 1
            PieceSpec { kind: PieceKind::Pawn },   // 1
            PieceSpec { kind: PieceKind::Pawn },   // 1
            PieceSpec { kind: PieceKind::Pawn },   // 1
            PieceSpec { kind: PieceKind::Pawn },   // 1
            PieceSpec { kind: PieceKind::Pawn },   // 1
            PieceSpec { kind: PieceKind::Pawn },   // 1
        ]; // 총 39점
        assert!(state.setup_pocket(0, valid_pocket).is_ok());
        
        // 39점 초과는 에러
        let invalid_pocket = vec![
            PieceSpec { kind: PieceKind::Amazon }, // 13
            PieceSpec { kind: PieceKind::Queen },  // 9
            PieceSpec { kind: PieceKind::Queen },  // 9
            PieceSpec { kind: PieceKind::Queen },  // 9
        ]; // 총 40점
        assert!(state.setup_pocket(1, invalid_pocket).is_err());
    }
    
    #[test]
    fn test_capture_stack_transfer() {
        let mut state = GameState::new(0);
        
        // 공격자 배치 (나이트: 3점, 이동3)
        let attacker = state.create_piece(PieceKind::Knight, 0);
        let attacker_id = attacker.id.clone();
        state.pieces.insert(attacker_id.clone(), attacker);
        if let Some(p) = state.pieces.get_mut(&attacker_id) {
            p.pos = Some(Square::new(0, 0));
            p.move_stack = 3;
            p.stun = 0;
        }
        state.board.insert(Square::new(0, 0), attacker_id.clone());
        
        // 피해자 배치 (룩: 5점, 이동3, 스턴2)
        let victim = state.create_piece(PieceKind::Rook, 1);
        let victim_id = victim.id.clone();
        state.pieces.insert(victim_id.clone(), victim);
        if let Some(p) = state.pieces.get_mut(&victim_id) {
            p.pos = Some(Square::new(2, 1));
            p.move_stack = 3;
            p.stun = 2;
        }
        state.board.insert(Square::new(2, 1), victim_id.clone());
        
        // 캡처 실행
        state.capture(&attacker_id, &victim_id).unwrap();
        
        // 결과 확인
        let attacker = state.pieces.get(&attacker_id).unwrap();
        // 이동 스택: 3 - 1 + 3 = 5
        assert_eq!(attacker.move_stack, 5);
        // 스턴 스택: 0 + 2 = 2
        assert_eq!(attacker.stun, 2);
        
        // 피해자 제거됨
        assert!(state.pieces.get(&victim_id).is_none());
    }
    
    #[test]
    fn test_victory_condition() {
        let mut state = GameState::new(0);
        assert_eq!(state.check_victory(), GameResult::Ongoing);
        
        // 흑 킹 제거
        let black_king_id = state.board.get(&Square::new(4, 7)).cloned();
        if let Some(id) = black_king_id {
            state.board.remove(&Square::new(4, 7));
            state.pieces.remove(&id);
        }
        
        assert_eq!(state.check_victory(), GameResult::WhiteWins);
    }
    
    #[test]
    fn test_square_notation() {
        let e4 = Square::from_notation("e4").unwrap();
        assert_eq!(e4.x, 4);
        assert_eq!(e4.y, 3);
        assert_eq!(e4.to_notation(), "e4");
        
        let a1 = Square::from_notation("a1").unwrap();
        assert_eq!(a1.x, 0);
        assert_eq!(a1.y, 0);
        
        let h8 = Square::from_notation("h8").unwrap();
        assert_eq!(h8.x, 7);
        assert_eq!(h8.y, 7);
    }
    
    #[test]
    fn test_pawn_promotion_stun() {
        let state = GameState::new(0);
        
        // 폰 생성
        let pawn = Piece::new("pawn1".to_string(), PieceKind::Pawn, 0);
        
        // 1랭크(y=0) 착수: 스턴 0
        let stun_at_1 = state.calculate_placement_stun(&pawn, Square::new(0, 0));
        assert_eq!(stun_at_1, 0);
        
        // 7랭크(y=6) 착수: 스턴 ~7 (거리 1)
        let stun_at_7 = state.calculate_placement_stun(&pawn, Square::new(0, 6));
        assert!(stun_at_7 > 0);
    }
    
    #[test]
    fn test_crown_piece() {
        let mut state = GameState::new(0);
        
        // 폰 배치
        state.pockets.insert(0, vec![PieceSpec { kind: PieceKind::Pawn }]);
        let pawn_id = state.place_piece(0, PieceKind::Pawn, Square::new(0, 1)).unwrap();
        
        // 턴 종료 후 계승
        state.end_turn();
        state.end_turn();
        state.action_taken = false;
        
        assert!(state.crown_piece(0, &pawn_id).is_ok());
        assert!(state.pieces.get(&pawn_id).unwrap().is_royal);
    }
    
    #[test]
    fn test_pawn_cannot_place_on_promotion_rank() {
        let mut state = GameState::new(0);
        state.pockets.insert(0, vec![PieceSpec { kind: PieceKind::Pawn }]);
        
        // 8랭크(y=7)에 폰 착수 시도 - 실패해야 함
        let result = state.place_piece(0, PieceKind::Pawn, Square::new(0, 7));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_king_legal_moves() {
        let state = GameState::new(0);
        
        // 백 킹 (e1)의 이동 가능 칸 확인
        let white_king_id = state.board.get(&Square::new(4, 0)).unwrap().clone();
        let moves = state.get_legal_moves(&white_king_id);
        
        // e1에서 킹이 갈 수 있는 칸: d1, f1, d2, e2, f2 (5칸)
        assert!(!moves.is_empty());
        
        // d2로 이동 가능한지 확인
        assert!(moves.iter().any(|m| m.to == Square::new(3, 1)));
        // e2로 이동 가능한지 확인
        assert!(moves.iter().any(|m| m.to == Square::new(4, 1)));
    }
    
    #[test]
    fn test_rook_legal_moves() {
        let mut state = GameState::new(0);
        
        // 룩 배치 (d4)
        let rook = state.create_piece(PieceKind::Rook, 0);
        let rook_id = rook.id.clone();
        state.pieces.insert(rook_id.clone(), rook);
        if let Some(p) = state.pieces.get_mut(&rook_id) {
            p.pos = Some(Square::new(3, 3)); // d4
            p.move_stack = 3;
            p.stun = 0;
        }
        state.board.insert(Square::new(3, 3), rook_id.clone());
        
        // chessembly 직접 테스트
        let script = "take-move(1, 0) repeat(1); take-move(-1, 0) repeat(1); take-move(0, 1) repeat(1); take-move(0, -1) repeat(1);";
        
        let mut board = state.to_chessembly_board(&rook_id).unwrap();
        let mut interpreter = Interpreter::new();
        interpreter.set_debug(state.debug_mode);
        interpreter.parse(script);
        let activations = interpreter.execute(&mut board);
        
        println!("Script: {}", script);
        println!("Piece at: ({}, {})", board.piece_x, board.piece_y);
        println!("Activations count: {}", activations.len());
        for a in &activations {
            let target_x = board.piece_x + a.dx;
            let target_y = board.piece_y + a.dy;
            println!("  dx={}, dy={} -> ({}, {})", a.dx, a.dy, target_x, target_y);
        }
        
        // 오른쪽으로 이동 가능
        assert!(activations.iter().any(|a| a.dx == 1 && a.dy == 0), "오른쪽 이동 필요");
        // 왼쪽으로 이동 가능
        assert!(activations.iter().any(|a| a.dx == -1 && a.dy == 0), "왼쪽 이동 필요");
        // 위로 이동 가능
        assert!(activations.iter().any(|a| a.dx == 0 && a.dy == 1), "위 이동 필요");
        // 아래로 이동 가능
        assert!(activations.iter().any(|a| a.dx == 0 && a.dy == -1), "아래 이동 필요");
    }
    
    #[test]
    fn test_knight_legal_moves() {
        let mut state = GameState::new(0);
        
        // 나이트 배치 (d4)
        let knight = state.create_piece(PieceKind::Knight, 0);
        let knight_id = knight.id.clone();
        state.pieces.insert(knight_id.clone(), knight);
        if let Some(p) = state.pieces.get_mut(&knight_id) {
            p.pos = Some(Square::new(3, 3)); // d4
            p.move_stack = 3;
            p.stun = 0;
        }
        state.board.insert(Square::new(3, 3), knight_id.clone());
        
        let moves = state.get_legal_moves(&knight_id);
        
        // 나이트 L자 이동: b3, b5, c2, c6, e2, e6, f3, f5 (8칸)
        assert_eq!(moves.len(), 8);
        
        // b5 (1,4)로 이동 가능
        assert!(moves.iter().any(|m| m.to == Square::new(1, 4)));
        // f5 (5,4)로 이동 가능
        assert!(moves.iter().any(|m| m.to == Square::new(5, 4)));
    }
    
    #[test]
    fn test_is_valid_move() {
        let state = GameState::new(0);
        
        let white_king_id = state.board.get(&Square::new(4, 0)).unwrap().clone();
        
        // e1 -> e2: 유효
        assert!(state.is_valid_move(&white_king_id, Square::new(4, 0), Square::new(4, 1)));
        
        // e1 -> e3: 킹은 2칸 이동 불가
        assert!(!state.is_valid_move(&white_king_id, Square::new(4, 0), Square::new(4, 2)));
    }
}
